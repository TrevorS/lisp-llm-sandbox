//! Database operations with backend-agnostic design
//!
//! Provides minimal primitives for database access. High-level abstractions
//! live in the Lisp stdlib (src/stdlib/lisp/db.lisp).
//!
//! - `db:open`: Open database connection from spec map
//! - `db:close`: Close database connection
//! - `db:exec`: Execute SQL statement (INSERT, UPDATE, DELETE, CREATE)
//! - `db:query`: Execute SELECT query, returns list of row maps
//!
//! ## Connection Design
//!
//! Connections are represented as maps:
//! - Input spec: `{:backend "sqlite" :path "users.db"}`
//! - Returned connection: `{:backend "sqlite" :path "users.db" :handle 42}`
//!
//! The `:handle` field contains a unique connection ID used to look up
//! the actual database connection in the thread-local registry.

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;
use rusqlite::{params_from_iter, Connection};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use super::SANDBOX;

// ============================================================================
// Connection Registry (Thread-Local)
// ============================================================================

thread_local! {
    static CONNECTIONS: RefCell<HashMap<u64, Connection>> = RefCell::new(HashMap::new());
}

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

/// Generate a unique connection handle
fn next_handle() -> u64 {
    NEXT_HANDLE.fetch_add(1, Ordering::SeqCst)
}

/// Store a connection and return its handle
fn store_connection(conn: Connection) -> u64 {
    let handle = next_handle();
    CONNECTIONS.with(|conns| {
        conns.borrow_mut().insert(handle, conn);
    });
    handle
}

/// Remove a connection from the registry
fn remove_connection(handle: u64) -> Result<(), EvalError> {
    CONNECTIONS.with(|conns| {
        conns
            .borrow_mut()
            .remove(&handle)
            .ok_or_else(|| EvalError::IoError(format!("Invalid connection handle: {}", handle)))
            .map(|_| ())
    })
}

/// Execute a function with a connection from the registry
fn with_connection<F, R>(handle: u64, f: F) -> Result<R, EvalError>
where
    F: FnOnce(&Connection) -> Result<R, EvalError>,
{
    CONNECTIONS.with(|conns| {
        let conns_ref = conns.borrow();
        let conn = conns_ref
            .get(&handle)
            .ok_or_else(|| EvalError::IoError(format!("Invalid connection handle: {}", handle)))?;
        f(conn)
    })
}

// ============================================================================
// Database Primitives
// ============================================================================

#[builtin(name = "db:open", category = "Database", related(db:close, db:exec, db:query))]
/// Opens a database connection from a connection spec map.
///
/// The spec must contain a `:backend` key. Currently only "sqlite" is supported.
/// For SQLite, spec must also contain `:path` key.
///
/// Returns a connection map with `:handle` field added.
///
/// # Examples
///
/// ```lisp
/// (db:open {:backend "sqlite" :path "users.db"})
/// => {:backend "sqlite" :path "users.db" :handle 1}
/// ```
///
/// # See Also
///
/// db:close, db:exec, db:query
pub fn db_open(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let spec = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    // Extract backend
    let backend = spec
        .get("backend")
        .ok_or_else(|| EvalError::IoError("Connection spec missing :backend key".to_string()))?;

    let backend_str = match backend {
        Value::String(s) => s.as_str(),
        _ => return Err(EvalError::IoError("Backend must be a string".to_string())),
    };

    // Currently only SQLite is supported
    match backend_str {
        "sqlite" => {
            // Extract path
            let path = spec
                .get("path")
                .ok_or_else(|| {
                    EvalError::IoError("SQLite connection spec missing :path key".to_string())
                })?;

            let path_str = match path {
                Value::String(s) => s.as_str(),
                _ => return Err(EvalError::IoError("Path must be a string".to_string())),
            };

            // Validate and construct full path through sandbox
            SANDBOX.with(|s| {
                let sandbox_ref = s.borrow();
                let sandbox = sandbox_ref
                    .as_ref()
                    .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

                // Validate path format (no absolute paths, no .. traversals)
                if path_str.starts_with('/') || path_str.starts_with("\\") {
                    return Err(EvalError::IoError(format!(
                        "Absolute paths not allowed: {}",
                        path_str
                    )));
                }

                if path_str.contains("..") {
                    return Err(EvalError::IoError(format!(
                        "Path traversal not allowed: {}",
                        path_str
                    )));
                }

                // Get the full path by joining with first allowed path
                let full_path = sandbox
                    .get_full_path(path_str)
                    .map_err(|e| EvalError::IoError(e.to_string()))?;

                // Open connection
                let conn = Connection::open(&full_path)
                    .map_err(|e| EvalError::IoError(format!("Failed to open database: {}", e)))?;

                // Store connection and get handle
                let handle = store_connection(conn);

                // Build result map with handle added
                let mut result = spec.clone();
                result.insert("handle".to_string(), Value::Number(handle as f64));

                Ok(Value::Map(result))
            })
        }
        _ => Err(EvalError::IoError(format!(
            "Unsupported database backend: {}",
            backend_str
        ))),
    }
}

#[builtin(name = "db:close", category = "Database", related(db:open, db:exec, db:query))]
/// Closes a database connection.
///
/// Takes a connection map with `:handle` field. Returns #t on success.
///
/// # Examples
///
/// ```lisp
/// (db:close conn) => #t
/// ```
///
/// # See Also
///
/// db:open, db:exec, db:query
pub fn db_close(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let conn_map = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    // Extract handle
    let handle_val = conn_map
        .get("handle")
        .ok_or_else(|| EvalError::IoError("Connection map missing :handle key".to_string()))?;

    let handle = match handle_val {
        Value::Number(n) => *n as u64,
        _ => return Err(EvalError::IoError("Handle must be a number".to_string())),
    };

    // Remove connection from registry
    remove_connection(handle)?;

    Ok(Value::Bool(true))
}

#[builtin(name = "db:exec", category = "Database", related(db:open, db:close, db:query))]
/// Executes a SQL statement (INSERT, UPDATE, DELETE, CREATE, etc.).
///
/// Returns the number of rows affected.
///
/// Takes:
/// - Connection map (with `:handle` field)
/// - SQL string
/// - Optional list of parameters
///
/// # Examples
///
/// ```lisp
/// (db:exec conn "CREATE TABLE users (id INTEGER, name TEXT)" '())
/// => 0
/// (db:exec conn "INSERT INTO users VALUES (?, ?)" '(1 "Alice"))
/// => 1
/// ```
///
/// # See Also
///
/// db:query, db:open, db:close
pub fn db_exec(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(EvalError::ArityMismatch);
    }

    // Extract connection handle
    let conn_map = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    let handle_val = conn_map
        .get("handle")
        .ok_or_else(|| EvalError::IoError("Connection map missing :handle key".to_string()))?;

    let handle = match handle_val {
        Value::Number(n) => *n as u64,
        _ => return Err(EvalError::IoError("Handle must be a number".to_string())),
    };

    // Extract SQL
    let sql = match &args[1] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    // Extract optional parameters
    let params: Vec<Value> = if args.len() == 3 {
        match &args[2] {
            Value::List(items) => items.clone(),
            Value::Nil => Vec::new(),
            _ => return Err(EvalError::TypeError),
        }
    } else {
        Vec::new()
    };

    // Execute with connection
    with_connection(handle, |conn| {
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| EvalError::IoError(format!("Failed to prepare statement: {}", e)))?;

        // Convert Value parameters to rusqlite parameters
        let rusqlite_params: Vec<rusqlite::types::Value> = params
            .iter()
            .map(value_to_rusqlite)
            .collect::<Result<Vec<_>, _>>()?;

        let rows_affected = stmt
            .execute(params_from_iter(rusqlite_params.iter()))
            .map_err(|e| EvalError::IoError(format!("Failed to execute statement: {}", e)))?;

        Ok(Value::Number(rows_affected as f64))
    })
}

#[builtin(name = "db:query", category = "Database", related(db:exec, db:open, db:close))]
/// Executes a SELECT query and returns results as a list of row maps.
///
/// Each row is represented as a map with column names as keys.
///
/// Takes:
/// - Connection map (with `:handle` field)
/// - SQL string
/// - Optional list of parameters
///
/// # Examples
///
/// ```lisp
/// (db:query conn "SELECT * FROM users" '())
/// => ({:id 1 :name "Alice"} {:id 2 :name "Bob"})
/// (db:query conn "SELECT * FROM users WHERE id = ?" '(1))
/// => ({:id 1 :name "Alice"})
/// ```
///
/// # See Also
///
/// db:exec, db:open, db:close
pub fn db_query(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(EvalError::ArityMismatch);
    }

    // Extract connection handle
    let conn_map = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    let handle_val = conn_map
        .get("handle")
        .ok_or_else(|| EvalError::IoError("Connection map missing :handle key".to_string()))?;

    let handle = match handle_val {
        Value::Number(n) => *n as u64,
        _ => return Err(EvalError::IoError("Handle must be a number".to_string())),
    };

    // Extract SQL
    let sql = match &args[1] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    // Extract optional parameters
    let params: Vec<Value> = if args.len() == 3 {
        match &args[2] {
            Value::List(items) => items.clone(),
            Value::Nil => Vec::new(),
            _ => return Err(EvalError::TypeError),
        }
    } else {
        Vec::new()
    };

    // Execute with connection
    with_connection(handle, |conn| {
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| EvalError::IoError(format!("Failed to prepare statement: {}", e)))?;

        // Convert Value parameters to rusqlite parameters
        let rusqlite_params: Vec<rusqlite::types::Value> = params
            .iter()
            .map(value_to_rusqlite)
            .collect::<Result<Vec<_>, _>>()?;

        // Get column names
        let column_names: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Execute query and collect rows
        let rows = stmt
            .query_map(params_from_iter(rusqlite_params.iter()), |row| {
                let mut row_map = HashMap::new();

                for (i, col_name) in column_names.iter().enumerate() {
                    let value = rusqlite_to_value(row, i)?;
                    row_map.insert(col_name.clone(), value);
                }

                Ok(Value::Map(row_map))
            })
            .map_err(|e| EvalError::IoError(format!("Failed to execute query: {}", e)))?;

        let result: Vec<Value> = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| EvalError::IoError(format!("Failed to read query results: {}", e)))?;

        Ok(Value::List(result))
    })
}

// ============================================================================
// Type Conversion Helpers
// ============================================================================

/// Convert Lisp Value to rusqlite Value
fn value_to_rusqlite(val: &Value) -> Result<rusqlite::types::Value, EvalError> {
    match val {
        Value::Nil => Ok(rusqlite::types::Value::Null),
        Value::Number(n) => Ok(rusqlite::types::Value::Real(*n)),
        Value::String(s) => Ok(rusqlite::types::Value::Text(s.clone())),
        Value::Bool(true) => Ok(rusqlite::types::Value::Integer(1)),
        Value::Bool(false) => Ok(rusqlite::types::Value::Integer(0)),
        _ => Err(EvalError::TypeError),
    }
}

/// Convert rusqlite row column to Lisp Value
fn rusqlite_to_value(row: &rusqlite::Row, idx: usize) -> Result<Value, rusqlite::Error> {
    use rusqlite::types::ValueRef;

    match row.get_ref(idx)? {
        ValueRef::Null => Ok(Value::Nil),
        ValueRef::Integer(i) => Ok(Value::Number(i as f64)),
        ValueRef::Real(r) => Ok(Value::Number(r)),
        ValueRef::Text(t) => {
            let s = std::str::from_utf8(t).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    idx,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok(Value::String(s.to_string()))
        }
        ValueRef::Blob(b) => {
            // Convert blob to string (assuming UTF-8)
            let s = std::str::from_utf8(b).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    idx,
                    rusqlite::types::Type::Blob,
                    Box::new(e),
                )
            })?;
            Ok(Value::String(s.to_string()))
        }
    }
}

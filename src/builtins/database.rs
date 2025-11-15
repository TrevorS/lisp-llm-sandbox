//! Database operations: db-execute, db-query
//!
//! Functions for SQLite database operations with capability-based sandboxing.
//!
//! - `db-execute`: Execute SQL statements (CREATE, INSERT, UPDATE, DELETE)
//!   Returns number of rows affected
//! - `db-query`: Execute SELECT queries
//!   Returns list of maps (each row as a map with column names as keys)
//!
//! All database files are restricted to whitelisted paths via capability-based sandboxing

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;
use std::collections::HashMap;

use super::SANDBOX;

#[builtin(name = "db-execute", category = "Database", related(db-query))]
/// Executes a SQL statement that modifies the database (CREATE, INSERT, UPDATE, DELETE).
///
/// Database file path is relative to allowed sandbox directories.
/// Supports parameterized queries to prevent SQL injection.
///
/// # Examples
///
/// ```lisp
/// (db-execute "data.db" "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
/// (db-execute "data.db" "INSERT INTO users (id, name) VALUES (?, ?)" '(1 "Alice"))
/// (db-execute "data.db" "UPDATE users SET name = ? WHERE id = ?" '("Bob" 1))
/// ```
///
/// # See Also
///
/// db-query
pub fn db_execute(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(EvalError::ArityMismatch);
    }

    let db_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let sql = match &args[1] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    // Optional parameters list
    let params = if args.len() == 3 {
        match &args[2] {
            Value::List(items) => Some(items.clone()),
            _ => return Err(EvalError::TypeError),
        }
    } else {
        None
    };

    SANDBOX.with(|s| {
        let sandbox_ref = s.borrow();
        let sandbox = sandbox_ref
            .as_ref()
            .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

        sandbox
            .db_execute(db_path, sql, params.as_deref())
            .map(|rows_affected| Value::Number(rows_affected as f64))
            .map_err(|e| EvalError::IoError(e.to_string()))
    })
}

#[builtin(name = "db-query", category = "Database", related(db-execute))]
/// Executes a SELECT query and returns the result set.
///
/// Returns a list of maps, where each map represents a row with column names as keys.
/// Database file path is relative to allowed sandbox directories.
/// Supports parameterized queries to prevent SQL injection.
///
/// # Examples
///
/// ```lisp
/// (db-query "data.db" "SELECT * FROM users")
/// => ({:id 1 :name "Alice"} {:id 2 :name "Bob"})
///
/// (db-query "data.db" "SELECT name FROM users WHERE id = ?" '(1))
/// => ({:name "Alice"})
/// ```
///
/// # See Also
///
/// db-execute
pub fn db_query(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(EvalError::ArityMismatch);
    }

    let db_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let sql = match &args[1] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    // Optional parameters list
    let params = if args.len() == 3 {
        match &args[2] {
            Value::List(items) => Some(items.clone()),
            _ => return Err(EvalError::TypeError),
        }
    } else {
        None
    };

    SANDBOX.with(|s| {
        let sandbox_ref = s.borrow();
        let sandbox = sandbox_ref
            .as_ref()
            .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

        sandbox
            .db_query(db_path, sql, params.as_deref())
            .map(|rows| {
                // Convert Vec<HashMap<String, DbValue>> to Value::List of Value::Map
                Value::List(
                    rows.into_iter()
                        .map(|row| {
                            let value_map: HashMap<String, Value> = row
                                .into_iter()
                                .map(|(k, v)| {
                                    let val = match v {
                                        crate::sandbox::DbValue::Integer(i) => {
                                            Value::Number(i as f64)
                                        }
                                        crate::sandbox::DbValue::Real(r) => Value::Number(r),
                                        crate::sandbox::DbValue::Text(t) => Value::String(t),
                                        crate::sandbox::DbValue::Null => Value::Nil,
                                    };
                                    (k, val)
                                })
                                .collect();
                            Value::Map(value_map)
                        })
                        .collect(),
                )
            })
            .map_err(|e| EvalError::IoError(e.to_string()))
    })
}

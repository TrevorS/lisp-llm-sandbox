// ABOUTME: Comprehensive database module tests covering security, functionality, and edge cases

use lisp_llm_sandbox::*;
use serial_test::serial;
use std::path::PathBuf;
use std::rc::Rc;

/// Setup helper for database tests with sandbox initialization
fn setup_with_sandbox() -> (Rc<env::Environment>, macros::MacroRegistry) {
    // Initialize sandbox
    let fs_config = config::FsConfig {
        allowed_paths: vec![PathBuf::from("./data")],
        max_file_size: 10485760,
    };
    let net_config = config::NetConfig {
        enabled: false,
        allowed_addresses: vec![],
    };
    let sandbox = sandbox::Sandbox::new(fs_config, net_config).unwrap();
    builtins::set_sandbox_storage(sandbox);

    // Setup environment
    let env = env::Environment::new();
    let mut macro_reg = macros::MacroRegistry::new();
    builtins::register_builtins(env.clone());

    // Load stdlib modules
    let modules = [
        include_str!("../src/stdlib/lisp/core.lisp"),
        include_str!("../src/stdlib/lisp/math.lisp"),
        include_str!("../src/stdlib/lisp/string.lisp"),
        include_str!("../src/stdlib/lisp/test.lisp"),
        include_str!("../src/stdlib/lisp/http.lisp"),
        include_str!("../src/stdlib/lisp/db.lisp"),
    ];

    for module in &modules {
        load_stdlib_code(module, env.clone(), &mut macro_reg)
            .expect("Failed to load stdlib module");
    }

    (env, macro_reg)
}

fn load_stdlib_code(
    code: &str,
    env: Rc<env::Environment>,
    macro_reg: &mut macros::MacroRegistry,
) -> Result<(), String> {
    let mut remaining = code.trim();
    while !remaining.is_empty() {
        match parser::parse_one(remaining) {
            Ok((expr, rest)) => {
                eval::eval_with_macros(expr, env.clone(), macro_reg)
                    .map_err(|e| format!("Eval error: {:?}", e))?;
                remaining = rest.trim();
            }
            Err(e) => return Err(format!("Parse error: {}", e)),
        }
    }
    Ok(())
}

fn eval_code(
    code: &str,
    env: Rc<env::Environment>,
    macro_reg: &mut macros::MacroRegistry,
) -> Result<value::Value, String> {
    let expr = parser::parse(code).map_err(|e| format!("Parse error: {}", e))?;
    eval::eval_with_macros(expr, env, macro_reg).map_err(|e| format!("Eval error: {:?}", e))
}

// ============================================================================
// Connection Lifecycle Tests
// ============================================================================

#[test]
#[serial]
fn test_connection_open_and_close() {
    let (env, mut macro_reg) = setup_with_sandbox();

    // Open connection
    let code = r#"(db:connect (sqlite:spec "test_lifecycle.db"))"#;
    let result = eval_code(code, env.clone(), &mut macro_reg).unwrap();

    // Verify it's a map with backend, path, and handle
    match result {
        value::Value::Map(m) => {
            assert!(m.contains_key("backend"));
            assert!(m.contains_key("path"));
            assert!(m.contains_key("handle"));
        }
        _ => panic!("Expected Map, got {:?}", result),
    }

    // Close connection
    eval_code("(define conn (db:connect (sqlite:spec \"test_lifecycle.db\")))", env.clone(), &mut macro_reg).unwrap();
    let result = eval_code("(db:close conn)", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Bool(true)));
}

#[test]
#[serial]
fn test_double_close_error() {
    let (env, mut macro_reg) = setup_with_sandbox();

    eval_code("(define conn (db:connect (sqlite:spec \"test_double_close.db\")))", env.clone(), &mut macro_reg).unwrap();
    eval_code("(db:close conn)", env.clone(), &mut macro_reg).unwrap();

    // Second close should error
    let result = eval_code("(db:close conn)", env.clone(), &mut macro_reg);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid connection handle"));
}

// ============================================================================
// Security Tests
// ============================================================================

#[test]
#[serial]
fn test_path_traversal_blocked() {
    let (env, mut macro_reg) = setup_with_sandbox();

    // Test ../
    let result = eval_code(r#"(db:connect (sqlite:spec "../etc/passwd.db"))"#, env.clone(), &mut macro_reg);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Path traversal not allowed"));
}

#[test]
#[serial]
fn test_absolute_path_blocked() {
    let (env, mut macro_reg) = setup_with_sandbox();

    // Unix absolute path
    let result = eval_code(r#"(db:connect (sqlite:spec "/etc/passwd.db"))"#, env.clone(), &mut macro_reg);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Absolute paths not allowed"));
}

#[test]
#[serial]
fn test_sql_injection_protection() {
    let (env, mut macro_reg) = setup_with_sandbox();

    // Create table and insert malicious data
    eval_code("(define conn (db:connect (sqlite:spec \"test_injection.db\")))", env.clone(), &mut macro_reg).unwrap();
    // Drop table if it exists to ensure clean state
    let _ = eval_code(r#"(db:execute conn "DROP TABLE IF EXISTS users" '())"#, env.clone(), &mut macro_reg);
    eval_code(r#"(db:execute conn "CREATE TABLE users (id INTEGER, name TEXT)" '())"#, env.clone(), &mut macro_reg).unwrap();

    // Try SQL injection - should be treated as literal string
    let malicious = r#"(db:execute conn "INSERT INTO users VALUES (?, ?)" '(1 "Alice'; DROP TABLE users; --"))"#;
    eval_code(malicious, env.clone(), &mut macro_reg).unwrap();

    // Table should still exist and have the data
    let result = eval_code(r#"(db:query conn "SELECT * FROM users WHERE id = ?" '(1))"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(rows) => {
            assert_eq!(rows.len(), 1);
            // Verify the malicious string is stored as data
            if let value::Value::Map(row) = &rows[0] {
                if let Some(value::Value::String(name)) = row.get("name") {
                    assert!(name.contains("DROP TABLE"));
                }
            }
        }
        _ => panic!("Expected List of rows"),
    }

    eval_code("(db:close conn)", env.clone(), &mut macro_reg).unwrap();
}

// ============================================================================
// Database Operations Tests
// ============================================================================

#[test]
#[serial]
fn test_create_table_and_insert() {
    let (env, mut macro_reg) = setup_with_sandbox();

    eval_code("(define conn (db:connect (sqlite:spec \"test_create.db\")))", env.clone(), &mut macro_reg).unwrap();

    // Create table
    let result = eval_code(
        r#"(db:execute conn "CREATE TABLE IF NOT EXISTS products (id INTEGER, name TEXT, price REAL)" '())"#,
        env.clone(),
        &mut macro_reg,
    ).unwrap();
    assert!(matches!(result, value::Value::Number(_)));

    // Insert data
    let result = eval_code(
        r#"(db:execute conn "INSERT INTO products VALUES (?, ?, ?)" '(1 "Widget" 9.99))"#,
        env.clone(),
        &mut macro_reg,
    ).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 1.0), // 1 row affected
        _ => panic!("Expected Number"),
    }

    eval_code("(db:close conn)", env.clone(), &mut macro_reg).unwrap();
}

#[test]
#[serial]
fn test_query_results() {
    let (env, mut macro_reg) = setup_with_sandbox();

    eval_code("(define conn (db:connect (sqlite:spec \"test_query.db\")))", env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT)" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "DELETE FROM users" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "INSERT INTO users VALUES (?, ?)" '(1 "Alice"))"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "INSERT INTO users VALUES (?, ?)" '(2 "Bob"))"#, env.clone(), &mut macro_reg).unwrap();

    // Query all
    let result = eval_code(r#"(db:query conn "SELECT * FROM users" '())"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(rows) => {
            assert_eq!(rows.len(), 2);
            // Verify rows are maps
            for row in rows {
                assert!(matches!(row, value::Value::Map(_)));
            }
        }
        _ => panic!("Expected List"),
    }

    eval_code("(db:close conn)", env.clone(), &mut macro_reg).unwrap();
}

// ============================================================================
// Query Builder Tests
// ============================================================================

#[test]
#[serial]
fn test_db_insert() {
    let (env, mut macro_reg) = setup_with_sandbox();

    eval_code("(define conn (db:connect (sqlite:spec \"test_insert_builder.db\")))", env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT, age INTEGER)" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "DELETE FROM users" '())"#, env.clone(), &mut macro_reg).unwrap();

    // Insert using map syntax
    let result = eval_code(r#"(db:insert conn "users" {:id 1 :name "Alice" :age 30})"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 1.0),
        _ => panic!("Expected Number(1)"),
    }

    // Verify insertion
    let result = eval_code(r#"(db:query conn "SELECT * FROM users WHERE id = ?" '(1))"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(rows) => assert_eq!(rows.len(), 1),
        _ => panic!("Expected List"),
    }

    eval_code("(db:close conn)", env.clone(), &mut macro_reg).unwrap();
}

#[test]
#[serial]
fn test_db_update() {
    let (env, mut macro_reg) = setup_with_sandbox();

    eval_code("(define conn (db:connect (sqlite:spec \"test_update_builder.db\")))", env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT, age INTEGER)" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "DELETE FROM users" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:insert conn "users" {:id 1 :name "Alice" :age 30})"#, env.clone(), &mut macro_reg).unwrap();

    // Update age
    let result = eval_code(r#"(db:update conn "users" {:age 31} {:id 1})"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 1.0),
        _ => panic!("Expected Number(1)"),
    }

    eval_code("(db:close conn)", env.clone(), &mut macro_reg).unwrap();
}

#[test]
#[serial]
fn test_db_delete() {
    let (env, mut macro_reg) = setup_with_sandbox();

    eval_code("(define conn (db:connect (sqlite:spec \"test_delete_builder.db\")))", env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT)" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "DELETE FROM users" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:insert conn "users" {:id 1 :name "Alice"})"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:insert conn "users" {:id 2 :name "Bob"})"#, env.clone(), &mut macro_reg).unwrap();

    // Delete one row
    let result = eval_code(r#"(db:delete conn "users" {:id 1})"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 1.0),
        _ => panic!("Expected Number(1)"),
    }

    // Verify only Bob remains
    let result = eval_code(r#"(db:query conn "SELECT * FROM users" '())"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(rows) => assert_eq!(rows.len(), 1),
        _ => panic!("Expected List"),
    }

    eval_code("(db:close conn)", env.clone(), &mut macro_reg).unwrap();
}

#[test]
#[serial]
fn test_db_find() {
    let (env, mut macro_reg) = setup_with_sandbox();

    eval_code("(define conn (db:connect (sqlite:spec \"test_find_builder.db\")))", env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT, age INTEGER)" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "DELETE FROM users" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:insert conn "users" {:id 1 :name "Alice" :age 30})"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:insert conn "users" {:id 2 :name "Bob" :age 25})"#, env.clone(), &mut macro_reg).unwrap();

    // Find with WHERE
    let result = eval_code(r#"(db:find conn "users" "*" {:age 30})"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(rows) => assert_eq!(rows.len(), 1),
        _ => panic!("Expected List"),
    }

    // Find all (empty where-map)
    let result = eval_code(r#"(db:find conn "users" '("id" "name") {})"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(rows) => assert_eq!(rows.len(), 2),
        _ => panic!("Expected List"),
    }

    eval_code("(db:close conn)", env.clone(), &mut macro_reg).unwrap();
}

#[test]
#[serial]
fn test_db_count() {
    let (env, mut macro_reg) = setup_with_sandbox();

    eval_code("(define conn (db:connect (sqlite:spec \"test_count_builder.db\")))", env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "CREATE TABLE IF NOT EXISTS users (id INTEGER, age INTEGER)" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "DELETE FROM users" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:insert conn "users" {:id 1 :age 30})"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:insert conn "users" {:id 2 :age 25})"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:insert conn "users" {:id 3 :age 30})"#, env.clone(), &mut macro_reg).unwrap();

    // Count all
    let result = eval_code(r#"(db:count conn "users" {})"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 3.0),
        _ => panic!("Expected Number(3)"),
    }

    // Count with WHERE
    let result = eval_code(r#"(db:count conn "users" {:age 30})"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 2.0),
        _ => panic!("Expected Number(2)"),
    }

    eval_code("(db:close conn)", env.clone(), &mut macro_reg).unwrap();
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
#[serial]
fn test_nil_and_bool_types() {
    let (env, mut macro_reg) = setup_with_sandbox();

    eval_code("(define conn (db:connect (sqlite:spec \"test_types.db\")))", env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "CREATE TABLE IF NOT EXISTS types (id INTEGER, value INTEGER)" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "DELETE FROM types" '())"#, env.clone(), &mut macro_reg).unwrap();

    // Insert nil (becomes NULL) - use list construction instead of quote to avoid symbol
    eval_code(r#"(db:execute conn "INSERT INTO types VALUES (?, ?)" (list 1 nil))"#, env.clone(), &mut macro_reg).unwrap();

    // Insert booleans (become 1/0)
    eval_code(r#"(db:execute conn "INSERT INTO types VALUES (?, ?)" (list 2 #t))"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "INSERT INTO types VALUES (?, ?)" (list 3 #f))"#, env.clone(), &mut macro_reg).unwrap();

    // Query and verify types are converted
    let result = eval_code(r#"(db:query conn "SELECT * FROM types ORDER BY id" '())"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(rows) => assert_eq!(rows.len(), 3),
        _ => panic!("Expected List"),
    }

    eval_code("(db:close conn)", env.clone(), &mut macro_reg).unwrap();
}

#[test]
#[serial]
fn test_empty_result_set() {
    let (env, mut macro_reg) = setup_with_sandbox();

    eval_code("(define conn (db:connect (sqlite:spec \"test_empty.db\")))", env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "CREATE TABLE IF NOT EXISTS empty (id INTEGER)" '())"#, env.clone(), &mut macro_reg).unwrap();
    eval_code(r#"(db:execute conn "DELETE FROM empty" '())"#, env.clone(), &mut macro_reg).unwrap();

    // Query empty table
    let result = eval_code(r#"(db:query conn "SELECT * FROM empty" '())"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(rows) => assert_eq!(rows.len(), 0),
        _ => panic!("Expected empty List"),
    }

    // db:count on empty table
    let result = eval_code(r#"(db:count conn "empty" {})"#, env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 0.0),
        _ => panic!("Expected Number(0)"),
    }

    eval_code("(db:close conn)", env.clone(), &mut macro_reg).unwrap();
}

use lisp_llm_sandbox::*;
use std::fs;
use std::path::PathBuf;

fn setup_test_env() -> (sandbox::Sandbox, PathBuf) {
    let test_dir = PathBuf::from("./test_db_temp");
    // Clean up from previous run
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let fs_config = config::FsConfig {
        allowed_paths: vec![test_dir.clone()],
        ..Default::default()
    };

    let net_config = config::NetConfig::default();
    let sandbox = sandbox::Sandbox::new(fs_config, net_config).unwrap();

    (sandbox, test_dir)
}

fn cleanup_test_env(test_dir: &PathBuf) {
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
#[serial_test::serial]
fn test_db_create_table_and_insert() {
    let (sandbox, test_dir) = setup_test_env();

    // Create table
    let result = sandbox.db_execute(
        "test.db",
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)",
        None,
    );
    assert!(result.is_ok());

    // Insert data
    let result = sandbox.db_execute(
        "test.db",
        "INSERT INTO users (id, name) VALUES (1, 'Alice')",
        None,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // 1 row affected

    cleanup_test_env(&test_dir);
}

#[test]
#[serial_test::serial]
fn test_db_query_basic() {
    let (sandbox, test_dir) = setup_test_env();

    // Create and populate table
    sandbox
        .db_execute(
            "test.db",
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)",
            None,
        )
        .unwrap();

    sandbox
        .db_execute(
            "test.db",
            "INSERT INTO users (id, name) VALUES (1, 'Alice')",
            None,
        )
        .unwrap();

    sandbox
        .db_execute(
            "test.db",
            "INSERT INTO users (id, name) VALUES (2, 'Bob')",
            None,
        )
        .unwrap();

    // Query data
    let result = sandbox.db_query("test.db", "SELECT * FROM users ORDER BY id", None);
    assert!(result.is_ok());

    let rows = result.unwrap();
    assert_eq!(rows.len(), 2);

    // Check first row
    assert_eq!(rows[0].get("id").unwrap(), &sandbox::DbValue::Integer(1));
    assert_eq!(
        rows[0].get("name").unwrap(),
        &sandbox::DbValue::Text("Alice".to_string())
    );

    // Check second row
    assert_eq!(rows[1].get("id").unwrap(), &sandbox::DbValue::Integer(2));
    assert_eq!(
        rows[1].get("name").unwrap(),
        &sandbox::DbValue::Text("Bob".to_string())
    );

    cleanup_test_env(&test_dir);
}

#[test]
#[serial_test::serial]
fn test_db_parameterized_query() {
    let (sandbox, test_dir) = setup_test_env();

    // Create and populate table
    sandbox
        .db_execute(
            "test.db",
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)",
            None,
        )
        .unwrap();

    // Insert with parameters
    let params = vec![
        value::Value::Number(1.0),
        value::Value::String("Alice".to_string()),
        value::Value::Number(30.0),
    ];

    let result = sandbox.db_execute(
        "test.db",
        "INSERT INTO users (id, name, age) VALUES (?, ?, ?)",
        Some(&params),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);

    // Query with parameters
    let query_params = vec![value::Value::String("Alice".to_string())];
    let result = sandbox.db_query(
        "test.db",
        "SELECT * FROM users WHERE name = ?",
        Some(&query_params),
    );
    assert!(result.is_ok());

    let rows = result.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].get("name").unwrap(),
        &sandbox::DbValue::Text("Alice".to_string())
    );
    assert_eq!(rows[0].get("age").unwrap(), &sandbox::DbValue::Integer(30));

    cleanup_test_env(&test_dir);
}

#[test]
#[serial_test::serial]
fn test_db_update_and_delete() {
    let (sandbox, test_dir) = setup_test_env();

    // Create and populate table
    sandbox
        .db_execute(
            "test.db",
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)",
            None,
        )
        .unwrap();

    sandbox
        .db_execute(
            "test.db",
            "INSERT INTO users (id, name) VALUES (1, 'Alice')",
            None,
        )
        .unwrap();

    // Update
    let update_params = vec![
        value::Value::String("Bob".to_string()),
        value::Value::Number(1.0),
    ];
    let result = sandbox.db_execute(
        "test.db",
        "UPDATE users SET name = ? WHERE id = ?",
        Some(&update_params),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // 1 row affected

    // Verify update
    let rows = sandbox
        .db_query("test.db", "SELECT name FROM users WHERE id = 1", None)
        .unwrap();
    assert_eq!(
        rows[0].get("name").unwrap(),
        &sandbox::DbValue::Text("Bob".to_string())
    );

    // Delete
    let result = sandbox.db_execute("test.db", "DELETE FROM users WHERE id = 1", None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // 1 row deleted

    // Verify deletion
    let rows = sandbox
        .db_query("test.db", "SELECT * FROM users", None)
        .unwrap();
    assert_eq!(rows.len(), 0);

    cleanup_test_env(&test_dir);
}

#[test]
#[serial_test::serial]
fn test_db_null_values() {
    let (sandbox, test_dir) = setup_test_env();

    // Create table with nullable column
    sandbox
        .db_execute(
            "test.db",
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)",
            None,
        )
        .unwrap();

    // Insert with NULL using Nil value
    let params = vec![
        value::Value::Number(1.0),
        value::Value::String("Alice".to_string()),
        value::Value::Nil,
    ];
    sandbox
        .db_execute(
            "test.db",
            "INSERT INTO users (id, name, email) VALUES (?, ?, ?)",
            Some(&params),
        )
        .unwrap();

    // Query and verify NULL
    let rows = sandbox
        .db_query("test.db", "SELECT * FROM users", None)
        .unwrap();
    assert_eq!(rows[0].get("email").unwrap(), &sandbox::DbValue::Null);

    cleanup_test_env(&test_dir);
}

#[test]
#[serial_test::serial]
fn test_db_real_numbers() {
    let (sandbox, test_dir) = setup_test_env();

    // Create table with REAL column
    sandbox
        .db_execute(
            "test.db",
            "CREATE TABLE measurements (id INTEGER PRIMARY KEY, value REAL)",
            None,
        )
        .unwrap();

    // Insert real number
    let test_value = 42.5;
    let params = vec![value::Value::Number(1.0), value::Value::Number(test_value)];
    sandbox
        .db_execute(
            "test.db",
            "INSERT INTO measurements (id, value) VALUES (?, ?)",
            Some(&params),
        )
        .unwrap();

    // Query and verify
    let rows = sandbox
        .db_query("test.db", "SELECT * FROM measurements", None)
        .unwrap();

    match rows[0].get("value").unwrap() {
        sandbox::DbValue::Real(v) => assert!((v - test_value).abs() < 0.00001),
        _ => panic!("Expected Real value"),
    }

    cleanup_test_env(&test_dir);
}

#[test]
#[serial_test::serial]
fn test_db_path_traversal_protection() {
    let (sandbox, _test_dir) = setup_test_env();

    // Attempt path traversal
    let result = sandbox.db_execute(
        "../../../etc/passwd.db",
        "CREATE TABLE hack (id INTEGER)",
        None,
    );
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(sandbox::SandboxError::PathNotAllowed(_))
    ));
}

#[test]
#[serial_test::serial]
fn test_db_absolute_path_protection() {
    let (sandbox, _test_dir) = setup_test_env();

    // Attempt absolute path
    let result = sandbox.db_execute("/tmp/test.db", "CREATE TABLE hack (id INTEGER)", None);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(sandbox::SandboxError::PathNotAllowed(_))
    ));
}

#[test]
#[serial_test::serial]
fn test_db_sql_error_handling() {
    let (sandbox, test_dir) = setup_test_env();

    // Invalid SQL
    let result = sandbox.db_execute("test.db", "INVALID SQL STATEMENT", None);
    assert!(result.is_err());

    cleanup_test_env(&test_dir);
}

#[test]
#[serial_test::serial]
fn test_db_empty_result_set() {
    let (sandbox, test_dir) = setup_test_env();

    // Create empty table
    sandbox
        .db_execute(
            "test.db",
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)",
            None,
        )
        .unwrap();

    // Query empty table
    let rows = sandbox
        .db_query("test.db", "SELECT * FROM users", None)
        .unwrap();
    assert_eq!(rows.len(), 0);

    cleanup_test_env(&test_dir);
}

#[test]
#[serial_test::serial]
fn test_db_boolean_values() {
    let (sandbox, test_dir) = setup_test_env();

    // Create table
    sandbox
        .db_execute(
            "test.db",
            "CREATE TABLE settings (id INTEGER PRIMARY KEY, enabled INTEGER)",
            None,
        )
        .unwrap();

    // Insert boolean (stored as integer in SQLite)
    let params = vec![value::Value::Number(1.0), value::Value::Bool(true)];
    sandbox
        .db_execute(
            "test.db",
            "INSERT INTO settings (id, enabled) VALUES (?, ?)",
            Some(&params),
        )
        .unwrap();

    // Query and verify (will be Integer in SQLite)
    let rows = sandbox
        .db_query("test.db", "SELECT * FROM settings", None)
        .unwrap();
    assert_eq!(
        rows[0].get("enabled").unwrap(),
        &sandbox::DbValue::Integer(1)
    );

    cleanup_test_env(&test_dir);
}

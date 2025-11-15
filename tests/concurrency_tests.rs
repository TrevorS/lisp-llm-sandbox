// ABOUTME: Tests for concurrency primitives (channels)

use lisp_llm_sandbox::*;
use std::rc::Rc;

fn setup() -> Rc<env::Environment> {
    let env = env::Environment::new();
    builtins::register_builtins(env.clone());
    env
}

fn eval_expr(code: &str, env: Rc<env::Environment>) -> Result<value::Value, error::EvalError> {
    let expr = parser::parse(code).map_err(|e| error::EvalError::Custom(e))?;
    let mut macro_reg = macros::MacroRegistry::new();
    eval::eval_with_macros(expr, env, &mut macro_reg)
}

#[test]
fn test_make_channel_unbuffered() {
    let env = setup();
    let result = eval_expr("(make-channel)", env);
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), value::Value::Channel { .. }));
}

#[test]
fn test_make_channel_buffered() {
    let env = setup();
    let result = eval_expr("(make-channel 10)", env.clone());
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), value::Value::Channel { .. }));

    // Test with zero capacity
    let result = eval_expr("(make-channel 0)", env.clone());
    assert!(result.is_ok());

    // Test with negative capacity (should error)
    let result = eval_expr("(make-channel -1)", env);
    assert!(result.is_err());
}

#[test]
fn test_channel_type_predicate() {
    let env = setup();

    // Test with channel
    let code = "(channel? (make-channel))";
    let result = eval_expr(code, env.clone()).unwrap();
    assert!(matches!(result, value::Value::Bool(true)));

    // Test with non-channel
    let code = "(channel? 42)";
    let result = eval_expr(code, env.clone()).unwrap();
    assert!(matches!(result, value::Value::Bool(false)));

    let code = "(channel? \"hello\")";
    let result = eval_expr(code, env.clone()).unwrap();
    assert!(matches!(result, value::Value::Bool(false)));

    let code = "(channel? (list 1 2 3))";
    let result = eval_expr(code, env).unwrap();
    assert!(matches!(result, value::Value::Bool(false)));
}

#[test]
fn test_channel_send_recv_simple() {
    let env = setup();

    // Test sending and receiving a number
    let code = r#"
        (begin
            (define ch (make-channel))
            (channel-send ch 42)
            (channel-recv ch))
    "#;
    let result = eval_expr(code, env.clone()).unwrap();
    assert!(matches!(result, value::Value::Number(n) if n == 42.0));

    // Test sending and receiving a string
    let code = r#"
        (begin
            (define ch (make-channel))
            (channel-send ch "hello")
            (channel-recv ch))
    "#;
    let result = eval_expr(code, env.clone()).unwrap();
    assert!(matches!(result, value::Value::String(s) if s == "hello"));

    // Test sending and receiving a list
    let code = r#"
        (begin
            (define ch (make-channel))
            (channel-send ch (list 1 2 3))
            (channel-recv ch))
    "#;
    let result = eval_expr(code, env).unwrap();
    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], value::Value::Number(n) if n == 1.0));
            assert!(matches!(items[1], value::Value::Number(n) if n == 2.0));
            assert!(matches!(items[2], value::Value::Number(n) if n == 3.0));
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_channel_multiple_send_recv() {
    let env = setup();

    let code = r#"
        (begin
            (define ch (make-channel 5))
            (channel-send ch 1)
            (channel-send ch 2)
            (channel-send ch 3)
            (define a (channel-recv ch))
            (define b (channel-recv ch))
            (define c (channel-recv ch))
            (list a b c))
    "#;
    let result = eval_expr(code, env).unwrap();
    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], value::Value::Number(n) if n == 1.0));
            assert!(matches!(items[1], value::Value::Number(n) if n == 2.0));
            assert!(matches!(items[2], value::Value::Number(n) if n == 3.0));
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_channel_buffered_capacity() {
    let env = setup();

    // Test that buffered channel can hold multiple values
    let code = r#"
        (begin
            (define ch (make-channel 3))
            (channel-send ch "a")
            (channel-send ch "b")
            (channel-send ch "c")
            (list
                (channel-recv ch)
                (channel-recv ch)
                (channel-recv ch)))
    "#;
    let result = eval_expr(code, env).unwrap();
    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], value::Value::String(s) if s == "a"));
            assert!(matches!(&items[1], value::Value::String(s) if s == "b"));
            assert!(matches!(&items[2], value::Value::String(s) if s == "c"));
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_channel_send_returns_value() {
    let env = setup();

    let code = r#"
        (begin
            (define ch (make-channel))
            (define sent (channel-send ch 42))
            sent)
    "#;
    let result = eval_expr(code, env).unwrap();
    assert!(matches!(result, value::Value::Number(n) if n == 42.0));
}

#[test]
fn test_channel_with_map_values() {
    let env = setup();

    let code = r#"
        (begin
            (define ch (make-channel))
            (channel-send ch {:name "Alice" :age 30})
            (channel-recv ch))
    "#;
    let result = eval_expr(code, env).unwrap();
    match result {
        value::Value::Map(map) => {
            match map.get("name") {
                Some(value::Value::String(s)) if s == "Alice" => {},
                _ => panic!("Expected name to be Alice"),
            }
            match map.get("age") {
                Some(value::Value::Number(n)) if *n == 30.0 => {},
                _ => panic!("Expected age to be 30"),
            }
        }
        _ => panic!("Expected map value"),
    }
}

#[test]
fn test_channel_close() {
    let env = setup();

    // Test that close returns nil
    let code = r#"
        (begin
            (define ch (make-channel))
            (channel-close ch))
    "#;
    let result = eval_expr(code, env).unwrap();
    assert!(matches!(result, value::Value::Nil));
}

#[test]
fn test_channel_errors() {
    let env = setup();

    // Test send without channel
    let result = eval_expr("(channel-send 42 100)", env.clone());
    assert!(result.is_err());

    // Test recv without channel
    let result = eval_expr("(channel-recv 42)", env.clone());
    assert!(result.is_err());

    // Test close without channel
    let result = eval_expr("(channel-close 42)", env.clone());
    assert!(result.is_err());

    // Test make-channel with invalid capacity
    let result = eval_expr("(make-channel \"invalid\")", env);
    assert!(result.is_err());
}

#[test]
fn test_channel_wrong_argument_count() {
    let env = setup();

    // channel-send needs 2 arguments
    let result = eval_expr("(channel-send (make-channel))", env.clone());
    assert!(result.is_err());

    // channel-recv needs 1 argument
    let result = eval_expr(r#"
        (begin
            (define ch (make-channel))
            (channel-recv ch ch))
    "#, env.clone());
    assert!(result.is_err());

    // channel-close needs 1 argument
    let result = eval_expr("(channel-close)", env.clone());
    assert!(result.is_err());

    // channel? needs 1 argument
    let result = eval_expr("(channel?)", env);
    assert!(result.is_err());
}

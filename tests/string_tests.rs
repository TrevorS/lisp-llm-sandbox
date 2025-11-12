// Tests for string manipulation and testing assertion functions

use lisp_llm_sandbox::env::Environment;
use lisp_llm_sandbox::error::EvalError;
use lisp_llm_sandbox::eval::eval;
use lisp_llm_sandbox::parser::parse;
use lisp_llm_sandbox::value::Value;
use std::rc::Rc;

/// Helper to parse and evaluate an expression
fn eval_expr(expr: &str, env: &Rc<Environment>) -> Result<Value, EvalError> {
    let parsed = parse(expr).map_err(|e| EvalError::Custom(e.to_string()))?;
    eval(parsed, env.clone())
}

/// Helper to get a test environment with builtins
fn test_env() -> Rc<Environment> {
    let env = Environment::new();
    lisp_llm_sandbox::builtins::register_builtins(env.clone());
    env
}

// ============================================================================
// Testing Assertions Tests
// ============================================================================

#[test]
fn test_assert_true() {
    let env = test_env();
    let result = eval_expr("(assert #t)", &env).unwrap();
    assert!(matches!(result, Value::Bool(true)));
}

#[test]
fn test_assert_false() {
    let env = test_env();
    let result = eval_expr("(assert #f)", &env).unwrap();
    assert!(matches!(result, Value::Error(_)));
}

#[test]
fn test_assert_with_message() {
    let env = test_env();
    let result = eval_expr("(assert #f \"custom message\")", &env).unwrap();
    match result {
        Value::Error(msg) => assert_eq!(msg, "custom message"),
        _ => panic!("Expected error value"),
    }
}

#[test]
fn test_assert_equal_numbers() {
    let env = test_env();
    let result = eval_expr("(assert-equal 5 5)", &env).unwrap();
    assert!(matches!(result, Value::Bool(true)));

    let result = eval_expr("(assert-equal 5 6)", &env).unwrap();
    assert!(matches!(result, Value::Error(_)));
}

#[test]
fn test_assert_equal_strings() {
    let env = test_env();
    let result = eval_expr("(assert-equal \"hello\" \"hello\")", &env).unwrap();
    assert!(matches!(result, Value::Bool(true)));

    let result = eval_expr("(assert-equal \"hello\" \"world\")", &env).unwrap();
    assert!(matches!(result, Value::Error(_)));
}

#[test]
fn test_assert_equal_lists() {
    let env = test_env();
    let result = eval_expr("(assert-equal '(1 2 3) '(1 2 3))", &env).unwrap();
    assert!(matches!(result, Value::Bool(true)));

    let result = eval_expr("(assert-equal '(1 2 3) '(1 2 4))", &env).unwrap();
    assert!(matches!(result, Value::Error(_)));
}

#[test]
fn test_assert_error() {
    let env = test_env();
    let result = eval_expr("(assert-error (error \"test\"))", &env).unwrap();
    assert!(matches!(result, Value::Bool(true)));

    let result = eval_expr("(assert-error 42)", &env).unwrap();
    assert!(matches!(result, Value::Error(_)));
}

// ============================================================================
// String Manipulation Tests
// ============================================================================

#[test]
fn test_string_split() {
    let env = test_env();
    let result = eval_expr("(string-split \"a,b,c\" \",\")", &env).unwrap();
    match result {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::String(s) if s == "a"));
            assert!(matches!(&items[1], Value::String(s) if s == "b"));
            assert!(matches!(&items[2], Value::String(s) if s == "c"));
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_string_join() {
    let env = test_env();
    let result = eval_expr("(string-join '(\"a\" \"b\" \"c\") \",\")", &env).unwrap();
    match result {
        Value::String(s) => assert_eq!(s, "a,b,c"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_substring() {
    let env = test_env();
    let result = eval_expr("(substring \"hello\" 0 3)", &env).unwrap();
    match result {
        Value::String(s) => assert_eq!(s, "hel"),
        _ => panic!("Expected string"),
    }

    let result = eval_expr("(substring \"hello\" 2 5)", &env).unwrap();
    match result {
        Value::String(s) => assert_eq!(s, "llo"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_string_trim() {
    let env = test_env();
    let result = eval_expr("(string-trim \"  hello  \")", &env).unwrap();
    match result {
        Value::String(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_string_upper() {
    let env = test_env();
    let result = eval_expr("(string-upper \"hello\")", &env).unwrap();
    match result {
        Value::String(s) => assert_eq!(s, "HELLO"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_string_lower() {
    let env = test_env();
    let result = eval_expr("(string-lower \"HELLO\")", &env).unwrap();
    match result {
        Value::String(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_string_replace() {
    let env = test_env();
    let result = eval_expr("(string-replace \"hello\" \"l\" \"L\")", &env).unwrap();
    match result {
        Value::String(s) => assert_eq!(s, "heLLo"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_string_contains() {
    let env = test_env();
    let result = eval_expr("(string-contains? \"hello\" \"ell\")", &env).unwrap();
    assert!(matches!(result, Value::Bool(true)));

    let result = eval_expr("(string-contains? \"hello\" \"xyz\")", &env).unwrap();
    assert!(matches!(result, Value::Bool(false)));
}

#[test]
fn test_string_starts_with() {
    let env = test_env();
    let result = eval_expr("(string-starts-with? \"hello\" \"he\")", &env).unwrap();
    assert!(matches!(result, Value::Bool(true)));

    let result = eval_expr("(string-starts-with? \"hello\" \"lo\")", &env).unwrap();
    assert!(matches!(result, Value::Bool(false)));
}

#[test]
fn test_string_ends_with() {
    let env = test_env();
    let result = eval_expr("(string-ends-with? \"hello\" \"lo\")", &env).unwrap();
    assert!(matches!(result, Value::Bool(true)));

    let result = eval_expr("(string-ends-with? \"hello\" \"he\")", &env).unwrap();
    assert!(matches!(result, Value::Bool(false)));
}

#[test]
fn test_string_empty() {
    let env = test_env();
    let result = eval_expr("(string-empty? \"\")", &env).unwrap();
    assert!(matches!(result, Value::Bool(true)));

    let result = eval_expr("(string-empty? \"hello\")", &env).unwrap();
    assert!(matches!(result, Value::Bool(false)));
}

#[test]
fn test_string_length() {
    let env = test_env();
    let result = eval_expr("(string-length \"hello\")", &env).unwrap();
    match result {
        Value::Number(n) => assert_eq!(n, 5.0),
        _ => panic!("Expected number"),
    }

    // Test Unicode handling
    let result = eval_expr("(string-length \"café\")", &env).unwrap();
    match result {
        Value::Number(n) => assert_eq!(n, 4.0),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_string_to_number() {
    let env = test_env();
    let result = eval_expr("(string->number \"42\")", &env).unwrap();
    match result {
        Value::Number(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected number"),
    }

    let result = eval_expr("(string->number \"3.14\")", &env).unwrap();
    match result {
        Value::Number(n) => assert_eq!(n, 3.14),
        _ => panic!("Expected number"),
    }

    // Test invalid string
    let result = eval_expr("(string->number \"xyz\")", &env).unwrap();
    assert!(matches!(result, Value::Error(_)));
}

#[test]
fn test_number_to_string() {
    let env = test_env();
    let result = eval_expr("(number->string 42)", &env).unwrap();
    match result {
        Value::String(s) => assert_eq!(s, "42"),
        _ => panic!("Expected string"),
    }

    let result = eval_expr("(number->string 3.14)", &env).unwrap();
    match result {
        Value::String(s) => assert_eq!(s, "3.14"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_string_to_list() {
    let env = test_env();
    let result = eval_expr("(string->list \"abc\")", &env).unwrap();
    match result {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::String(s) if s == "a"));
            assert!(matches!(&items[1], Value::String(s) if s == "b"));
            assert!(matches!(&items[2], Value::String(s) if s == "c"));
        }
        _ => panic!("Expected list"),
    }
}

#[test]
fn test_list_to_string() {
    let env = test_env();
    let result = eval_expr("(list->string '(\"h\" \"e\" \"l\" \"l\" \"o\"))", &env).unwrap();
    match result {
        Value::String(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected string"),
    }
}

// ============================================================================
// Integration Tests - Using assertions in Lisp
// ============================================================================

#[test]
fn test_lisp_string_tests() {
    let env = test_env();

    // Test that we can write tests in Lisp itself!
    let test_code = r#"
        (begin
            (assert-equal (string-split "a,b,c" ",") '("a" "b" "c") "split test")
            (assert-equal (string-join '("a" "b") "-") "a-b" "join test")
            (assert-equal (substring "hello" 1 4) "ell" "substring test")
            (assert (string-contains? "hello" "ell") "contains test")
            (assert-equal (string-upper "test") "TEST" "upper test")
            (assert-equal (string->number "42") 42 "string->number test")
        )
    "#;

    let result = eval_expr(test_code, &env).unwrap();
    // All assertions should pass, returning #t
    assert!(matches!(result, Value::Bool(true)));
}

#[test]
fn test_lisp_failing_assertion() {
    let env = test_env();

    // This should fail and return an Error value
    let result = eval_expr("(assert-equal 5 10)", &env).unwrap();
    assert!(matches!(result, Value::Error(_)));
}

//! Testing and assertion operations
//!
//! Provides both assertion primitives and test registry for building test suites:
//!
//! **Assertions:**
//! - `assert`: Basic assertion with message
//! - `assert-equal`: Equality assertion with better error messages
//! - `assert-error`: Assert that a value is an error
//!
//! **Test Registry:**
//! - `register-test`: Store a test by name
//! - `run-all-tests`: Execute all registered tests
//! - `clear-tests`: Clear the test registry
//!
//! Assertions return #t on success, or create an Error value on failure

use crate::env::Environment;
use crate::error::EvalError;
use crate::eval::eval;
use crate::value::Value;
use lisp_macros::builtin;
use std::cell::RefCell;
use std::collections::HashMap;

// ============================================================================
// Test Registry
// ============================================================================

thread_local! {
    static TEST_REGISTRY: RefCell<Vec<(String, Value)>> = const { RefCell::new(Vec::new()) };
}

#[builtin(name = "assert", category = "Testing", related(assert-equal, assert-error))]
/// Assert that condition is true. Returns #t on success, Error value on failure.
///
/// Useful for writing tests and validating assumptions in code.
///
/// # Examples
///
/// ```lisp
/// (assert #t) => #t
/// (assert #f) => Error: Assertion failed
/// (assert (= 2 (+ 1 1)) "math works") => #t
/// ```
///
/// # See Also
///
/// assert-equal, assert-error
pub fn builtin_assert(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() || args.len() > 2 {
        return Err(EvalError::ArityMismatch);
    }

    let condition = &args[0];
    let message = if args.len() == 2 {
        match &args[1] {
            Value::String(s) => s.clone(),
            other => format!("{}", other),
        }
    } else {
        "Assertion failed".to_string()
    };

    match condition {
        Value::Bool(true) => Ok(Value::Bool(true)),
        Value::Bool(false) | Value::Nil => Ok(Value::Error(message)),
        _ => Ok(Value::Error(format!(
            "{}: expected boolean, got {}",
            message, condition
        ))),
    }
}

#[builtin(name = "assert-equal", category = "Testing", related(assert, =))]
/// Assert that actual equals expected. Returns #t on success, Error value with details on failure.
///
/// Provides helpful error messages showing both actual and expected values.
///
/// # Examples
///
/// ```lisp
/// (assert-equal 5 5) => #t
/// (assert-equal 5 10) => Error with details
/// (assert-equal (+ 2 2) 4 "addition test") => #t
/// ```
///
/// # See Also
///
/// assert, =
pub fn builtin_assert_equal(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(EvalError::ArityMismatch);
    }

    let actual = &args[0];
    let expected = &args[1];
    let message = if args.len() == 3 {
        match &args[2] {
            Value::String(s) => s.clone(),
            other => format!("{}", other),
        }
    } else {
        "Values not equal".to_string()
    };

    // Compare values based on their types
    let is_equal = values_equal(actual, expected);

    if is_equal {
        Ok(Value::Bool(true))
    } else {
        Ok(Value::Error(format!(
            "{}\n  Expected: {}\n  Actual:   {}",
            message, expected, actual
        )))
    }
}

/// Helper function to recursively compare two values for equality
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Symbol(x), Value::Symbol(y)) => x == y,
        (Value::Keyword(x), Value::Keyword(y)) => x == y,
        (Value::Nil, Value::Nil) => true,
        (Value::List(x), Value::List(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(a, b)| values_equal(a, b))
        }
        (Value::Map(x), Value::Map(y)) => {
            // Compare maps: same keys with equal values
            if x.len() != y.len() {
                return false;
            }
            x.iter()
                .all(|(k, v)| y.get(k).map_or(false, |v2| values_equal(v, v2)))
        }
        (Value::Error(x), Value::Error(y)) => x == y,
        _ => false,
    }
}

#[builtin(name = "assert-error", category = "Testing", related(assert, error?))]
/// Assert that value is an error. Returns #t if value is an Error, Error value otherwise.
///
/// Useful for testing error handling and negative test cases.
///
/// # Examples
///
/// ```lisp
/// (assert-error (error "test")) => #t
/// (assert-error 42) => Error: Expected error value
/// ```
///
/// # See Also
///
/// assert, error?
pub fn builtin_assert_error(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() || args.len() > 2 {
        return Err(EvalError::ArityMismatch);
    }

    let value = &args[0];
    let message = if args.len() == 2 {
        match &args[1] {
            Value::String(s) => s.clone(),
            other => format!("{}", other),
        }
    } else {
        "Expected error value".to_string()
    };

    match value {
        Value::Error(_) => Ok(Value::Bool(true)),
        _ => Ok(Value::Error(format!("{}: got {}", message, value))),
    }
}

// ============================================================================
// Test Registry Functions
// ============================================================================

#[builtin(name = "register-test", category = "Testing", related(run-all-tests, clear-tests))]
/// Register a test with a name and zero-argument lambda.
///
/// Tests are stored globally and can be executed with run-all-tests.
///
/// # Examples
///
/// ```lisp
/// (register-test "simple" (lambda () (assert-equal 1 1)))
/// (register-test "math" (lambda () (assert-equal (+ 2 2) 4)))
/// ```
///
/// # See Also
///
/// run-all-tests, clear-tests
pub fn builtin_register_test(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let name = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(EvalError::TypeError),
    };

    let test_fn = args[1].clone();

    // Verify it's a lambda
    match &test_fn {
        Value::Lambda { .. } => {}
        _ => return Err(EvalError::Custom("Test must be a lambda".to_string())),
    }

    TEST_REGISTRY.with(|registry| {
        registry.borrow_mut().push((name, test_fn));
    });

    Ok(Value::Bool(true))
}

#[builtin(name = "run-all-tests", category = "Testing", related(register-test, clear-tests))]
/// Execute all registered tests and return structured results as a map.
///
/// Returns a map with: {:passed N :failed M :total T :tests [...]}
/// Each test in :tests is a map with :name, :status, :message keys.
///
/// # Examples
///
/// ```lisp
/// (run-all-tests) => {:passed 10 :failed 2 :total 12 :tests [...]}
/// ```
///
/// # See Also
///
/// register-test, clear-tests
pub fn builtin_run_all_tests(args: &[Value]) -> Result<Value, EvalError> {
    if !args.is_empty() {
        return Err(EvalError::ArityMismatch);
    }

    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    TEST_REGISTRY.with(|registry| {
        let tests = registry.borrow();

        for (name, test_fn) in tests.iter() {
            // Call the test lambda (expects 0 args)
            let call_expr = Value::List(vec![test_fn.clone()]);

            // Execute test and capture result
            match eval(call_expr, Environment::new()) {
                Ok(Value::Bool(true)) | Ok(Value::Nil) => {
                    // Test passed
                    passed += 1;
                    let mut result_map = HashMap::new();
                    result_map.insert("name".to_string(), Value::String(name.clone()));
                    result_map.insert("status".to_string(), Value::Symbol("passed".to_string()));
                    result_map.insert("message".to_string(), Value::String(String::new()));
                    results.push(Value::Map(result_map));
                }
                Ok(Value::Error(msg)) => {
                    // Test failed with assertion error
                    failed += 1;
                    let mut result_map = HashMap::new();
                    result_map.insert("name".to_string(), Value::String(name.clone()));
                    result_map.insert("status".to_string(), Value::Symbol("failed".to_string()));
                    result_map.insert("message".to_string(), Value::String(msg));
                    results.push(Value::Map(result_map));
                }
                Ok(_) => {
                    // Test returned non-error value, consider it passed
                    passed += 1;
                    let mut result_map = HashMap::new();
                    result_map.insert("name".to_string(), Value::String(name.clone()));
                    result_map.insert("status".to_string(), Value::Symbol("passed".to_string()));
                    result_map.insert("message".to_string(), Value::String(String::new()));
                    results.push(Value::Map(result_map));
                }
                Err(e) => {
                    // Test threw an exception
                    failed += 1;
                    let mut result_map = HashMap::new();
                    result_map.insert("name".to_string(), Value::String(name.clone()));
                    result_map.insert("status".to_string(), Value::Symbol("error".to_string()));
                    result_map.insert("message".to_string(), Value::String(format!("{:?}", e)));
                    results.push(Value::Map(result_map));
                }
            }
        }
    });

    // Return result as map
    let mut result_map = HashMap::new();
    result_map.insert("passed".to_string(), Value::Number(passed as f64));
    result_map.insert("failed".to_string(), Value::Number(failed as f64));
    result_map.insert("total".to_string(), Value::Number((passed + failed) as f64));
    result_map.insert("tests".to_string(), Value::List(results));

    Ok(Value::Map(result_map))
}

#[builtin(name = "clear-tests", category = "Testing", related(register-test, run-all-tests))]
/// Clear all registered tests from the registry.
///
/// Useful for reloading test files or starting fresh.
///
/// # Examples
///
/// ```lisp
/// (clear-tests) => #t
/// ```
///
/// # See Also
///
/// register-test, run-all-tests
pub fn builtin_clear_tests(args: &[Value]) -> Result<Value, EvalError> {
    if !args.is_empty() {
        return Err(EvalError::ArityMismatch);
    }

    TEST_REGISTRY.with(|registry| {
        registry.borrow_mut().clear();
    });

    Ok(Value::Bool(true))
}

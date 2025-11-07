// ABOUTME: Built-in functions module providing core Lisp primitives

use crate::env::Environment;
use crate::error::EvalError;
use crate::value::Value;
use std::rc::Rc;

// ============================================================================
// Arithmetic Operations
// ============================================================================

/// Addition: (+ 1 2 3) => 6, (+ ) => 0
pub fn builtin_add(args: &[Value]) -> Result<Value, EvalError> {
    let mut sum = 0.0;
    for arg in args {
        match arg {
            Value::Number(n) => sum += n,
            _ => return Err(EvalError::TypeError),
        }
    }
    Ok(Value::Number(sum))
}

/// Subtraction: (- 10 3) => 7, (- 5) => -5, (- 10 5 2) => 3
pub fn builtin_sub(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::ArityMismatch);
    }

    let first = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    if args.len() == 1 {
        return Ok(Value::Number(-first));
    }

    let mut result = first;
    for arg in &args[1..] {
        match arg {
            Value::Number(n) => result -= n,
            _ => return Err(EvalError::TypeError),
        }
    }
    Ok(Value::Number(result))
}

/// Multiplication: (* 2 3 4) => 24, (* ) => 1
pub fn builtin_mul(args: &[Value]) -> Result<Value, EvalError> {
    let mut product = 1.0;
    for arg in args {
        match arg {
            Value::Number(n) => product *= n,
            _ => return Err(EvalError::TypeError),
        }
    }
    Ok(Value::Number(product))
}

/// Division: (/ 10 2) => 5, (/ 10 2 5) => 1
pub fn builtin_div(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::ArityMismatch);
    }

    let first = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    if args.len() == 1 {
        if first == 0.0 {
            return Err(EvalError::Custom("Division by zero".to_string()));
        }
        return Ok(Value::Number(1.0 / first));
    }

    let mut result = first;
    for arg in &args[1..] {
        match arg {
            Value::Number(n) => {
                if *n == 0.0 {
                    return Err(EvalError::Custom("Division by zero".to_string()));
                }
                result /= n;
            }
            _ => return Err(EvalError::TypeError),
        }
    }
    Ok(Value::Number(result))
}

/// Modulo: (% 10 3) => 1
pub fn builtin_mod(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let a = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    let b = match args[1] {
        Value::Number(n) => {
            if n == 0.0 {
                return Err(EvalError::Custom("Division by zero".to_string()));
            }
            n
        }
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Number(a % b))
}

// ============================================================================
// Comparison Operations
// ============================================================================

/// Equality: (= 1 1) => #t
pub fn builtin_eq(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let result = match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Symbol(a), Value::Symbol(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        _ => false,
    };

    Ok(Value::Bool(result))
}

/// Less than: (< 1 2) => #t
pub fn builtin_lt(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let a = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    let b = match args[1] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Bool(a < b))
}

/// Greater than: (> 2 1) => #t
pub fn builtin_gt(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let a = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    let b = match args[1] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Bool(a > b))
}

/// Less than or equal: (<= 1 1) => #t
pub fn builtin_le(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let a = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    let b = match args[1] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Bool(a <= b))
}

/// Greater than or equal: (>= 2 1) => #t
pub fn builtin_ge(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let a = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    let b = match args[1] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Bool(a >= b))
}

// ============================================================================
// Logic Operations
// ============================================================================

/// Logical AND: (and #t #f) => #f
pub fn builtin_and(args: &[Value]) -> Result<Value, EvalError> {
    for arg in args {
        match arg {
            Value::Bool(false) => return Ok(Value::Bool(false)),
            Value::Bool(true) => continue,
            _ => return Err(EvalError::TypeError),
        }
    }
    Ok(Value::Bool(true))
}

/// Logical OR: (or #t #f) => #t
pub fn builtin_or(args: &[Value]) -> Result<Value, EvalError> {
    for arg in args {
        match arg {
            Value::Bool(true) => return Ok(Value::Bool(true)),
            Value::Bool(false) => continue,
            _ => return Err(EvalError::TypeError),
        }
    }
    Ok(Value::Bool(false))
}

/// Logical NOT: (not #f) => #t
pub fn builtin_not(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match args[0] {
        Value::Bool(b) => Ok(Value::Bool(!b)),
        _ => Err(EvalError::TypeError),
    }
}

// ============================================================================
// List Operations
// ============================================================================

/// Cons: (cons 1 '(2 3)) => (1 2 3)
pub fn builtin_cons(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let mut result = vec![args[0].clone()];

    match &args[1] {
        Value::List(items) => result.extend(items.clone()),
        Value::Nil => (),
        _ => return Err(EvalError::TypeError),
    }

    Ok(Value::List(result))
}

/// Car: (car '(1 2 3)) => 1
pub fn builtin_car(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match &args[0] {
        Value::List(items) if !items.is_empty() => Ok(items[0].clone()),
        Value::List(_) => Err(EvalError::Custom("car of empty list".to_string())),
        _ => Err(EvalError::TypeError),
    }
}

/// Cdr: (cdr '(1 2 3)) => (2 3)
pub fn builtin_cdr(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match &args[0] {
        Value::List(items) if !items.is_empty() => {
            if items.len() == 1 {
                Ok(Value::Nil)
            } else {
                Ok(Value::List(items[1..].to_vec()))
            }
        }
        Value::List(_) => Err(EvalError::Custom("cdr of empty list".to_string())),
        _ => Err(EvalError::TypeError),
    }
}

/// List: (list 1 2 3) => (1 2 3)
pub fn builtin_list(args: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::List(args.to_vec()))
}

/// Length: (length '(1 2 3)) => 3
pub fn builtin_length(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match &args[0] {
        Value::List(items) => Ok(Value::Number(items.len() as f64)),
        Value::Nil => Ok(Value::Number(0.0)),
        _ => Err(EvalError::TypeError),
    }
}

/// Empty?: (empty? '()) => #t
pub fn builtin_empty(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match &args[0] {
        Value::List(items) => Ok(Value::Bool(items.is_empty())),
        Value::Nil => Ok(Value::Bool(true)),
        _ => Err(EvalError::TypeError),
    }
}

// ============================================================================
// Type Predicates
// ============================================================================

/// Number predicate: (number? 42) => #t
pub fn builtin_number_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Number(_))))
}

/// String predicate: (string? "hi") => #t
pub fn builtin_string_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::String(_))))
}

/// List predicate: (list? '(1 2)) => #t
pub fn builtin_list_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::List(_))))
}

/// Nil predicate: (nil? nil) => #t
pub fn builtin_nil_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}

/// Symbol predicate: (symbol? 'x) => #t
pub fn builtin_symbol_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Symbol(_))))
}

/// Bool predicate: (bool? #t) => #t
pub fn builtin_bool_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
}

// ============================================================================
// I/O Operations
// ============================================================================

/// Print: (print "hello" 42) => prints hello42 (no newline) and returns nil
pub fn builtin_print(args: &[Value]) -> Result<Value, EvalError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        match arg {
            Value::String(s) => print!("{}", s),
            other => print!("{}", other),
        }
    }
    Ok(Value::Nil)
}

/// Println: (println "hello" 42) => prints hello 42 with newline and returns nil
pub fn builtin_println(args: &[Value]) -> Result<Value, EvalError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        match arg {
            Value::String(s) => print!("{}", s),
            other => print!("{}", other),
        }
    }
    println!();
    Ok(Value::Nil)
}

// ============================================================================
// Error Handling
// ============================================================================

/// Error: (error "something went wrong") => creates an Error value
pub fn builtin_error(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let msg = match &args[0] {
        Value::String(s) => s.clone(),
        other => format!("{}", other),
    };

    Ok(Value::Error(msg))
}

/// Error?: (error? value) => #t if value is an Error, #f otherwise
pub fn builtin_error_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Error(_))))
}

/// Error-msg: (error-msg err) => extracts message from Error value
pub fn builtin_error_msg(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match &args[0] {
        Value::Error(msg) => Ok(Value::String(msg.clone())),
        _ => Err(EvalError::TypeError),
    }
}

// ============================================================================
// Registration
// ============================================================================

/// Register all built-in functions in the global environment
pub fn register_builtins(env: Rc<Environment>) {
    // Arithmetic
    env.define("+".to_string(), Value::BuiltIn(builtin_add));
    env.define("-".to_string(), Value::BuiltIn(builtin_sub));
    env.define("*".to_string(), Value::BuiltIn(builtin_mul));
    env.define("/".to_string(), Value::BuiltIn(builtin_div));
    env.define("%".to_string(), Value::BuiltIn(builtin_mod));

    // Comparison
    env.define("=".to_string(), Value::BuiltIn(builtin_eq));
    env.define("<".to_string(), Value::BuiltIn(builtin_lt));
    env.define(">".to_string(), Value::BuiltIn(builtin_gt));
    env.define("<=".to_string(), Value::BuiltIn(builtin_le));
    env.define(">=".to_string(), Value::BuiltIn(builtin_ge));

    // Logic
    env.define("and".to_string(), Value::BuiltIn(builtin_and));
    env.define("or".to_string(), Value::BuiltIn(builtin_or));
    env.define("not".to_string(), Value::BuiltIn(builtin_not));

    // List operations
    env.define("cons".to_string(), Value::BuiltIn(builtin_cons));
    env.define("car".to_string(), Value::BuiltIn(builtin_car));
    env.define("cdr".to_string(), Value::BuiltIn(builtin_cdr));
    env.define("list".to_string(), Value::BuiltIn(builtin_list));
    env.define("length".to_string(), Value::BuiltIn(builtin_length));
    env.define("empty?".to_string(), Value::BuiltIn(builtin_empty));

    // Type predicates
    env.define("number?".to_string(), Value::BuiltIn(builtin_number_p));
    env.define("string?".to_string(), Value::BuiltIn(builtin_string_p));
    env.define("list?".to_string(), Value::BuiltIn(builtin_list_p));
    env.define("nil?".to_string(), Value::BuiltIn(builtin_nil_p));
    env.define("symbol?".to_string(), Value::BuiltIn(builtin_symbol_p));
    env.define("bool?".to_string(), Value::BuiltIn(builtin_bool_p));

    // I/O
    env.define("print".to_string(), Value::BuiltIn(builtin_print));
    env.define("println".to_string(), Value::BuiltIn(builtin_println));

    // Error handling
    env.define("error".to_string(), Value::BuiltIn(builtin_error));
    env.define("error?".to_string(), Value::BuiltIn(builtin_error_p));
    env.define("error-msg".to_string(), Value::BuiltIn(builtin_error_msg));
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Arithmetic Tests
    // ========================================================================

    #[test]
    fn test_add() {
        let result =
            builtin_add(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 6.0),
            _ => panic!("Expected Number(6.0)"),
        }

        // Zero arguments
        let result = builtin_add(&[]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 0.0),
            _ => panic!("Expected Number(0.0)"),
        }
    }

    #[test]
    fn test_sub() {
        let result = builtin_sub(&[Value::Number(10.0), Value::Number(3.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 7.0),
            _ => panic!("Expected Number(7.0)"),
        }

        // Single argument negation
        let result = builtin_sub(&[Value::Number(5.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, -5.0),
            _ => panic!("Expected Number(-5.0)"),
        }

        // Multiple arguments
        let result =
            builtin_sub(&[Value::Number(10.0), Value::Number(5.0), Value::Number(2.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 3.0),
            _ => panic!("Expected Number(3.0)"),
        }
    }

    #[test]
    fn test_mul() {
        let result =
            builtin_mul(&[Value::Number(2.0), Value::Number(3.0), Value::Number(4.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 24.0),
            _ => panic!("Expected Number(24.0)"),
        }

        // Zero arguments
        let result = builtin_mul(&[]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 1.0),
            _ => panic!("Expected Number(1.0)"),
        }
    }

    #[test]
    fn test_div() {
        let result = builtin_div(&[Value::Number(10.0), Value::Number(2.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 5.0),
            _ => panic!("Expected Number(5.0)"),
        }

        // Division by zero
        let result = builtin_div(&[Value::Number(10.0), Value::Number(0.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_mod() {
        let result = builtin_mod(&[Value::Number(10.0), Value::Number(3.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 1.0),
            _ => panic!("Expected Number(1.0)"),
        }
    }

    // ========================================================================
    // Comparison Tests
    // ========================================================================

    #[test]
    fn test_eq() {
        let result = builtin_eq(&[Value::Number(1.0), Value::Number(1.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_eq(&[Value::Number(1.0), Value::Number(2.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_lt() {
        let result = builtin_lt(&[Value::Number(1.0), Value::Number(2.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_lt(&[Value::Number(2.0), Value::Number(1.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_gt() {
        let result = builtin_gt(&[Value::Number(2.0), Value::Number(1.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    fn test_le() {
        let result = builtin_le(&[Value::Number(1.0), Value::Number(1.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    fn test_ge() {
        let result = builtin_ge(&[Value::Number(2.0), Value::Number(1.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }
    }

    // ========================================================================
    // Logic Tests
    // ========================================================================

    #[test]
    fn test_and() {
        let result = builtin_and(&[Value::Bool(true), Value::Bool(false)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }

        let result = builtin_and(&[Value::Bool(true), Value::Bool(true)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    fn test_or() {
        let result = builtin_or(&[Value::Bool(true), Value::Bool(false)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_or(&[Value::Bool(false), Value::Bool(false)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_not() {
        let result = builtin_not(&[Value::Bool(false)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_not(&[Value::Bool(true)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    // ========================================================================
    // List Operation Tests
    // ========================================================================

    #[test]
    fn test_cons() {
        let result = builtin_cons(&[
            Value::Number(1.0),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ])
        .unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                match &items[0] {
                    Value::Number(n) => assert_eq!(*n, 1.0),
                    _ => panic!("Expected Number(1.0)"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_car() {
        let result = builtin_car(&[Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])])
        .unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 1.0),
            _ => panic!("Expected Number(1.0)"),
        }
    }

    #[test]
    fn test_cdr() {
        let result = builtin_cdr(&[Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])])
        .unwrap();
        match result {
            Value::List(items) => assert_eq!(items.len(), 2),
            _ => panic!("Expected List"),
        }

        // Single element
        let result = builtin_cdr(&[Value::List(vec![Value::Number(1.0)])]).unwrap();
        match result {
            Value::Nil => (),
            _ => panic!("Expected Nil"),
        }
    }

    #[test]
    fn test_list() {
        let result =
            builtin_list(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]).unwrap();
        match result {
            Value::List(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_length() {
        let result = builtin_length(&[Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])])
        .unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 3.0),
            _ => panic!("Expected Number(3.0)"),
        }
    }

    #[test]
    fn test_empty() {
        let result = builtin_empty(&[Value::List(vec![])]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_empty(&[Value::List(vec![Value::Number(1.0)])]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    // ========================================================================
    // Type Predicate Tests
    // ========================================================================

    #[test]
    fn test_number_p() {
        let result = builtin_number_p(&[Value::Number(42.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_number_p(&[Value::String("hello".to_string())]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_string_p() {
        let result = builtin_string_p(&[Value::String("hello".to_string())]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_string_p(&[Value::Number(42.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_list_p() {
        let result = builtin_list_p(&[Value::List(vec![Value::Number(1.0)])]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_list_p(&[Value::Number(42.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_nil_p() {
        let result = builtin_nil_p(&[Value::Nil]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_nil_p(&[Value::Number(42.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_symbol_p() {
        let result = builtin_symbol_p(&[Value::Symbol("x".to_string())]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_symbol_p(&[Value::Number(42.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_bool_p() {
        let result = builtin_bool_p(&[Value::Bool(true)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_bool_p(&[Value::Number(42.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }
}

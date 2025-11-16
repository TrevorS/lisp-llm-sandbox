//! Comparison operations: =, <, >, <=, >=
//!
//! Relational operators for comparing numeric and symbolic values.
//!
//! - `=`: Equality comparison
//! - `<`: Less than
//! - `>`: Greater than
//! - `<=`: Less than or equal
//! - `>=`: Greater than or equal
//!
//! All comparison functions return boolean (#t or #f)

use crate::error::{EvalError, ARITY_TWO};
use crate::value::Value;
use lisp_macros::builtin;

#[builtin(name = "=", category = "Comparison", related(<, >, <=, >=))]
/// Tests if all arguments are equal. Works with numbers, strings, symbols.
///
/// # Examples
///
/// ```lisp
/// (= 5 5) => #t
/// (= 5 6) => #f
/// (= "hello" "hello") => #t
/// ```
///
/// # See Also
///
/// <, >, <=, >=
pub fn builtin_eq(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::arity_error("=", ARITY_TWO, args.len()));
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

#[builtin(name = "<", category = "Comparison", related(>, <=, >=, =))]
/// Tests if each argument is strictly less than the next.
///
/// # Examples
///
/// ```lisp
/// (< 1 2) => #t
/// (< 1 1) => #f
/// (< 5 3) => #f
/// ```
///
/// # See Also
///
/// >, <=, >=, =
pub fn builtin_lt(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::arity_error("<", ARITY_TWO, args.len()));
    }

    let a = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::type_error("<", "number", &args[0], 1)),
    };

    let b = match args[1] {
        Value::Number(n) => n,
        _ => return Err(EvalError::type_error("<", "number", &args[1], 2)),
    };

    Ok(Value::Bool(a < b))
}

#[builtin(name = ">", category = "Comparison", related(<, <=, >=, =))]
/// Tests if each argument is strictly greater than the next.
///
/// # Examples
///
/// ```lisp
/// (> 3 2) => #t
/// (> 3 3) => #f
/// ```
///
/// # See Also
///
/// <, <=, >=, =
pub fn builtin_gt(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::arity_error(">", ARITY_TWO, args.len()));
    }

    let a = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::type_error(">", "number", &args[0], 1)),
    };

    let b = match args[1] {
        Value::Number(n) => n,
        _ => return Err(EvalError::type_error(">", "number", &args[1], 2)),
    };

    Ok(Value::Bool(a > b))
}

#[builtin(name = "<=", category = "Comparison", related(<, >, >=, =))]
/// Tests if each argument is less than or equal to the next.
///
/// # Examples
///
/// ```lisp
/// (<= 1 2) => #t
/// (<= 5 5) => #t
/// ```
///
/// # See Also
///
/// <, >, >=, =
pub fn builtin_le(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::arity_error("<=", ARITY_TWO, args.len()));
    }

    let a = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::type_error("<=", "number", &args[0], 1)),
    };

    let b = match args[1] {
        Value::Number(n) => n,
        _ => return Err(EvalError::type_error("<=", "number", &args[1], 2)),
    };

    Ok(Value::Bool(a <= b))
}

#[builtin(name = ">=", category = "Comparison", related(<, >, <=, =))]
/// Tests if each argument is greater than or equal to the next.
///
/// # Examples
///
/// ```lisp
/// (>= 3 2) => #t
/// (>= 5 5) => #t
/// ```
///
/// # See Also
///
/// <, >, <=, =
pub fn builtin_ge(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::arity_error(">=", ARITY_TWO, args.len()));
    }

    let a = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::type_error(">=", "number", &args[0], 1)),
    };

    let b = match args[1] {
        Value::Number(n) => n,
        _ => return Err(EvalError::type_error(">=", "number", &args[1], 2)),
    };

    Ok(Value::Bool(a >= b))
}

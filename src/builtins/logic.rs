//! Logic operations: and, or, not
//!
//! Boolean operators for logical composition and negation.
//!
//! - `and`: Logical AND (short-circuits on first false)
//! - `or`: Logical OR (short-circuits on first true)
//! - `not`: Logical NOT (negation)
//!
//! All functions return boolean (#t or #f)

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;

#[builtin(name = "and", category = "Logic", related(or, not))]
/// Logical AND. Returns #f if any argument is falsy, otherwise returns the last argument.
///
/// Short-circuits: stops evaluating after first falsy value.
///
/// # Examples
///
/// ```lisp
/// (and #t #t #t) => #t
/// (and #t #f #t) => #f
/// ```
///
/// # See Also
///
/// or, not
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

#[builtin(name = "or", category = "Logic", related(and, not))]
/// Logical OR. Returns the first truthy value or #f if all are falsy.
///
/// Short-circuits: stops evaluating after first truthy value.
///
/// # Examples
///
/// ```lisp
/// (or #f #f #t) => #t
/// (or #f #f) => #f
/// ```
///
/// # See Also
///
/// and, not
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

#[builtin(name = "not", category = "Logic", related(and, or))]
/// Logical NOT. Returns #t if val is falsy (#f or nil), otherwise #f.
///
/// # Examples
///
/// ```lisp
/// (not #f) => #t
/// (not #t) => #f
/// (not nil) => #t
/// ```
///
/// # See Also
///
/// and, or
pub fn builtin_not(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match args[0] {
        Value::Bool(b) => Ok(Value::Bool(!b)),
        _ => Err(EvalError::TypeError),
    }
}

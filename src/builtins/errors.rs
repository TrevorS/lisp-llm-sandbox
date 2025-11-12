//! Error handling operations: error, error?, error-msg
//!
//! Functions for working with catchable error values.
//!
//! - `error`: Create an error value with message
//! - `error?`: Test if value is an error
//! - `error-msg`: Extract error message from error value
//!
//! Errors are first-class values, not exceptions, enabling graceful error handling

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;

#[builtin(name = "error", category = "Error handling", related(error?, error-msg))]
/// Raises an error with the given message. Always throws.
///
/// # Examples
///
/// ```lisp
/// (error "invalid input") => Error: invalid input
/// ```
///
/// # See Also
///
/// error?, error-msg
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

#[builtin(name = "error?", category = "Error handling", related(error, error-msg))]
/// Tests if val is an error value.
///
/// # Examples
///
/// ```lisp
/// (error? (error "test")) => would throw before testing
/// ```
///
/// # See Also
///
/// error, error-msg
pub fn builtin_error_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Error(_))))
}

#[builtin(name = "error-msg", category = "Error handling", related(error, error?))]
/// Extracts the message from an error value.
///
/// # Examples
///
/// ```lisp
/// (error-msg (error "test")) => would throw before extracting
/// ```
///
/// # See Also
///
/// error, error?
pub fn builtin_error_msg(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match &args[0] {
        Value::Error(msg) => Ok(Value::String(msg.clone())),
        _ => Err(EvalError::TypeError),
    }
}

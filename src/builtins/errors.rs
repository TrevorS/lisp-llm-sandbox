//! Error handling operations: error, error?, error-msg
//!
//! Functions for working with catchable error values.
//!
//! - `error`: Create an error value with message
//! - `error?`: Test if value is an error
//! - `error-msg`: Extract error message from error value
//!
//! Errors are first-class values, not exceptions, enabling graceful error handling

use crate::env::Environment;
use crate::error::EvalError;
use crate::value::Value;
use std::rc::Rc;

/// Raises an error with the given message
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

/// Tests if val is an error value
pub fn builtin_error_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Error(_))))
}

/// Extracts the message from an error value
pub fn builtin_error_msg(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match &args[0] {
        Value::Error(msg) => Ok(Value::String(msg.clone())),
        _ => Err(EvalError::TypeError),
    }
}

/// Register all error handling builtins in the environment
pub fn register(env: &Rc<Environment>) {
    env.define("error".to_string(), Value::BuiltIn(builtin_error));
    env.define("error?".to_string(), Value::BuiltIn(builtin_error_p));
    env.define("error-msg".to_string(), Value::BuiltIn(builtin_error_msg));

    // Register help entries
    crate::help::register_help(crate::help::HelpEntry {
        name: "error".to_string(),
        signature: "(error ...)".to_string(),
        description: "Raises an error with the given message. Always throws.".to_string(),
        examples: vec!["(error \"invalid input\") => Error: invalid input".to_string()],
        related: vec!["error?".to_string(), "error-msg".to_string()],
        category: "Error handling".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "error?".to_string(),
        signature: "(error? ...)".to_string(),
        description: "Tests if val is an error value.".to_string(),
        examples: vec!["(error? (error \"test\")) => would throw before testing".to_string()],
        related: vec!["error".to_string(), "error-msg".to_string()],
        category: "Error handling".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "error-msg".to_string(),
        signature: "(error-msg ...)".to_string(),
        description: "Extracts the message from an error value.".to_string(),
        examples: vec!["(error-msg (error \"test\")) => would throw before extracting".to_string()],
        related: vec!["error".to_string(), "error?".to_string()],
        category: "Error handling".to_string(),
    });
}

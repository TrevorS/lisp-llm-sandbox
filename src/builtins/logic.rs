//! Logic operations: and, or, not
//!
//! Boolean operators for logical composition and negation.
//!
//! - `and`: Logical AND (short-circuits on first false)
//! - `or`: Logical OR (short-circuits on first true)
//! - `not`: Logical NOT (negation)
//!
//! All functions return boolean (#t or #f)

use crate::env::Environment;
use crate::error::EvalError;
use crate::value::Value;
use std::rc::Rc;

/// Logical AND. Returns #f if any argument is falsy, otherwise returns the last argument.
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

/// Logical OR. Returns the first truthy value or #f if all are falsy.
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

/// Logical NOT. Returns #t if val is falsy (#f or nil), otherwise #f.
pub fn builtin_not(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match args[0] {
        Value::Bool(b) => Ok(Value::Bool(!b)),
        _ => Err(EvalError::TypeError),
    }
}

/// Register all logic builtins in the environment
pub fn register(env: &Rc<Environment>) {
    env.define("and".to_string(), Value::BuiltIn(builtin_and));
    env.define("or".to_string(), Value::BuiltIn(builtin_or));
    env.define("not".to_string(), Value::BuiltIn(builtin_not));

    // Register help entries
    crate::help::register_help(crate::help::HelpEntry {
        name: "and".to_string(),
        signature: "(and ...)".to_string(),
        description: "Logical AND. Returns #f if any argument is falsy, otherwise returns the last argument.\nShort-circuits: stops evaluating after first falsy value.".to_string(),
        examples: vec![
            "(and #t #t #t) => #t".to_string(),
            "(and #t #f #t) => #f".to_string(),
            "(and 1 2 3) => 3".to_string(),
        ],
        related: vec!["or".to_string(), "not".to_string()],
        category: "Logic".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "or".to_string(),
        signature: "(or ...)".to_string(),
        description: "Logical OR. Returns the first truthy value or #f if all are falsy.\nShort-circuits: stops evaluating after first truthy value.".to_string(),
        examples: vec![
            "(or #f #f #t) => #t".to_string(),
            "(or #f #f) => #f".to_string(),
            "(or nil 2) => 2".to_string(),
        ],
        related: vec!["and".to_string(), "not".to_string()],
        category: "Logic".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "not".to_string(),
        signature: "(not ...)".to_string(),
        description: "Logical NOT. Returns #t if val is falsy (#f or nil), otherwise #f."
            .to_string(),
        examples: vec![
            "(not #f) => #t".to_string(),
            "(not #t) => #f".to_string(),
            "(not nil) => #t".to_string(),
            "(not 5) => #f".to_string(),
        ],
        related: vec!["and".to_string(), "or".to_string()],
        category: "Logic".to_string(),
    });
}

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

use crate::env::Environment;
use crate::error::EvalError;
use crate::value::Value;
use std::rc::Rc;

/// Tests if all arguments are equal. Works with numbers, strings, symbols.
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

/// Tests if each argument is strictly less than the next
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

/// Tests if each argument is strictly greater than the next
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

/// Tests if each argument is less than or equal to the next
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

/// Tests if each argument is greater than or equal to the next
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

/// Register all comparison builtins in the environment
pub fn register(env: &Rc<Environment>) {
    env.define("=".to_string(), Value::BuiltIn(builtin_eq));
    env.define("<".to_string(), Value::BuiltIn(builtin_lt));
    env.define(">".to_string(), Value::BuiltIn(builtin_gt));
    env.define("<=".to_string(), Value::BuiltIn(builtin_le));
    env.define(">=".to_string(), Value::BuiltIn(builtin_ge));

    // Register help entries
    crate::help::register_help(crate::help::HelpEntry {
        name: "=".to_string(),
        signature: "(= ...)".to_string(),
        description: "Tests if all arguments are equal. Works with numbers, strings, symbols."
            .to_string(),
        examples: vec![
            "(= 5 5) => #t".to_string(),
            "(= 5 5 5) => #t".to_string(),
            "(= 5 6) => #f".to_string(),
            "(= \"hello\" \"hello\") => #t".to_string(),
        ],
        related: vec![
            "<".to_string(),
            ">".to_string(),
            "<=".to_string(),
            ">=".to_string(),
        ],
        category: "Comparison".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "<".to_string(),
        signature: "(< ...)".to_string(),
        description: "Tests if each argument is strictly less than the next.".to_string(),
        examples: vec![
            "(< 1 2 3) => #t".to_string(),
            "(< 1 1) => #f".to_string(),
            "(< 5 3) => #f".to_string(),
        ],
        related: vec![
            ">".to_string(),
            "<=".to_string(),
            ">=".to_string(),
            "=".to_string(),
        ],
        category: "Comparison".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: ">".to_string(),
        signature: "(> ...)".to_string(),
        description: "Tests if each argument is strictly greater than the next.".to_string(),
        examples: vec!["(> 3 2 1) => #t".to_string(), "(> 3 3) => #f".to_string()],
        related: vec![
            "<".to_string(),
            "<=".to_string(),
            ">=".to_string(),
            "=".to_string(),
        ],
        category: "Comparison".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "<=".to_string(),
        signature: "(<= ...)".to_string(),
        description: "Tests if each argument is less than or equal to the next.".to_string(),
        examples: vec![
            "(<= 1 2 2 3) => #t".to_string(),
            "(<= 5 5) => #t".to_string(),
        ],
        related: vec![
            "<".to_string(),
            ">".to_string(),
            ">=".to_string(),
            "=".to_string(),
        ],
        category: "Comparison".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: ">=".to_string(),
        signature: "(>= ...)".to_string(),
        description: "Tests if each argument is greater than or equal to the next.".to_string(),
        examples: vec![
            "(>= 3 2 2 1) => #t".to_string(),
            "(>= 5 5) => #t".to_string(),
        ],
        related: vec![
            "<".to_string(),
            ">".to_string(),
            "<=".to_string(),
            "=".to_string(),
        ],
        category: "Comparison".to_string(),
    });
}

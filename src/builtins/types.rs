//! Type predicates: number?, string?, list?, nil?, symbol?, bool?
//!
//! Functions for checking the type of a value.
//!
//! - `number?`: Test if value is a numeric (f64)
//! - `string?`: Test if value is a string
//! - `list?`: Test if value is a list
//! - `nil?`: Test if value is nil
//! - `symbol?`: Test if value is a symbol
//! - `bool?`: Test if value is a boolean (#t or #f)
//!
//! All return boolean (#t or #f)

use crate::env::Environment;
use crate::error::EvalError;
use crate::value::Value;
use std::rc::Rc;

/// Tests if val is a number (integer or float)
pub fn builtin_number_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Number(_))))
}

/// Tests if val is a string
pub fn builtin_string_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::String(_))))
}

/// Tests if val is a list (including nil)
pub fn builtin_list_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::List(_))))
}

/// Tests if val is nil (empty list)
pub fn builtin_nil_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}

/// Tests if val is a symbol
pub fn builtin_symbol_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Symbol(_))))
}

/// Tests if val is a boolean (#t or #f)
pub fn builtin_bool_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
}

/// Register all type predicate builtins in the environment
pub fn register(env: &Rc<Environment>) {
    env.define("number?".to_string(), Value::BuiltIn(builtin_number_p));
    env.define("string?".to_string(), Value::BuiltIn(builtin_string_p));
    env.define("list?".to_string(), Value::BuiltIn(builtin_list_p));
    env.define("nil?".to_string(), Value::BuiltIn(builtin_nil_p));
    env.define("symbol?".to_string(), Value::BuiltIn(builtin_symbol_p));
    env.define("bool?".to_string(), Value::BuiltIn(builtin_bool_p));

    // Register help entries
    crate::help::register_help(crate::help::HelpEntry {
        name: "number?".to_string(),
        signature: "(number? val)".to_string(),
        description: "Tests if val is a number (integer or float).".to_string(),
        examples: vec![
            "(number? 42) => #t".to_string(),
            "(number? 3.14) => #t".to_string(),
            "(number? \"42\") => #f".to_string(),
        ],
        related: vec![
            "string?".to_string(),
            "symbol?".to_string(),
            "list?".to_string(),
        ],
        category: "Type predicates".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "string?".to_string(),
        signature: "(string? val)".to_string(),
        description: "Tests if val is a string.".to_string(),
        examples: vec![
            "(string? \"hello\") => #t".to_string(),
            "(string? 42) => #f".to_string(),
            "(string? 'hello) => #f".to_string(),
        ],
        related: vec!["number?".to_string(), "symbol?".to_string()],
        category: "Type predicates".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "list?".to_string(),
        signature: "(list? val)".to_string(),
        description: "Tests if val is a list (including nil).".to_string(),
        examples: vec![
            "(list? '(1 2 3)) => #t".to_string(),
            "(list? nil) => #t".to_string(),
            "(list? 42) => #f".to_string(),
        ],
        related: vec![
            "number?".to_string(),
            "string?".to_string(),
            "nil?".to_string(),
        ],
        category: "Type predicates".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "nil?".to_string(),
        signature: "(nil? val)".to_string(),
        description: "Tests if val is nil (empty list).".to_string(),
        examples: vec![
            "(nil? nil) => #t".to_string(),
            "(nil? '()) => #t".to_string(),
            "(nil? 0) => #f".to_string(),
        ],
        related: vec!["empty?".to_string(), "list?".to_string()],
        category: "Type predicates".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "symbol?".to_string(),
        signature: "(symbol? val)".to_string(),
        description: "Tests if val is a symbol (e.g., from 'hello or var names).".to_string(),
        examples: vec![
            "(symbol? 'hello) => #t".to_string(),
            "(symbol? \"hello\") => #f".to_string(),
            "(symbol? hello) => error (undefined variable)".to_string(),
        ],
        related: vec!["string?".to_string(), "number?".to_string()],
        category: "Type predicates".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "bool?".to_string(),
        signature: "(bool? val)".to_string(),
        description: "Tests if val is a boolean (#t or #f).".to_string(),
        examples: vec![
            "(bool? #t) => #t".to_string(),
            "(bool? #f) => #t".to_string(),
            "(bool? 1) => #f".to_string(),
        ],
        related: vec!["number?".to_string(), "string?".to_string()],
        category: "Type predicates".to_string(),
    });
}

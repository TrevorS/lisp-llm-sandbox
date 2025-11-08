//! List operations: cons, car, cdr, list, length, empty?
//!
//! Functions for building and manipulating lists.
//!
//! - `cons`: Construct a list by prepending element to list
//! - `car`: Get first element of list
//! - `cdr`: Get rest of list (all but first element)
//! - `list`: Create a list from arguments
//! - `length`: Get number of elements in list
//! - `empty?`: Test if list is empty

use crate::env::Environment;
use crate::error::EvalError;
use crate::value::Value;
use std::rc::Rc;

/// Constructs a new list by prepending elem to list
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

/// Returns the first element of a list. Also called 'head'.
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

/// Returns all elements except the first. Also called 'tail'.
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

/// Creates a new list containing the given elements in order
pub fn builtin_list(args: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::List(args.to_vec()))
}

/// Returns the number of elements in a list
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

/// Tests if a list is empty (nil or ())
pub fn builtin_empty_q(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match &args[0] {
        Value::List(items) => Ok(Value::Bool(items.is_empty())),
        Value::Nil => Ok(Value::Bool(true)),
        _ => Err(EvalError::TypeError),
    }
}

/// Register all list builtins in the environment
pub fn register(env: &Rc<Environment>) {
    env.define("cons".to_string(), Value::BuiltIn(builtin_cons));
    env.define("car".to_string(), Value::BuiltIn(builtin_car));
    env.define("cdr".to_string(), Value::BuiltIn(builtin_cdr));
    env.define("list".to_string(), Value::BuiltIn(builtin_list));
    env.define("length".to_string(), Value::BuiltIn(builtin_length));
    env.define("empty?".to_string(), Value::BuiltIn(builtin_empty_q));

    // Register help entries
    crate::help::register_help(crate::help::HelpEntry {
        name: "cons".to_string(),
        signature: "(cons ...)".to_string(),
        description: "Constructs a new list by prepending elem to list.\nReturns a new list; original is not modified.".to_string(),
        examples: vec![
            "(cons 1 '(2 3)) => (1 2 3)".to_string(),
            "(cons 'a '(b c)) => (a b c)".to_string(),
            "(cons 1 nil) => (1)".to_string(),
        ],
        related: vec!["car".to_string(), "cdr".to_string(), "list".to_string()],
        category: "List operations".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "car".to_string(),
        signature: "(car ...)".to_string(),
        description: "Returns the first element of a list. Also called 'head'.\nThrows error on empty list or non-list.".to_string(),
        examples: vec![
            "(car '(1 2 3)) => 1".to_string(),
            "(car '(a)) => a".to_string(),
        ],
        related: vec!["cdr".to_string(), "cons".to_string()],
        category: "List operations".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "cdr".to_string(),
        signature: "(cdr ...)".to_string(),
        description: "Returns all elements except the first. Also called 'tail'.\nReturns nil for single-element list.".to_string(),
        examples: vec![
            "(cdr '(1 2 3)) => (2 3)".to_string(),
            "(cdr '(a b)) => (b)".to_string(),
            "(cdr '(1)) => nil".to_string(),
        ],
        related: vec!["car".to_string(), "cons".to_string()],
        category: "List operations".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "list".to_string(),
        signature: "(list ...)".to_string(),
        description: "Creates a new list containing the given elements in order.".to_string(),
        examples: vec![
            "(list 1 2 3) => (1 2 3)".to_string(),
            "(list 'a 'b) => (a b)".to_string(),
            "(list) => nil".to_string(),
        ],
        related: vec!["cons".to_string(), "car".to_string(), "cdr".to_string()],
        category: "List operations".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "length".to_string(),
        signature: "(length ...)".to_string(),
        description: "Returns the number of elements in a list.".to_string(),
        examples: vec![
            "(length '(1 2 3)) => 3".to_string(),
            "(length '()) => 0".to_string(),
            "(length '(a)) => 1".to_string(),
        ],
        related: vec!["empty?".to_string(), "list".to_string()],
        category: "List operations".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "empty?".to_string(),
        signature: "(empty? list)".to_string(),
        description:
            "Tests if a list is empty (nil or ()).\nReturns #t for empty lists, #f otherwise."
                .to_string(),
        examples: vec![
            "(empty? nil) => #t".to_string(),
            "(empty? '()) => #t".to_string(),
            "(empty? '(1)) => #f".to_string(),
        ],
        related: vec!["length".to_string(), "nil?".to_string()],
        category: "List operations".to_string(),
    });
}

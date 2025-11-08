//! Console I/O operations: print, println
//!
//! Functions for output to standard output.
//!
//! - `print`: Output value without newline
//! - `println`: Output value with trailing newline
//!
//! Both return nil

use crate::env::Environment;
use crate::error::EvalError;
use crate::value::Value;
use std::rc::Rc;

/// Prints values to stdout without newline. Returns nil.
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

/// Prints values to stdout with newline at end. Returns nil.
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

/// Register all console I/O builtins in the environment
pub fn register(env: &Rc<Environment>) {
    env.define("print".to_string(), Value::BuiltIn(builtin_print));
    env.define("println".to_string(), Value::BuiltIn(builtin_println));

    // Register help entries
    crate::help::register_help(crate::help::HelpEntry {
        name: "print".to_string(),
        signature: "(print ...)".to_string(),
        description: "Prints values to stdout without newline. Returns nil.".to_string(),
        examples: vec![
            "(print \"hello\") => outputs: hello".to_string(),
            "(print 1 2 3) => outputs: 1 2 3".to_string(),
        ],
        related: vec!["println".to_string()],
        category: "Console I/O".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "println".to_string(),
        signature: "(println ...)".to_string(),
        description: "Prints values to stdout with newline at end. Returns nil.".to_string(),
        examples: vec![
            "(println \"hello\") => outputs: hello".to_string(),
            "(println \"a\" \"b\") => outputs: a b".to_string(),
        ],
        related: vec!["print".to_string()],
        category: "Console I/O".to_string(),
    });
}

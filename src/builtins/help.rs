//! Help system operations: help, doc
//!
//! Functions for accessing documentation and help information.
//!
//! - `help`: Show help for a function (displays markdown documentation)
//! - `doc`: Extract docstring from a user-defined function
//!
//! The help system includes all 32 built-in functions and 8 special forms.
//! User-defined functions can include docstrings as the first element of the body.

use crate::env::Environment;
use crate::error::{EvalError, ARITY_ONE, ARITY_ZERO_OR_ONE};
use crate::value::Value;
use std::sync::Arc;

/// Show help information
pub fn builtin_help(args: &[Value]) -> Result<Value, EvalError> {
    use crate::help;

    match args.len() {
        0 => {
            // Show quick reference
            let output = help::format_quick_reference();
            println!("{}", output);
            Ok(Value::Nil)
        }
        1 => {
            // Get help for specific function
            match &args[0] {
                Value::Symbol(name) => {
                    // First try built-in help
                    if let Some(entry) = help::get_help(name) {
                        let output = help::format_help_entry(&entry);
                        println!("{}", output);
                        return Ok(Value::Nil);
                    }

                    // If not found in help registry, it might be a user function
                    // User functions would need to be looked up in environment
                    // For now, just report not found
                    Err(EvalError::runtime_error(
                        "help",
                        format!("no help found for '{}'", name),
                    ))
                }
                _ => Err(EvalError::type_error("help", "symbol", &args[0], 1)),
            }
        }
        _ => Err(EvalError::arity_error(
            "help",
            ARITY_ZERO_OR_ONE,
            args.len(),
        )),
    }
}

/// Returns the docstring of a function as a string
pub fn builtin_doc(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_error("doc", ARITY_ONE, args.len()));
    }

    match &args[0] {
        Value::Lambda { docstring, .. } => match docstring {
            Some(doc) => Ok(Value::String(doc.clone())),
            None => Ok(Value::Nil),
        },
        _ => Err(EvalError::type_error("doc", "lambda", &args[0], 1)),
    }
}

/// Register all help system builtins in the environment
pub fn register(_env: &Arc<Environment>) {
    crate::eval::extend_global_env("help".to_string(), Value::BuiltIn(builtin_help));
    crate::eval::extend_global_env("doc".to_string(), Value::BuiltIn(builtin_doc));

    // Register help entries
    crate::help::register_help(crate::help::HelpEntry {
        name: "help".to_string(),
        signature: "(help) or (help 'function-name)".to_string(),
        description: "Show help information. With no arguments, displays quick reference.\nWith a function name, shows detailed documentation for that function.".to_string(),
        examples: vec![
            "(help) => shows quick reference".to_string(),
            "(help 'cons) => detailed help for cons".to_string(),
            "(help 'map) => help for user or stdlib function".to_string(),
        ],
        related: vec!["doc".to_string()],
        category: "Help system".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "doc".to_string(),
        signature: "(doc ...)".to_string(),
        description: "Returns the docstring of a function as a string.\nWorks with user-defined functions that have docstrings.".to_string(),
        examples: vec!["(doc factorial) => \"Computes factorial\"".to_string()],
        related: vec!["help".to_string()],
        category: "Help system".to_string(),
    });
}

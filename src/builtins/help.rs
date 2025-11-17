//! Help system operations: help
//!
//! Functions for accessing documentation and help information.
//!
//! - `help`: Show help for a function (displays markdown documentation)
//!
//! The help system includes all built-in functions and special forms.
//! Stdlib functions are documented using ;;; comments which are parsed and registered.

use crate::env::Environment;
use crate::error::{EvalError, ARITY_ZERO_OR_ONE};
use crate::value::Value;
use std::rc::Rc;

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

/// Register all help system builtins in the environment
pub fn register(env: &Rc<Environment>) {
    env.define("help".to_string(), Value::BuiltIn(builtin_help));

    // Register help entries
    crate::help::register_help(crate::help::HelpEntry {
        name: "help".to_string(),
        signature: "(help) or (help 'function-name)".to_string(),
        description: "Show help information. With no arguments, displays quick reference.\nWith a function name, shows detailed documentation for that function.\n\nDocumentation for built-ins and special forms is maintained in Rust code.\nDocumentation for stdlib functions uses ;;; comments which are parsed and registered.".to_string(),
        examples: vec![
            "(help) => shows quick reference".to_string(),
            "(help 'cons) => detailed help for cons".to_string(),
            "(help 'map) => help for stdlib function".to_string(),
        ],
        related: vec![],
        category: "Help system".to_string(),
    });
}

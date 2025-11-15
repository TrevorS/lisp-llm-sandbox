//! # Built-in Functions Module
//!
//! Core built-in functions for the Lisp interpreter, organized into 12 categories with 55 total functions.
//!
//! ## Naming Convention
//!
//! All builtin functions use **kebab-case WITHOUT namespaces** (e.g., `map-get`, `string-split`, `http-request`)
//! because they are fundamental language primitives.
//!
//! For domain-specific helpers or library-like APIs with 3+ related functions,
//! create a stdlib module with namespaced names instead (see src/stdlib/).
//! For example: `json:encode`, `http:body`, `map:query`
//!
//! See CLAUDE.md "Naming Conventions" section for detailed guidance.
//!
//! ## Categories
//!
//! - **[arithmetic]** (5): +, -, *, /, % - Numeric operations
//! - **[comparison]** (5): =, <, >, <=, >= - Value comparisons
//! - **[logic]** (3): and, or, not - Boolean operations
//! - **[types]** (6): number?, string?, list?, nil?, symbol?, bool? - Type predicates
//! - **[lists]** (6): cons, car, cdr, list, length, empty? - List manipulation
//! - **[console]** (2): print, println - Output operations
//! - **[filesystem]** (5): read-file, write-file, file-exists?, file-size, list-files - File I/O
//! - **[network]** (2): http-get, http-post - Network requests
//! - **[errors]** (3): error, error?, error-msg - Error handling
//! - **[strings]** (17): string-split, string-join, string-append, substring, string-trim, string-upper, string-lower, string-replace, string-contains?, string-starts-with?, string-ends-with?, string-empty?, string-length, string->number, number->string, string->list, list->string - String manipulation
//! - **[testing]** (6): assert, assert-equal, assert-error, register-test, run-all-tests, clear-tests - Testing and assertions
//! - **[help_builtins]** (2): help, doc - Documentation system
//!
//! Each category is a sub-module with its own register function that sets up both the
//! function bindings and their help documentation entries in the help system registry.

use crate::env::Environment;
use crate::error::EvalError;
use crate::help::HelpEntry;
use crate::sandbox::Sandbox;
use crate::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// Builtin Auto-Registration Infrastructure
// ============================================================================

/// Registration entry for a builtin function (auto-collected via inventory)
pub struct BuiltinRegistration {
    pub name: &'static str,
    pub function: fn(&[Value]) -> Result<Value, EvalError>,
    pub signature: &'static str,
    pub description: &'static str,
    pub examples: &'static [&'static str],
    pub related: &'static [&'static str],
    pub category: &'static str,
}

// Collect all builtin registrations at compile time
inventory::collect!(BuiltinRegistration);

// ============================================================================
// Sandbox Storage for I/O Built-in Functions
// ============================================================================

thread_local! {
    static SANDBOX: RefCell<Option<Sandbox>> = const { RefCell::new(None) };
}

/// Initialize the sandbox for I/O built-in functions
pub fn set_sandbox_storage(sandbox: Sandbox) {
    SANDBOX.with(|s| {
        *s.borrow_mut() = Some(sandbox);
    });
}

// ============================================================================
// Sub-modules
// ============================================================================

pub mod arithmetic;
pub mod comparison;
pub mod concurrency;
pub mod console;
pub mod errors;
pub mod filesystem;
#[path = "help.rs"]
pub mod help_builtins;
pub mod lists;
pub mod logic;
pub mod maps;
pub mod network;
pub mod strings;
pub mod testing;
pub mod types;

// ============================================================================
// Main Registration Function (Auto-Registration via Inventory)
// ============================================================================

/// Register all built-in functions in the environment
///
/// This function automatically discovers and registers all functions marked with
/// #[builtin] across all modules via the inventory crate's compile-time collection.
pub fn register_builtins(env: Rc<Environment>) {
    // Automatically iterate over all collected builtins
    for builtin in inventory::iter::<BuiltinRegistration> {
        env.define(builtin.name.to_string(), Value::BuiltIn(builtin.function));

        // Convert static help data to HelpEntry
        crate::help::register_help(HelpEntry {
            name: builtin.name.to_string(),
            signature: builtin.signature.to_string(),
            description: builtin.description.to_string(),
            examples: builtin.examples.iter().map(|s| s.to_string()).collect(),
            related: builtin.related.iter().map(|s| s.to_string()).collect(),
            category: builtin.category.to_string(),
        });
    }

    // Note: help_builtins module still needs manual registration since it uses
    // special forms and environment access (not simple builtin functions)
    help_builtins::register(&env);
}

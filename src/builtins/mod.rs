//! # Built-in Functions Module
//!
//! Core built-in functions for the Lisp interpreter, organized into 10 categories with 32 total functions.
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
//! - **[help_builtins]** (2): help, doc - Documentation system
//!
//! Each category is a sub-module with its own register function that sets up both the
//! function bindings and their help documentation entries in the help system registry.

use crate::env::Environment;
use crate::sandbox::Sandbox;
use std::cell::RefCell;
use std::rc::Rc;

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
pub mod console;
pub mod errors;
pub mod filesystem;
#[path = "help.rs"]
pub mod help_builtins;
pub mod lists;
pub mod logic;
pub mod network;
pub mod types;

// Re-export for convenience
pub use arithmetic::register as register_arithmetic;
pub use comparison::register as register_comparison;
pub use console::register as register_console;
pub use errors::register as register_errors;
pub use filesystem::register as register_filesystem;
pub use help_builtins::register as register_help;
pub use lists::register as register_lists;
pub use logic::register as register_logic;
pub use network::register as register_network;
pub use types::register as register_types;

// ============================================================================
// Main Registration Function
// ============================================================================

/// Register all built-in functions in the environment
pub fn register_builtins(env: Rc<Environment>) {
    // Register all builtin categories
    register_arithmetic(&env);
    register_comparison(&env);
    register_logic(&env);
    register_types(&env);
    register_lists(&env);
    register_console(&env);
    register_filesystem(&env);
    register_network(&env);
    register_errors(&env);
    register_help(&env);
}

//! Standard library modules for higher-level functionality
//!
//! This module contains stdlib extensions that build on top of the core builtins.
//! Each submodule provides a domain-specific set of functions (json, http, time, etc.)

use crate::env::Environment;
use std::rc::Rc;

pub mod json;

/// Register all stdlib modules in the environment
pub fn register_stdlib(env: Rc<Environment>) {
    json::register(&env);
}

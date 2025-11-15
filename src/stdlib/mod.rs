//! Standard library modules for domain-specific functionality
//!
//! This module contains stdlib extensions that build on top of the core builtins.
//! Each submodule provides a domain-specific set of functions (json, http, etc.).
//!
//! ## Naming Convention
//!
//! Stdlib modules use **namespaced function names** (e.g., `json:encode`, `http:body`)
//! to distinguish them from core primitives. This follows Clojure-style conventions:
//!
//! - **Namespaced** (`module:function`) — Domain-specific helpers, library-like APIs
//! - **Kebab-case** (`function-name`) — Core primitives, standard operations
//!
//! When creating a new stdlib module, use namespaces if you have 3+ related functions
//! that form a cohesive API around a specific concept. See CLAUDE.md "Naming Conventions"
//! section for detailed guidance.

use crate::env::Environment;
use std::rc::Rc;

pub mod json;

/// Register all stdlib modules in the environment
pub fn register_stdlib(env: Rc<Environment>) {
    json::register(&env);
}

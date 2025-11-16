// ABOUTME: Error types for evaluation failures in the Lisp interpreter

use crate::value::Value;
use thiserror::Error;

// ===== Arity constant strings (eliminates allocations in error paths) =====
pub const ARITY_ONE: &str = "1";
pub const ARITY_TWO: &str = "2";
pub const ARITY_THREE: &str = "3";
pub const ARITY_AT_LEAST_ONE: &str = "at least 1";
pub const ARITY_ZERO_OR_ONE: &str = "0-1";
pub const ARITY_ONE_OR_TWO: &str = "1-2";
pub const ARITY_TWO_OR_THREE: &str = "2-3";

// ===== Common error message strings =====
pub const ERR_SANDBOX_NOT_INIT: &str = "Sandbox not initialized";

#[allow(dead_code)]
#[derive(Error, Debug, Clone)]
pub enum EvalError {
    // ===== Enhanced error variants with rich context =====
    /// Type mismatch error with function name, expected type, actual type, and position
    #[error("{function}: expected {expected}, got {actual} at argument {position}")]
    TypeMismatch {
        function: String,
        expected: String,
        actual: String,
        position: usize,
    },

    /// Arity error with function name, expected count/range, and actual count
    #[error("{function}: expected {expected} argument{}, got {actual}", if *.expected == "1" { "" } else { "s" })]
    ArityError {
        function: String,
        expected: String, // "2", "1-3", "at least 1"
        actual: usize,
    },

    /// Runtime error with function context
    #[error("{function}: {message}")]
    RuntimeError { function: String, message: String },

    // ===== Special error variants (non-contextual by nature) =====
    #[error("Undefined symbol: {0}")]
    UndefinedSymbol(String),

    #[error("Value is not callable")]
    NotCallable,
}

impl EvalError {
    /// Create a type mismatch error with full context
    pub fn type_error(function: &str, expected: &str, actual: &Value, position: usize) -> Self {
        EvalError::TypeMismatch {
            function: function.to_string(),
            expected: expected.to_string(),
            actual: actual.type_name(),
            position,
        }
    }

    /// Create an arity error with expected and actual counts
    pub fn arity_error(function: &str, expected: impl Into<String>, actual: usize) -> Self {
        EvalError::ArityError {
            function: function.to_string(),
            expected: expected.into(),
            actual,
        }
    }

    /// Create a runtime error with function context
    pub fn runtime_error(function: &str, message: impl Into<String>) -> Self {
        EvalError::RuntimeError {
            function: function.to_string(),
            message: message.into(),
        }
    }
}

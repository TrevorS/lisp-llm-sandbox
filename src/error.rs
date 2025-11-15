// ABOUTME: Error types for evaluation failures in the Lisp interpreter

use thiserror::Error;
use crate::value::Value;

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
        expected: String,  // "2", "1-3", "at least 1"
        actual: usize,
    },

    /// Runtime error with function context
    #[error("{function}: {message}")]
    RuntimeError {
        function: String,
        message: String,
    },

    // ===== Special error variants (non-contextual by nature) =====

    #[error("Undefined symbol: {0}")]
    UndefinedSymbol(String),

    #[error("Value is not callable")]
    NotCallable,

    #[error("I/O error: {0}")]
    IoError(String),
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

    /// Add function context to any error (for wrapping existing errors)
    pub fn in_function(self, _function: &str) -> Self {
        match self {
            // Already has context - don't double-wrap
            EvalError::TypeMismatch { .. }
            | EvalError::ArityError { .. }
            | EvalError::RuntimeError { .. } => self,

            // Don't wrap these - they're specific enough or don't need context
            EvalError::UndefinedSymbol(_) | EvalError::NotCallable | EvalError::IoError(_) => self,
        }
    }
}

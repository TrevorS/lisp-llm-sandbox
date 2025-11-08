// ABOUTME: Error types for evaluation failures in the Lisp interpreter

use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum EvalError {
    #[error("Undefined symbol: {0}")]
    UndefinedSymbol(String),

    #[error("Value is not callable")]
    NotCallable,

    #[error("Type error in operation")]
    TypeError,

    #[error("Arity mismatch: incorrect number of arguments")]
    ArityMismatch,

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("{0}")]
    Custom(String),
}

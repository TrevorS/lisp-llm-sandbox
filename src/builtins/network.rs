//! Network I/O operations: http-get, http-post
//!
//! Functions for HTTP network requests with capability-based sandboxing.
//!
//! - `http-get`: Make GET request to URL, return response body as string
//! - `http-post`: Make POST request to URL with data, return response body
//!
//! All requests are checked against a URL allowlist for safety

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;

use super::SANDBOX;

#[builtin(name = "http-get", category = "Network I/O", related(http-post))]
/// Performs an HTTP GET request and returns the response body as a string.
///
/// URL must be in allowed addresses list. Has 30 second timeout.
///
/// WARNING: DNS lookup cannot be interrupted, may hang if DNS is slow.
///
/// # Examples
///
/// ```lisp
/// (http-get "https://example.com") => "<html>..."
/// ```
///
/// # See Also
///
/// http-post
pub fn http_get(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    SANDBOX.with(|s| {
        let sandbox_ref = s.borrow();
        let sandbox = sandbox_ref
            .as_ref()
            .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

        sandbox
            .http_get(url)
            .map(Value::String)
            .map_err(|e| EvalError::IoError(e.to_string()))
    })
}

#[builtin(name = "http-post", category = "Network I/O", related(http-get))]
/// Performs an HTTP POST request and returns the response body as a string.
///
/// URL must be in allowed addresses. Sends body as plain text. 30 second timeout.
///
/// WARNING: DNS lookup cannot be interrupted, may hang if DNS is slow.
///
/// # Examples
///
/// ```lisp
/// (http-post "https://api.example.com" "data") => "response"
/// ```
///
/// # See Also
///
/// http-get
pub fn http_post(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let body = match &args[1] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    SANDBOX.with(|s| {
        let sandbox_ref = s.borrow();
        let sandbox = sandbox_ref
            .as_ref()
            .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

        sandbox
            .http_post(url, body)
            .map(Value::String)
            .map_err(|e| EvalError::IoError(e.to_string()))
    })
}

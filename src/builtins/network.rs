//! Network I/O operations: http-request
//!
//! Functions for HTTP network requests with capability-based sandboxing.
//!
//! - `http-request`: Flexible HTTP request with method, headers, body, and timeout options
//!   Returns structured map response with status, headers, and body
//!
//! All requests are checked against a URL allowlist for safety

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;
use std::collections::HashMap;

use super::SANDBOX;

#[builtin(name = "http-request", category = "Network I/O")]
/// Performs a flexible HTTP request with specified method and options.
///
/// URL must be in allowed addresses list. Options is a map with:
/// - :method - HTTP method as string ("GET", "POST", "PUT", "DELETE", "PATCH")
/// - :headers - Optional map of header name->value pairs
/// - :body - Optional request body as string
/// - :timeout - Optional timeout in milliseconds (default 30000)
///
/// Returns a map with :status, :headers, :body keys.
///
/// # Examples
///
/// ```lisp
/// (http-request "https://example.com" {:method "GET"})
/// (http-request "https://api.example.com" {:method "POST" :body "{...}" :timeout 5000})
/// ```
///
/// # See Also
///
/// http-get, http-post
pub fn http_request(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let url = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let options = match &args[1] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    // Extract method (required)
    let method = match options.get("method") {
        Some(Value::String(m)) => m.clone(),
        _ => {
            return Err(EvalError::Custom(
                "Missing or invalid :method in options".to_string(),
            ))
        }
    };

    // Extract optional headers map
    let headers = match options.get("headers") {
        Some(Value::Map(h)) => {
            let mut header_vec = Vec::new();
            for (k, v) in h.iter() {
                match v {
                    Value::String(val) => header_vec.push((k.clone(), val.clone())),
                    _ => {
                        return Err(EvalError::Custom(
                            "Header values must be strings".to_string(),
                        ))
                    }
                }
            }
            Some(header_vec)
        }
        None => None,
        _ => return Err(EvalError::Custom("Invalid :headers in options".to_string())),
    };

    // Extract optional body
    let body = match options.get("body") {
        Some(Value::String(b)) => Some(b.as_str()),
        None => None,
        _ => return Err(EvalError::Custom("Body must be a string".to_string())),
    };

    // Extract optional timeout
    let timeout = match options.get("timeout") {
        Some(Value::Number(t)) => Some(*t as u64),
        None => None,
        _ => return Err(EvalError::Custom("Timeout must be a number".to_string())),
    };

    SANDBOX.with(|s| {
        let sandbox_ref = s.borrow();
        let sandbox = sandbox_ref
            .as_ref()
            .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

        let response = sandbox
            .http_request(url, &method, headers, body, timeout)
            .map_err(|e| EvalError::IoError(e.to_string()))?;

        // Build response map
        let mut response_map = HashMap::new();
        response_map.insert("status".to_string(), Value::Number(response.status as f64));

        // Build headers map
        let mut headers_map = HashMap::new();
        for (k, v) in response.headers.iter() {
            headers_map.insert(k.clone(), Value::String(v.clone()));
        }
        response_map.insert("headers".to_string(), Value::Map(headers_map));

        response_map.insert("body".to_string(), Value::String(response.body));

        Ok(Value::Map(response_map))
    })
}

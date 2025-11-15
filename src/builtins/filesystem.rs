//! Filesystem I/O operations: read-file, write-file, file-exists?, file-size, list-files, file-stat
//!
//! Functions for safe file operations with capability-based sandboxing.
//!
//! - `read-file`: Read entire file contents as string
//! - `write-file`: Write string to file
//! - `file-exists?`: Check if file exists
//! - `file-size`: Get file size in bytes
//! - `list-files`: List files in directory
//! - `file-stat`: Get file metadata (size, type, timestamps, readonly)
//!
//! All operations are restricted to whitelisted paths via capability-based sandboxing

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;
use std::collections::HashMap;

use super::SANDBOX;

#[builtin(name = "read-file", category = "Filesystem I/O", related(write-file, file-exists?))]
/// Reads and returns the contents of a file as a string.
///
/// Path is relative to allowed sandbox directories.
///
/// # Examples
///
/// ```lisp
/// (read-file "data/input.txt") => "file contents"
/// ```
///
/// # See Also
///
/// write-file, file-exists?
pub fn read_file(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    SANDBOX.with(|s| {
        let sandbox_ref = s.borrow();
        let sandbox = sandbox_ref
            .as_ref()
            .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

        sandbox
            .read_file(path)
            .map(Value::String)
            .map_err(|e| EvalError::IoError(e.to_string()))
    })
}

#[builtin(name = "write-file", category = "Filesystem I/O", related(read-file, file-exists?))]
/// Writes contents to a file, creating it if it doesn't exist.
///
/// Returns #t on success. Path is relative to sandbox.
///
/// # Examples
///
/// ```lisp
/// (write-file "data/output.txt" "hello") => #t
/// ```
///
/// # See Also
///
/// read-file, file-exists?
pub fn write_file(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let contents = match &args[1] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    SANDBOX.with(|s| {
        let sandbox_ref = s.borrow();
        let sandbox = sandbox_ref
            .as_ref()
            .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

        sandbox
            .write_file(path, contents)
            .map(|_| Value::Bool(true))
            .map_err(|e| EvalError::IoError(e.to_string()))
    })
}

#[builtin(name = "file-exists?", category = "Filesystem I/O", related(file-size, read-file))]
/// Tests if a file exists and is accessible in sandbox.
///
/// Returns #t or #f.
///
/// # Examples
///
/// ```lisp
/// (file-exists? "data/file.txt") => #t
/// (file-exists? "nonexistent.txt") => #f
/// ```
///
/// # See Also
///
/// file-size, read-file
pub fn file_exists_q(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    SANDBOX.with(|s| {
        let sandbox_ref = s.borrow();
        let sandbox = sandbox_ref
            .as_ref()
            .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

        sandbox
            .file_exists(path)
            .map(Value::Bool)
            .map_err(|e| EvalError::IoError(e.to_string()))
    })
}

#[builtin(name = "file-size", category = "Filesystem I/O", related(file-exists?, read-file))]
/// Returns the size of a file in bytes.
///
/// Throws error if file doesn't exist.
///
/// # Examples
///
/// ```lisp
/// (file-size "data/file.txt") => 1024
/// ```
///
/// # See Also
///
/// file-exists?, read-file
pub fn file_size(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    SANDBOX.with(|s| {
        let sandbox_ref = s.borrow();
        let sandbox = sandbox_ref
            .as_ref()
            .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

        sandbox
            .file_size(path)
            .map(|size| Value::Number(size as f64))
            .map_err(|e| EvalError::IoError(e.to_string()))
    })
}

#[builtin(name = "list-files", category = "Filesystem I/O", related(file-exists?))]
/// Returns a list of filenames in a directory.
///
/// Does not include . or .., returns only names not full paths.
///
/// # Examples
///
/// ```lisp
/// (list-files "data") => ("file1.txt" "file2.txt")
/// ```
///
/// # See Also
///
/// file-exists?
pub fn list_files(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let dir = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    SANDBOX.with(|s| {
        let sandbox_ref = s.borrow();
        let sandbox = sandbox_ref
            .as_ref()
            .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

        sandbox
            .list_files(dir)
            .map(|files| Value::List(files.into_iter().map(Value::String).collect::<Vec<_>>()))
            .map_err(|e| EvalError::IoError(e.to_string()))
    })
}

#[builtin(name = "file-stat", category = "Filesystem I/O", related(file-exists?, file-size))]
/// Returns file metadata as a map with :size, :type, :modified, :accessed, :created, :readonly keys.
///
/// - :size - File size in bytes (number)
/// - :type - File type: "file", "directory", or "symlink" (string)
/// - :modified - Modification time in Unix seconds (number)
/// - :accessed - Last access time in Unix seconds (number)
/// - :created - Creation time in Unix seconds (number)
/// - :readonly - Read-only flag (boolean)
///
/// # Examples
///
/// ```lisp
/// (file-stat "data/file.txt") => {:size 1024 :type "file" :modified 1234567890 ...}
/// ```
///
/// # See Also
///
/// file-exists?, file-size
pub fn file_stat(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    SANDBOX.with(|s| {
        let sandbox_ref = s.borrow();
        let sandbox = sandbox_ref
            .as_ref()
            .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

        sandbox
            .file_stat(path)
            .map(|stat| {
                let mut result_map = HashMap::new();
                result_map.insert("size".to_string(), Value::Number(stat.size as f64));
                result_map.insert("type".to_string(), Value::String(stat.file_type));
                result_map.insert("modified".to_string(), Value::Number(stat.modified));
                result_map.insert("accessed".to_string(), Value::Number(stat.accessed));
                result_map.insert("created".to_string(), Value::Number(stat.created));
                result_map.insert("readonly".to_string(), Value::Bool(stat.readonly));
                Value::Map(result_map)
            })
            .map_err(|e| EvalError::IoError(e.to_string()))
    })
}

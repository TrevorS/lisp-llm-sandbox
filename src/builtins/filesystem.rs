//! Filesystem I/O operations: read-file, write-file, file-exists?, file-size, list-files
//!
//! Functions for safe file operations with capability-based sandboxing.
//!
//! - `read-file`: Read entire file contents as string
//! - `write-file`: Write string to file
//! - `file-exists?`: Check if file exists
//! - `file-size`: Get file size in bytes
//! - `list-files`: List files in directory
//!
//! All operations are restricted to whitelisted paths via capability-based sandboxing

use crate::env::Environment;
use crate::error::EvalError;
use crate::value::Value;
use std::rc::Rc;

use super::SANDBOX;

/// Reads and returns the contents of a file as a string
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

/// Writes contents to a file, creating it if it doesn't exist
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

/// Tests if a file exists and is accessible in sandbox
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

/// Returns the size of a file in bytes
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

/// Returns a list of filenames in a directory
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

/// Register all filesystem I/O builtins in the environment
pub fn register(env: &Rc<Environment>) {
    env.define("read-file".to_string(), Value::BuiltIn(read_file));
    env.define("write-file".to_string(), Value::BuiltIn(write_file));
    env.define("file-exists?".to_string(), Value::BuiltIn(file_exists_q));
    env.define("file-size".to_string(), Value::BuiltIn(file_size));
    env.define("list-files".to_string(), Value::BuiltIn(list_files));

    // Register help entries
    crate::help::register_help(crate::help::HelpEntry {
        name: "read-file".to_string(),
        signature: "(read-file path)".to_string(),
        description: "Reads and returns the contents of a file as a string.\nPath is relative to allowed sandbox directories.".to_string(),
        examples: vec!["(read-file \"data/input.txt\") => \"file contents\"".to_string()],
        related: vec!["write-file".to_string(), "file-exists?".to_string()],
        category: "Filesystem I/O".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "write-file".to_string(),
        signature: "(write-file path contents)".to_string(),
        description: "Writes contents to a file, creating it if it doesn't exist.\nReturns #t on success. Path is relative to sandbox.".to_string(),
        examples: vec!["(write-file \"data/output.txt\" \"hello\") => #t".to_string()],
        related: vec!["read-file".to_string(), "file-exists?".to_string()],
        category: "Filesystem I/O".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "file-exists?".to_string(),
        signature: "(file-exists? path)".to_string(),
        description: "Tests if a file exists and is accessible in sandbox.\nReturns #t or #f."
            .to_string(),
        examples: vec![
            "(file-exists? \"data/file.txt\") => #t".to_string(),
            "(file-exists? \"nonexistent.txt\") => #f".to_string(),
        ],
        related: vec!["file-size".to_string(), "read-file".to_string()],
        category: "Filesystem I/O".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "file-size".to_string(),
        signature: "(file-size path)".to_string(),
        description: "Returns the size of a file in bytes.\nThrows error if file doesn't exist."
            .to_string(),
        examples: vec!["(file-size \"data/file.txt\") => 1024".to_string()],
        related: vec!["file-exists?".to_string(), "read-file".to_string()],
        category: "Filesystem I/O".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "list-files".to_string(),
        signature: "(list-files directory)".to_string(),
        description: "Returns a list of filenames in a directory.\nDoes not include . or .., returns only names not full paths.".to_string(),
        examples: vec!["(list-files \"data\") => (\"file1.txt\" \"file2.txt\")".to_string()],
        related: vec!["file-exists?".to_string()],
        category: "Filesystem I/O".to_string(),
    });
}

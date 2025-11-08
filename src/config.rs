// ABOUTME: Configuration and constants for the Lisp interpreter
// This module contains version info, welcome messages, and I/O sandbox configuration

use std::path::PathBuf;

#[allow(dead_code)]
pub const VERSION: &str = "1.0.0";
pub const WELCOME_MESSAGE: &str = "Lisp Interpreter v1.0";
pub const WELCOME_SUBTITLE: &str = "A production-ready Scheme-flavored Lisp in Rust";

// ============================================================================
// I/O Sandboxing Configuration
// ============================================================================

/// Filesystem sandbox configuration
#[derive(Debug, Clone)]
pub struct FsConfig {
    pub allowed_paths: Vec<PathBuf>,
    pub max_file_size: usize,
}

impl Default for FsConfig {
    fn default() -> Self {
        Self {
            // Default allowed paths for file I/O
            allowed_paths: vec![
                PathBuf::from("./data"),
                PathBuf::from("./examples"),
                PathBuf::from("./scripts"),
            ],
            // Default max file size: 10MB
            max_file_size: 10 * 1024 * 1024,
        }
    }
}

/// Network sandbox configuration
#[derive(Debug, Clone, Default)]
pub struct NetConfig {
    /// Whether network I/O is enabled
    pub enabled: bool,
    /// Allowed network addresses (host:port format)
    /// Empty = no restrictions (if enabled=true)
    pub allowed_addresses: Vec<String>,
}

/// Combined I/O sandbox configuration
/// Reserved for future phases where full combined config builder is needed
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct IoConfig {
    pub filesystem: FsConfig,
    pub network: NetConfig,
}

#[allow(dead_code)]
pub const HELP_TEXT: &str = r#"
Available commands:
  (quit) or (exit)     - Exit the REPL
  (help)               - Show this help message
  (builtins)           - List all built-in functions
  (clear)              - Clear the screen

Type any Lisp expression to evaluate it. Use Ctrl-D or (quit) to exit.
Full documentation available at: https://github.com/anthropics/lisp-llm-sandbox
"#;

#[allow(dead_code)]
pub const BUILTINS_SUMMARY: &str = r#"
Built-in Functions (36 total):

Arithmetic:     + - * / %
Comparison:     = < > <= >=
Logic:          and or not
Lists:          cons car cdr list length empty?
Predicates:     number? string? list? nil? symbol? bool?

Console I/O:    print println
Filesystem:     read-file write-file file-exists? file-size list-files
Network:        http-get http-post

Error:          error error? error-msg
Control:        if begin let define lambda quote
Macros:         defmacro quasiquote unquote unquote-splicing

Type (help) for more information.
"#;

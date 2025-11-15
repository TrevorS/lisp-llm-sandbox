// ABOUTME: Configuration and constants for the Lisp interpreter
// This module contains version info, welcome messages, and I/O sandbox configuration

use std::path::PathBuf;

pub const VERSION: &str = "1.0.0";
pub const WELCOME_MESSAGE: &str = "Lisp Interpreter v1.0";
pub const WELCOME_SUBTITLE: &str = "A production-ready Scheme-flavored Lisp in Rust";

pub const WELCOME_FOOTER: &str = r#"
Quick Start Examples:
  (+ 1 2 3)                              → 6
  (map (lambda (x) (* x 2)) '(1 2 3))   → (2 4 6)
  (define (fib n) ...)                   → Define recursive function
  (http-request "https://api.example.com" {:method "GET"})  → HTTP with options

LLM-Optimized Features:
  • Maps with keywords: {:name "Alice" :age 30}
  • Structured I/O returns maps with metadata
  • 46 stdlib functions auto-loaded (map, filter, reduce, etc.)
  • Macros and compile-time code transformation
  • Tail-call optimization for deep recursion

Available Commands:
  (help)                    - Show all 130+ functions by category
  (help 'function-name)     - Detailed help for a specific function
  (quit) or (exit)          - Exit the REPL

Type (help) to see all available functions, or dive in with any Lisp expression!
"#;

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

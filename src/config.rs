// ABOUTME: Configuration and constants for the Lisp interpreter
// This module contains version info, welcome messages, and other user-facing text

#[allow(dead_code)]
pub const VERSION: &str = "1.0.0";
pub const WELCOME_MESSAGE: &str = "Lisp Interpreter v1.0";
pub const WELCOME_SUBTITLE: &str = "A production-ready Scheme-flavored Lisp in Rust";

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
Built-in Functions (29 total):

Arithmetic:     + - * / %
Comparison:     = < > <= >=
Logic:          and or not
Lists:          cons car cdr list length empty?
Predicates:     number? string? list? nil? symbol? bool?
I/O:            print println
Error:          error error? error-msg
Control:        if begin let define lambda quote
Macros:         defmacro quasiquote unquote unquote-splicing

Type (help) for more information.
"#;

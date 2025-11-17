//! # Lisp Interpreter Library
//!
//! A production-ready Scheme-flavored Lisp interpreter written in Rust with comprehensive
//! documentation, tail-call optimization, and sandboxed I/O capabilities.
//!
//! ## Features
//!
//! - **Complete Lisp Semantics**: Full support for special forms (define, lambda, if, begin, let, quote, quasiquote, defmacro)
//! - **Tail-Call Optimization**: Trampolining evaluator enabling unlimited recursion depth
//! - **Standard Library**: 27 built-in Lisp functions (map, filter, reduce, zip, factorial, etc.)
//! - **32 Built-in Functions**: Organized by category (arithmetic, comparison, logic, types, lists, console, filesystem, network, errors, help)
//! - **Comprehensive Help System**: Markdown-based documentation for all functions with examples
//! - **Capability-Based Sandboxing**: Safe filesystem and network I/O with whitelisting
//! - **Macro System**: Compile-time code transformation with proper quasiquoting
//! - **Interactive REPL**: Full readline support with history and syntax highlighting
//! - **Script Execution**: Run Lisp programs from files
//!
//! ## Quick Start
//!
//! ```lisp
//! ;; Define a function
//! (define (factorial n)
//!   (if (<= n 1) 1 (* n (factorial (- n 1)))))
//!
//! ;; Use higher-order functions
//! (define numbers '(1 2 3 4 5))
//! (map (lambda (x) (* x x)) numbers)  ; => (1 4 9 16 25)
//! (filter (lambda (x) (> x 2)) numbers)  ; => (3 4 5)
//! (reduce + 0 numbers)  ; => 15
//! ```
//!
//! ## Architecture
//!
//! ### Core Components
//!
//! - **[eval]**: Main evaluator with TCO via trampolining
//! - **[parser]**: S-expression parser using nom combinator library
//! - **[mod@env]**: Environment (scope) management with parent-chain lookup
//! - **[value]**: Core value types (Number, String, Symbol, List, Lambda, Macro, Error, BuiltIn)
//! - **[help]**: Help system with hybrid lookup (registry + environment)
//!
//! ### Built-in Functions (32 total)
//!
//! **Arithmetic** (5): +, -, *, /, %
//!
//! **Comparison** (5): =, <, >, <=, >=
//!
//! **Logic** (3): and, or, not
//!
//! **Type Predicates** (6): number?, string?, list?, nil?, symbol?, bool?
//!
//! **List Operations** (6): cons, car, cdr, list, length, empty?
//!
//! **Console I/O** (2): print, println
//!
//! **File I/O** (5): read-file, write-file, file-exists?, file-size, list-files
//!
//! **Network I/O** (1): http-request
//!
//! **Error Handling** (3): error, error?, error-msg
//!
//! **Help System** (2): help, doc
//!
//! ### Special Forms (8)
//!
//! - **define**: Variable and function definitions
//! - **lambda**: Anonymous functions with lexical closure
//! - **if**: Conditional evaluation with short-circuit behavior
//! - **begin**: Sequence multiple expressions
//! - **let**: Local variable bindings
//! - **quote**: Prevent evaluation of expressions
//! - **quasiquote**: Selective evaluation within templates
//! - **defmacro**: Compile-time code transformations
//!
//! ### Standard Library (27 functions)
//!
//! **Higher-order** (5): map, filter, reduce, compose, partial
//!
//! **List Utilities** (9): reverse, append, member, nth, last, take, drop, zip, reverse-helper
//!
//! **Predicates** (3): all, any, count
//!
//! **Sequences** (1): range
//!
//! **Math** (9): abs, min, max, square, cube, even?, odd?, sum, product, factorial
//!
//! ## Key Technical Details
//!
//! ### Tail-Call Optimization
//!
//! The evaluator uses a `Step` enum that returns either a computed value or another
//! expression to evaluate. This trampolining pattern avoids stack overflow for deeply
//! nested recursion. The main loop repeatedly evaluates steps until reaching a value.
//!
//! ### Lexical Closures
//!
//! Lambda functions capture their definition-time environment via `Rc<Environment>`.
//! This enables proper closure semantics with parent-chain lookup for variable resolution.
//!
//! ### Help System
//!
//! The help system uses a thread-local registry for built-in functions and a separate
//! thread-local environment for user-defined functions. The `help` function provides
//! comprehensive documentation including examples and related functions.
//!
//! ### Macro System
//!
//! Macros expand before evaluation. Unlike functions (which evaluate arguments first),
//! macros receive unevaluated arguments and return code to be evaluated. This enables
//! syntactic abstraction and domain-specific languages.
//!
//! ### Sandboxed I/O
//!
//! Using cap-std for capability-based security:
//! - Filesystem paths must be whitelisted
//! - File operations check allowed paths before access
//! - Network requests use a URL allowlist
//! - File size limits prevent resource exhaustion
//!
//! ## Error Handling
//!
//! Errors are catchable values, not exceptions. The `error` function creates an Error
//! value, `error?` checks for errors, and `error-msg` extracts messages. This enables
//! graceful error handling in Lisp code.

pub mod builtins;
pub mod config;
pub mod env;
pub mod error;
pub mod eval;
pub mod help;
pub mod macros;
pub mod parser;
pub mod sandbox;
pub mod stdlib;
pub mod stdlib_registry;
pub mod value;

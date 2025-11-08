# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a production-ready Lisp interpreter written in Rust (2024 edition) with ~7k lines of code across 28 source files. The project implements a complete language with parser, evaluator, standard library, REPL, macros, and sandboxed I/O capabilities. All 10 implementation phases are complete and thoroughly tested (260+ tests).

## Development Commands

### Build & Run
```bash
make build       # Debug build
make release     # Optimized release build
make run         # Run REPL with full I/O (files + network enabled)
cargo build      # Quick build
cargo run --release script.lisp  # Execute a script file
```

### Quality Assurance
```bash
make test        # Run all 260+ tests
make fmt         # Format code
make clippy      # Lint with clippy
make all         # Full pipeline: fmt → clippy → test → build → release
cargo test --all -- --nocapture  # Tests with output
cargo test test_name -- --exact   # Run specific test
```

### Common Development Tasks
```bash
cargo clippy --all-targets --all-features  # Full linting
cargo fmt --check                          # Check formatting without changes
cargo doc --no-deps --open                 # Generate & view documentation
```

## Architecture & Key Components

### Core Evaluator Loop (src/eval.rs)
The heart of the interpreter uses **trampolining for tail-call optimization** (TCO). The evaluator returns `Step` enum values that the main loop processes, enabling unlimited recursion depth without stack overflow. This is critical for performance.

### Environment & Scoping (src/env.rs)
- Uses `Rc<RefCell<Environment>>` for reference-counted, mutable scope chains
- Parent-chain lookup enables closures to capture lexical environment
- Each binding is stored in a HashMap at the current scope level
- Understand this thoroughly when adding new scoping features (let, lambda)

### Value System (src/value.rs)
The `Value` enum represents all Lisp types. Key types:
- **Lambda**: Captures environment + docstring (for help system)
- **Macro**: Similar to Lambda but for compile-time transformation
- **BuiltIn**: Rust function pointers for native implementation
- **Error**: Catchable error values (not exceptions)

When adding new features, determine if they belong as Values, builtins, or special forms.

### Parser (src/parser.rs)
Uses **nom parser combinators**. Key points:
- Handles S-expressions, comments, strings, numbers, symbols
- Composable combinator approach (easy to extend)
- Error messages include position information
- Numbers are always f64, strings support escapes

### Sandboxed I/O (src/sandbox.rs)
Uses **cap-std** for capability-based security:
- Filesystem access restricted to allowed paths (passed via CLI)
- Directory traversal attacks blocked at the OS level
- File size limits enforced (default 10MB)
- Network allowlist for HTTP requests
- HTTP client with timeout support

The sandbox is thread-local and must be configured at startup. When adding new I/O operations, use the sandbox trait.

### Help System (src/help.rs)
**Thread-local registry** of documentation for all 36 built-in functions. When adding new built-ins:
1. Define the function in `builtins.rs`
2. Add docstring entry in `builtins.rs::register_help()`
3. Support docstrings in Lambda definitions: `(define (f x) "docstring" body)`

### Standard Library (src/stdlib.lisp)
21 Lisp functions loaded at startup (unless `--no-stdlib` is used). Includes:
- Higher-order: `map`, `filter`, `reduce`, `compose`, `partial`
- List utilities: `reverse`, `append`, `member`, `nth`, `zip`
- Math: `abs`, `min`, `max`, `square`, `factorial`
- Predicates: `all`, `any`, `even?`, `odd?`

## Important Patterns & Constraints

### Tail-Call Optimization
The evaluator uses a `Step` enum that returns either a value or another expression to evaluate. **Never implement recursion without understanding TCO** - use the trampolining pattern (eval returns Step, not direct values).

### Reference Counting with Interior Mutability
The codebase heavily uses `Rc<RefCell<T>>` for shared, mutable state (especially environments). This avoids borrow checker issues but requires careful unwrapping:
```rust
let env = Rc::clone(&env);  // Clone the Rc, not the inner value
let mut bindings = env.borrow_mut();  // Get mutable access
```

### Special Forms vs Built-ins
**Special forms** (`define`, `lambda`, `if`, `begin`, `let`, `quote`, `defmacro`) are in `eval.rs` because they need special evaluation rules. **Built-in functions** (`+`, `map`, `print`) are in `builtins.rs` because they evaluate all arguments first.

When adding features, choose correctly - wrong choice breaks semantics.

### Error Handling
Errors are catchable values (not thrown). Functions return `Result<Value, EvalError>`. The `error` built-in creates an Error value, `error?` checks for it, `error-msg` extracts the message. This enables graceful error handling in Lisp code.

### Macros System (src/macros.rs)
Macros are defined with `defmacro` and expanded **before evaluation**. Key distinction:
- **Macros**: Receive unevaluated arguments, return code to be evaluated
- **Functions**: Receive evaluated arguments, return values

The macro registry is separate from the environment. When extending macro features, modify `macros.rs` and the `Macro` variant in `eval.rs`.

## Testing Strategy

The test suite is comprehensive with 260+ tests organized by concern:
- Unit tests for individual components (parser, env, builtins)
- Integration tests for language features (closures, TCO, macros)
- Stdlib tests for standard library functions
- Sandbox tests for I/O security

**When adding features**:
1. Write tests first (TDD approach)
2. Tests should verify both success and edge cases
3. For sandboxed operations, use the `sandbox::test_sandbox()` helper
4. Thread-local tests need `#[serial_test::serial]` attribute

## Dependencies & Key Crates

- **nom** (8.0.0) - Parser combinators (S-expression parsing)
- **rustyline** (17.0.2) - REPL with readline support and history
- **cap-std** (3.4.5) - Capability-based filesystem isolation
- **thiserror** (2.0.17) - Error macros for clean error types
- **ureq** (2.10.0) - HTTP client with timeout support
- **clap** (4.5.51) - CLI argument parsing (main.rs)
- **serial_test** (3.2.0) - Synchronization for thread-local tests

## Code Organization Principles

### Separation of Concerns
- `parser.rs` - Only parsing, no evaluation
- `eval.rs` - Only evaluation logic, uses parser as input
- `builtins.rs` - Only built-in implementations, no parser/evaluator logic
- `sandbox.rs` - Only I/O safety, isolated from evaluator
- `env.rs` - Only scope management, no parsing/evaluation

### File Responsibilities
- `value.rs` - Value type definition and Display impl
- `error.rs` - Error types
- `main.rs` - REPL, CLI parsing, I/O built-in registration
- `lib.rs` - Module exports only
- `config.rs` - Constants and configuration structures
- `tools.rs` - Tool trait for extensibility

### Adding New Features
1. Define new Value variant if needed (src/value.rs)
2. Add parsing support (src/parser.rs)
3. Add evaluation logic (src/eval.rs or src/builtins.rs)
4. Write tests immediately (tests/ directory)
5. Update help documentation if user-facing
6. Update stdlib.lisp if it's a Lisp-level function

## Common Gotchas

### 1. Quote/Quasiquote Semantics
Quasiquote (`) returns unevaluated code structure. Unquote (,) evaluates within quasiquotes. The macro system relies on this. Test thoroughly.

### 2. Macro Expansion Timing
Macros expand **before** evaluation. An unevaluated symbol in macro parameters is NOT a variable lookup. This confuses many macro implementations.

### 3. Environment Sharing
Closures capture their definition-time environment via `Rc<Environment>`. Mutations after definition don't affect captured env. This is correct behavior but different from mutable capture in some languages.

### 4. Float Precision
All numbers are f64. Integer operations may have precision loss for very large numbers. No arbitrary precision support currently.

### 5. Error vs Panic
The system uses catchable `Error` values, NOT panics. Panics break the REPL. Always return `Result::Err` for recoverable errors.

## Performance Considerations

- Startup time: ~10ms (stdlib.lisp loading)
- Simple expressions: <1ms
- Deep recursion: Unlimited with TCO
- Memory: Rc-based sharing is efficient

For performance-critical Lisp code, tail-recursive algorithms are preferred. The TCO mechanism is efficient because it reuses the same stack frame.

## Git Workflow

The main branch is `master`. Before making significant changes:
- Create a feature branch: `/feature-branch create`
- Commit with meaningful messages (include "why" not just "what")
- Run full validation: `make all`
- Ensure tests pass before pushing

## Resources

- README.md - User-facing documentation and examples
- examples/ - Complete working programs (factorial, fibonacci, sorting)
- tests/ - Test suite organization and patterns

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a production-ready Scheme-flavored Lisp interpreter written in Rust with ~8k lines of code across 24 source files. The project implements a complete language with parser, evaluator, standard library, REPL, macros, and sandboxed I/O capabilities. Features include:
- **213 tests** covering all major features (90 unit + 120 integration + 29 concurrency + 17 stdlib + 1 sandbox + 21 builtin + 25 string tests)
- **8 special forms** (define, lambda, if, begin, let, quote, quasiquote, defmacro)
- **81 built-in functions** organized into 11 categories (arithmetic, comparison, logic, types, lists, console, filesystem, network, errors, help, concurrency)
- **50 standard library functions** in pure Lisp organized into 6 modules (core, math, string, test, http, concurrency)
- **Complete help system** with markdown documentation for all 139 functions (8 special forms + 81 builtins + 50 stdlib)
- **Go-style channels** for concurrent programming with buffered/unbuffered channels
- **Markdown-rendered help** with syntax highlighting via termimad

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
make test        # Run all 213 tests
make fmt         # Format code with rustfmt
make clippy      # Lint with clippy (0 warnings)
make all         # Full pipeline: clean → fmt → clippy → test → build → release
cargo test --all -- --nocapture  # Tests with output
cargo test test_name -- --exact   # Run specific test
```

### Documentation
```bash
make docs                 # Generate and open rustdoc
cargo doc --no-deps      # Generate documentation locally
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
- **Number**: f64 numeric values
- **String**: Immutable string data
- **Symbol**: Identifiers like `foo` or `+`
- **Keyword**: Self-evaluating symbols like `:name` for map keys
- **Bool**: #t and #f boolean values
- **List**: Linked list of Values (cons cells)
- **Map**: HashMap<String, Value> for key-value data structures
- **Lambda**: Captures environment + docstring (for help system)
- **Macro**: Similar to Lambda but for compile-time transformation
- **BuiltIn**: Rust function pointers for native implementation
- **Channel**: Thread-safe communication channels (Arc-wrapped crossbeam channels)
- **Error**: Catchable error values (not exceptions)
- **Nil**: Empty list / null value

When adding new features, determine if they belong as Values, builtins, or special forms.

**Recent Additions**:
- **Map and Keyword types** enable structured data and named parameters (LLM-first design)
- **Channel type** enables Go-style concurrent programming with thread-safe message passing
- **Type predicates** added: `map?`, `keyword?`, and `channel?` join existing predicates like `number?`, `string?`, `list?`, `symbol?`, `bool?`, `nil?`

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

### Concurrency System (src/builtins/concurrency.rs)
Go-style **channels** for thread-safe message passing:
- **Channels**: Unbuffered and buffered (configurable capacity)
- **Thread-safe**: Uses crossbeam MPMC channels wrapped in Arc
- **Self-evaluating**: Channel values can be passed around like any other value
- **Operations**: `make-channel`, `channel-send`, `channel-recv`, `channel-close`, `channel?`

**Key Features:**
```lisp
;; Create channels
(define ch (make-channel))      ; Unbuffered
(define ch (make-channel 10))   ; Buffered (capacity 10)

;; Send and receive
(channel-send ch 42)
(channel-recv ch)  ; => 42

;; Channels can hold any value type
(channel-send ch {:name "Alice" :age 30})
(channel-send ch (list 1 2 3))
```

**Implementation Notes:**
- Channel value contains Arc<Sender> and Arc<Receiver> for thread-safety
- Uses crossbeam-channel for MPMC (multi-producer, multi-consumer) support
- Works with `spawn` and `spawn-link` for true concurrent programming

### Help System (src/help.rs)
**Thread-local registry** with markdown documentation for 139 total functions:
- **81 built-in functions**: Each in its own module under `src/builtins/` with category-specific help
  - New additions: `channel?`, `make-channel`, `channel-send`, `channel-recv`, `channel-close` (concurrency primitives)
  - Previous additions: `map?`, `keyword?` (type predicates), `http-request` (flexible HTTP), `file-stat` (file metadata)
- **8 special forms**: Registered in `eval.rs` via `register_special_forms_part1()` and `register_special_forms_part2()`
- **50 stdlib functions**: ;;; comment documentation in 6 modules under `src/stdlib/lisp/` with parameters, returns, complexity analysis, examples

**When adding new built-ins:**
1. Create function in appropriate `src/builtins/*.rs` category module
2. Register both binding and help entry in that module's `register()` function
3. For Lisp functions, add docstring as second parameter: `(define (f x) "docstring" body)`
4. Help entries use markdown with **Parameters**, **Returns**, **Time Complexity**, **Examples**, **Notes**

**Hybrid lookup system:**
- Registry-based for builtins (via thread-local HELP_REGISTRY)
- Environment-based for user-defined functions (via CURRENT_ENV)
- Users access via `(help)` for quick reference or `(help 'function-name)` for details

### Standard Library (src/stdlib/lisp/)
The standard library has been reorganized into 6 modules, loaded at startup (unless `--no-stdlib` is used):

**Core Functions (core.lisp)**:
- **Higher-order** (5): `map`, `filter`, `reduce`, `compose`, `partial`
- **List utilities** (9): `reverse`, `append`, `member`, `nth`, `last`, `take`, `drop`, `zip`, `reverse-helper`
- **Map helpers** (6): `map:query`, `map:select`, `map:update`, `map:filter`, `map:from-entries`, `map:map-values`

**Math Functions (math.lisp)**:
- **Basic** (5): `abs`, `min`, `max`, `square`, `cube`
- **Predicates** (2): `even?`, `odd?`
- **Aggregations** (3): `sum`, `product`, `factorial`
- **List predicates** (3): `all`, `any`, `count`
- **Sequence generation** (1): `range`

**String Functions (string.lisp)**:
- **Transformation** (4): `string-capitalize`, `string-concat`, `string-reverse`, `string-repeat`
- **Parsing** (2): `string-words`, `string-lines`
- **Padding** (1): `string-pad-left`

**Testing Framework (test.lisp)**:
- **Registration**: `define-test` (macro)
- **Test utilities**: `print-test-summary`, `print-test-details`
- **Improvements**: Testing framework now returns maps instead of alists

**HTTP Utilities (http.lisp)**:
- **Helpers** (3): `http:check-status`, `http:body`, `http:status`
- Build on new `http-request` builtin for flexible HTTP operations

**Concurrency Utilities (concurrency.lisp)**:
- **Higher-level patterns**: Channel-based concurrency helpers and utilities
- Build on `spawn`, `spawn-link`, and channel builtins for concurrent programming

Each function has ;;; comment documentation with Parameters, Returns, Time Complexity, Examples, and Notes sections.

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

The test suite has **213 tests** organized across 7 test suites:
- **Unit tests** (90): Parser, environment, error handling
- **Integration tests** (120): Language features, closures, TCO, macros
- **Concurrency tests** (29): Concurrent execution, channels, spawn
- **Stdlib tests** (17): Standard library functions
- **Sandbox tests** (1): I/O security and sandboxing
- **Builtin tests** (21): Individual builtin functions
- **String tests** (25): String manipulation and operations

**Test execution**:
- `make test` runs all suites
- `cargo test test_name -- --exact` runs specific test
- `cargo test --all -- --nocapture` shows println output

**When adding features**:
1. Write tests first (TDD approach)
2. Tests verify both success and edge cases
3. For sandboxed operations, use `sandbox::test_sandbox()` helper
4. Thread-local tests need `#[serial_test::serial]` attribute
5. All tests must pass before committing

## Dependencies & Key Crates

- **nom** (8.0.0) - Parser combinators (S-expression parsing)
- **rustyline** (17.0.2) - REPL with readline support and history
- **cap-std** (3.4.5) - Capability-based filesystem isolation
- **crossbeam-channel** (0.5) - Thread-safe MPMC channels for concurrency
- **thiserror** (2.0.17) - Error macros for clean error types
- **ureq** (2.10.0) - HTTP client with timeout support
- **clap** (4.5.51) - CLI argument parsing (main.rs)
- **serial_test** (3.2.0) - Synchronization for thread-local tests

## Code Organization Principles

### Naming Conventions: Namespace vs Kebab-Case

The project uses a **hybrid namespacing approach** inspired by Clojure:

#### When to Use Namespaces (`module:function`)

Use namespaces for **domain-specific helper modules** — cohesive sets of 3+ functions that:
- Work together on a specific concept or data type
- Form a library-like API
- Are not fundamental language primitives

**Examples:**
- `json:encode`, `json:decode`, `json:pretty` — JSON serialization module
- `http:body`, `http:status`, `http:check-status` — HTTP response helpers
- `map:query`, `map:select`, `map:update` — Advanced map utilities

#### When to Use Kebab-Case (`function-name`)

Use kebab-case for **core primitives and standard operations**:
- Language primitives: `+`, `-`, `if`, `define`, `lambda`
- Core data structure operations: `cons`, `car`, `cdr`, `map-get`, `map-set`
- Standard library essentials: `map`, `filter`, `reduce`
- Basic I/O operations: `print`, `read-file`, `http-request`

**Examples:**
- `http-request` — primitive network operation
- `map-get`, `map-set` — fundamental map operations
- `string-split`, `string-trim` — core string operations

#### Rationale

This convention provides:
1. **Familiarity** — Common operations stay short (`filter`, `map-get`)
2. **Organization** — Related functions are visually grouped (`json:*`, `http:*`)
3. **Clarity** — Namespaces signal "library module" vs "core primitive"
4. **Discoverability** — Tab-completion friendly for module exploration

#### Edge Cases

- **`map` vs `map:*`**: The higher-order `map` function (applies function to list elements) is distinct from `map:*` functions (operate on map data structures). This is intentional.
- **String functions**: All remain unnamespaced because they're considered core language operations, not a domain-specific module.

### Separation of Concerns
- `parser.rs` - Only parsing, no evaluation
- `eval.rs` - Only evaluation logic, uses parser as input; contains special forms + registration functions
- `builtins/` - 11 category modules + coordination (see structure below)
- `sandbox.rs` - Only I/O safety, isolated from evaluator
- `env.rs` - Only scope management, no parsing/evaluation

### Builtins Directory Structure (src/builtins/)
```
builtins/
├── mod.rs              # Coordination, calls all register functions
├── arithmetic.rs       # +, -, *, /, %
├── comparison.rs       # =, <, >, <=, >=
├── concurrency.rs      # make-channel, channel-send, channel-recv, channel-close, channel?
├── logic.rs            # and, or, not
├── types.rs            # number?, string?, list?, nil?, symbol?, bool?
├── lists.rs            # cons, car, cdr, list, length, empty?
├── console.rs          # print, println
├── filesystem.rs       # read-file, write-file, file-exists?, file-size, list-files
├── network.rs          # http-request
├── errors.rs           # error, error?, error-msg
└── help.rs             # help, doc
```

Each module has:
- Function implementations
- `register(env)` function that registers bindings + help entries
- Module-level doc comments (for cargo doc)

### Core File Responsibilities
- `value.rs` - All Value enum variants and Display impl
- `error.rs` - EvalError enum and error types
- `help.rs` - Help registry (thread-local), help formatting, hybrid lookup
- `main.rs` - REPL, CLI parsing, initialization sequence
- `lib.rs` - Module exports + crate-level documentation
- `config.rs` - Constants (VERSION, WELCOME_MESSAGE, etc.)
- `tools.rs` - Tool trait for extensibility
- `highlighter.rs` - Syntax highlighting for REPL output
- `macros.rs` - Macro expansion before evaluation

### Adding New Features
1. Define new Value variant if needed (src/value.rs)
2. Add parsing support (src/parser.rs)
3. Add evaluation logic (src/eval.rs or src/builtins.rs)
4. Write tests immediately (tests/ directory)
5. Update help documentation if user-facing
6. Update stdlib.lisp if it's a Lisp-level function

## Documentation System (Recently Implemented)

### Complete Help Coverage
The interpreter has comprehensive markdown documentation for 139 functions:
- **8 Special Forms**: define, lambda, if, begin, let, quote, quasiquote, defmacro (in eval.rs)
- **81 Built-in Functions**: Across 11 categories in src/builtins/ (including concurrency)
- **50 Stdlib Functions**: Pure Lisp functions in src/stdlib.lisp

### Help Entry Format
Each help entry contains:
```rust
HelpEntry {
    name: "function-name",
    signature: "(function-name arg1 arg2)",
    description: "Multi-line description with **bold**, bullet points, examples",
    examples: vec!["(function-name 1 2) => result"],
    related: vec!["other-function"],
    category: "Category Name",
}
```

### Stdlib Docstrings Format
Enhanced Lisp docstrings include sections:
```lisp
(define (map f lst)
  "Apply function to each element, returning new list.

**Parameters:**
- f: Function to apply
- lst: Input list

**Returns:** New list with f applied to each element

**Time Complexity:** O(n) where n is list length

**Examples:**
- (map (lambda (x) (* x 2)) '(1 2 3)) => (2 4 6)

**Notes:** Uses tail call optimization for efficiency.")
```

### How to Add Documentation
1. **For new special forms**: Add registration in eval.rs (before test module)
2. **For new builtins**: Create/edit appropriate src/builtins/*.rs file
3. **For stdlib functions**: Update src/stdlib.lisp docstring
4. Run `cargo doc --no-deps --open` to verify documentation renders correctly

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

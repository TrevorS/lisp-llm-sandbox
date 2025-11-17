# Lisp Interpreter in Rust

A complete, production-ready Lisp interpreter implemented in Rust with an interactive REPL, macros, tail-call optimization, and error handling.

## Features

### Core Language Features
- **Numbers**: 64-bit floating point
- **Booleans**: `#t` and `#f`
- **Strings**: Double-quoted text
- **Symbols**: Variable and function names
- **Keywords**: Self-evaluating identifiers `:name`, `:age`
- **Maps**: Key-value structures `{:name "Alice" :age 30}`
- **Lists**: S-expressions `(1 2 3)`
- **Nil**: Special empty/null value

### Special Forms
- `define` - Variable and function definition
- `lambda` - Anonymous functions with closures
- `if` - Conditional branching
- `begin` - Sequential execution
- `let` - Lexical scoping
- `quasiquote` (`) - Template creation
- `unquote` (,) - Template substitution
- `unquote-splicing` (,@) - List splicing
- `defmacro` - Macro definition

### Built-in Functions (47 total, organized by category)

**Arithmetic** (5): `+`, `-`, `*`, `/`, `%`

**Comparison** (5): `=`, `<`, `>`, `<=`, `>=`

**Logic** (3): `and`, `or`, `not`

**Type Predicates** (8): `number?`, `string?`, `list?`, `nil?`, `symbol?`, `bool?`, `map?`, `keyword?`

**List Operations** (6): `cons`, `car`, `cdr`, `list`, `length`, `empty?`

**Map Operations** (11): `map-new`, `map-get`, `map-set`, `map-has?`, `map-keys`, `map-values`, `map-entries`, `map-merge`, `map-remove`, `map-empty?`, `map-size`

**Console I/O** (2): `print`, `println`

**Filesystem I/O** (5): `read-file`, `write-file`, `file-exists?`, `file-size`, `list-files`

**Network I/O** (2): `http-get`, `http-post`

**Database I/O** (4): `db:open`, `db:close`, `db:exec`, `db:query`

**Error Handling** (3): `error`, `error?`, `error-msg`

**Help System** (2): `help`, `doc`

### Advanced Features
- **Closures**: Functions capture their lexical environment
- **Tail Call Optimization**: Deep recursion without stack overflow
- **Macros**: Compile-time code transformation
- **Error Handling**: Catchable error values
- **Interactive REPL**: Full readline support with history
- **Sandboxed I/O**: Safe filesystem and network access with capability-based security
- **SQLite Database**: Built-in database support with query builders and SQL injection protection
- **First-Class Help System**: Built-in help for all functions, extensible to user code
- **Function Docstrings**: Define functions with documentation: `(define (f x) "docs" body)`
- **Structured Data**: Maps with keywords for LLM-friendly data structures
- **JSON Support**: Built-in JSON encoding and decoding via stdlib modules

## Quick Start

### Installation
```bash
# Clone or navigate to the project
cd lisp-llm-sandbox

# Build in release mode
cargo build --release

# Run the REPL (with full I/O enabled)
make run

# Or run with custom I/O configuration
cargo run --release -- --fs-sandbox . --allow-network
```

### CLI Options
```bash
# Allow filesystem access to specific paths
cargo run --release -- --fs-sandbox ./data --fs-sandbox ./scripts

# Enable network access with optional allowlist
cargo run --release -- --allow-network

# Restrict network to specific domains
cargo run --release -- --allow-network --net-allow example.com --net-allow api.github.com

# Set maximum file size (default 10MB)
cargo run --release -- --max-file-size 5242880

# Execute a script file
cargo run --release script.lisp

# Skip standard library loading
cargo run --release -- --no-stdlib
```

### Your First Session
```lisp
lisp> (define x 42)
=> x

lisp> x
=> 42

lisp> (define (square n) (* n n))
=> square

lisp> (square x)
=> 1764

lisp> (if (< 10 20) "yes" "no")
=> "yes"

lisp> (quit)
Goodbye!
```

## REPL Commands

- `(quit)` or `(exit)` - Exit the interpreter
- `(clear)` - Clear the screen
- **Ctrl-C** - Interrupt current input
- **Ctrl-D** - Exit gracefully
- **Up/Down arrows** - Navigate command history

## Help System

The interpreter has a first-class help system:

- `(help)` - Show quick reference of all available functions
- `(help 'cons)` - Show detailed help for a specific function
- `(doc my-function)` - Extract docstring from any function
- Define functions with docstrings: `(define (square x) "Square a number" (* x x))`

## Examples

### Factorial (with TCO)
```lisp
(define (factorial n acc)
  (if (<= n 1)
      acc
      (factorial (- n 1) (* n acc))))

(factorial 5 1)  ; => 120
(factorial 100 1)  ; No stack overflow!
```

### Fibonacci
```lisp
(define (fib n)
  (if (<= n 1)
      n
      (+ (fib (- n 1)) (fib (- n 2)))))

(fib 10)  ; => 55
```

### Higher-Order Functions
```lisp
(define (make-adder n)
  (lambda (x) (+ x n)))

(define add5 (make-adder 5))
(add5 10)  ; => 15
```

### Macros
```lisp
(defmacro when (test body)
  `(if ,test ,body nil))

(when #t 99)  ; => 99
(when #f 99)  ; => nil

(defmacro unless (test body)
  `(if ,test nil ,body))

(unless #f 42)  ; => 42
```

### List Processing
```lisp
(define nums (list 1 2 3 4 5))
(car nums)  ; => 1
(cdr nums)  ; => (2 3 4 5)
(length nums)  ; => 5

(cons 0 nums)  ; => (0 1 2 3 4 5)
```

### Error Handling
```lisp
(define result (error "something went wrong"))
(error? result)  ; => #t
(error-msg result)  ; => "something went wrong"
```

### Maps and Keywords (Structured Data)
```lisp
; Create a map with keywords as keys
(define person {:name "Alice" :age 30 :city "NYC"})
(println person)  ; => {:age 30 :city "NYC" :name "Alice"}

; Get values by keyword
(map-get person :name)  ; => "Alice"
(map-get person :missing "unknown")  ; => "unknown" (with default)

; Create new map with updated value (immutable)
(define older-person (map-set person :age 31))
(map-get person :age)  ; => 30 (original unchanged)
(map-get older-person :age)  ; => 31

; Map operations
(map-keys person)  ; => (:age :city :name)
(map-values person)  ; => (30 "NYC" "Alice")
(map-size person)  ; => 3
(map-has? person :name)  ; => #t

; Merge maps
(define extra {:country "USA" :active #t})
(define merged (map-merge person extra))
; merged => {:active #t :age 30 :city "NYC" :country "USA" :name "Alice"}
```

### JSON Encoding and Decoding
```lisp
; Encode Lisp values to JSON
(define person {:name "Bob" :scores '(90 85 95)})
(define json-str (json:encode person))
; json-str => "{\"name\":\"Bob\",\"scores\":[90,85,95]}"

; Decode JSON back to Lisp values
(define decoded (json:decode json-str))
(map-get decoded :name)  ; => "Bob"

; Pretty-print JSON with indentation
(println (json:pretty json-str))
```

### Sandboxed File I/O
```lisp
; Write a file
(write-file "data/greeting.txt" "Hello, Lisp!")  ; => #t

; Read a file
(read-file "data/greeting.txt")  ; => "Hello, Lisp!"

; Check if file exists
(file-exists? "data/greeting.txt")  ; => #t

; Get file size
(file-size "data/greeting.txt")  ; => 13

; List files in directory
(list-files "data")  ; => ("greeting.txt" ...)
```

### Network I/O
```lisp
; Make HTTP GET request
(http-get "https://example.com")  ; => HTML response body

; Make HTTP POST request
(http-post "https://api.example.com/data" "request body")  ; => response
```

### Database Operations (SQLite)
```lisp
; Connect to database
(define conn (db:connect (sqlite:spec "users.db")))

; Create table
(db:execute conn "CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)" '())

; Insert rows using query builder (maps to SQL)
(db:insert conn "users" {:id 1 :name "Alice" :age 30})
(db:insert conn "users" {:id 2 :name "Bob" :age 25})
(db:insert conn "users" {:id 3 :name "Charlie" :age 30})
; => 1 (rows affected)

; Find rows with WHERE conditions
(db:find conn "users" "*" {:age 30})
; => ({:id 1 :name "Alice" :age 30} {:id 3 :name "Charlie" :age 30})

; Update rows
(db:update conn "users" {:age 31} {:name "Alice"})
; => 1 (rows affected)

; Count rows
(db:count conn "users" {:age 30})
; => 1 (Charlie remains age 30)

; Delete rows
(db:delete conn "users" {:id 2})
; => 1 (rows affected)

; Raw SQL queries (with parameterization for security)
(db:query conn "SELECT name FROM users WHERE age > ?" '(25))
; => ({:name "Alice"} {:name "Charlie"})

; Always close connection to prevent resource leaks
(db:close conn)
; => #t
```

### Help System
```lisp
; Get quick reference
(help)  ; => Lists all 36 functions

; Get detailed help
(help 'map)  ; => Detailed documentation for map

; Get docstring from user function
(define (double x) "Double a number" (* 2 x))
(doc double)  ; => "Double a number"
```

## Project Structure

```
lisp-llm-sandbox/
├── src/
│   ├── main.rs              - REPL implementation, I/O built-in registration
│   ├── lib.rs               - Library exports
│   ├── value.rs             - Value type definitions (with Keywords and Maps)
│   ├── error.rs             - Error types
│   ├── parser.rs            - S-expression parser (nom-based, supports :keywords and {:map})
│   ├── env.rs               - Environment/scope management
│   ├── eval.rs              - Evaluator with TCO
│   ├── builtins/
│   │   ├── mod.rs           - Builtin module coordination
│   │   ├── arithmetic.rs    - Math operators
│   │   ├── comparison.rs    - Comparison operators
│   │   ├── logic.rs         - Boolean logic
│   │   ├── types.rs         - Type predicates and checks
│   │   ├── lists.rs         - List operations
│   │   ├── maps.rs          - Map operations (NEW)
│   │   ├── console.rs       - I/O functions
│   │   ├── filesystem.rs    - File operations
│   │   ├── network.rs       - HTTP operations
│   │   ├── testing.rs       - Testing utilities
│   │   ├── errors.rs        - Error handling
│   │   └── help.rs          - Help system
│   ├── stdlib/
│   │   ├── mod.rs           - Stdlib module coordination
│   │   ├── json.rs          - JSON encoding/decoding (NEW)
│   │   └── lisp/
│   │       ├── core.lisp    - Higher-order functions and list utilities
│   │       ├── math.lisp    - Math functions
│   │       ├── string.lisp  - String operations
│   │       ├── test.lisp    - Testing framework
│   │       └── http.lisp    - HTTP utilities
│   ├── macros.rs            - Macro system
│   ├── tools.rs             - Tool trait for extensibility
│   ├── help.rs              - Help documentation system
│   ├── sandbox.rs           - Sandboxed I/O with cap-std
│   ├── config.rs            - Configuration and constants
│   ├── highlighter.rs       - REPL syntax highlighting
│   ├── stdlib_registry.rs   - Stdlib function documentation registry
│   └── env.rs               - Environment/scope management
├── tests/
│   ├── integration_test.rs  - Complete integration tests
│   ├── stdlib_tests.rs      - Standard library tests
│   ├── builtins_test.rs     - Builtin function tests
│   └── string_tests.rs      - String manipulation tests
├── examples/
│   ├── quicksort.lisp           - Sorting algorithm
│   ├── factorial.lisp           - Recursive functions
│   ├── fibonacci.lisp           - Multiple implementations
│   ├── functional_programming.lisp - FP patterns
│   ├── data_processing.lisp     - Data analysis
│   └── maps_and_json.lisp       - Maps and JSON usage (NEW)
├── Cargo.toml               - Dependencies and metadata
└── README.md                - This file
```

## Architecture

### Parser (nom-based)
- Combinator-style parsing
- Handles all Lisp syntax
- Comment support
- Proper error messages

### Evaluator
- Tail-call optimization via trampolining
- Environment chains for lexical scoping
- Special form handling
- Macro expansion before evaluation
- Docstring extraction from function definitions

### Environment
- Reference-counted (Rc) for sharing
- Parent-chain lookup for closures
- Interior mutability (RefCell) for bindings

### Sandboxed I/O (cap-std based)
- Capability-based filesystem security
- Prevents directory traversal attacks
- Configurable allowed paths
- File size limits
- Network address allowlist
- HTTP request timeout support

### Help System
- Thread-local help registry
- Comprehensive documentation for 36 built-in functions
- Extensible docstring support for user functions
- Pretty-printed help output with ASCII formatting

### Value Types
```rust
enum Value {
    Number(f64),
    Bool(bool),
    Symbol(String),
    String(String),
    Keyword(String),                          // Self-evaluating keywords (:name)
    List(Vec<Value>),
    Map(HashMap<String, Value>),              // Key-value structures
    Lambda { params, body, env, docstring },  // Docstring support
    Macro { params, body },
    BuiltIn(fn(&[Value]) -> Result<Value, EvalError>),
    Error(String),
    Nil,
}
```

## Testing

### Run All Tests
```bash
cargo test
```

**Current Status**: 270+ tests passing ✓

### Test Coverage
- Unit tests (88 tests)
- Integration tests (118 tests)
- Standard library tests (17 tests)
- Sandbox I/O tests (1 test)
- Builtin function tests (21 tests)
- String manipulation tests (25 tests)

### Code Quality

All code passes quality checks:
- ✅ `cargo clippy --all-targets` - Zero warnings
- ✅ `cargo fmt --check` - Properly formatted
- ✅ `cargo test` - 270 tests passing
- ✅ `cargo build --release` - Clean build

## Dependencies

- **nom** (8.0.0) - Parser combinators
- **rustyline** (17.0.2) - Interactive REPL with readline support
- **thiserror** (2.0.17) - Error handling
- **cap-std** (3.4.5) - Capability-based filesystem sandboxing
- **ureq** (2.10.0) - HTTP client with timeout support
- **clap** (4.5.51) - CLI argument parsing
- **serial_test** (3.2.0) - Test synchronization
- **serde** (1.0) - Serialization framework
- **serde_json** (1.0) - JSON encoding and decoding
- **termimad** (0.28) - Markdown rendering in terminal

## Implementation Phases

- ✅ **Phase 1**: Value types and printing
- ✅ **Phase 2**: Parser with nom
- ✅ **Phase 3**: Environment and variable lookup
- ✅ **Phase 4**: Evaluator and built-in functions
- ✅ **Phase 5**: Lambda, closures, and control flow
- ✅ **Phase 6**: Macros and quasiquoting
- ✅ **Phase 7**: Tail-call optimization
- ✅ **Phase 8**: REPL and tool system
- ✅ **Phase 9**: Standard library (21 functions)
- ✅ **Phase 10**: Integration testing & production polish

## Performance

- **Startup**: ~10ms
- **Simple expressions**: <1ms
- **Deep recursion**: TCO enables unlimited depth
- **Memory**: Efficient Rc-based sharing

## Standard Library (Lisp and Rust Modules)

The interpreter includes a comprehensive standard library with 46+ functions organized into focused modules:

### Core Library (core.lisp)
**Higher-Order Functions** (5): `map`, `filter`, `reduce`, `compose`, `partial`

**List Utilities** (9): `reverse`, `append`, `member`, `nth`, `last`, `take`, `drop`, `zip`, `reverse-helper`

**Map Helpers** (6): `map:query`, `map:select`, `map:update`, `map:filter`, `map:from-entries`, `map:map-values`

### Math Library (math.lisp)
**Basic** (5): `abs`, `min`, `max`, `square`, `cube`

**Predicates** (2): `even?`, `odd?`

**Aggregations** (3): `sum`, `product`, `factorial`

**List Predicates** (3): `all`, `any`, `count`

**Sequence Generation** (1): `range`

### String Library (string.lisp)
**Transformation** (4): `string-capitalize`, `string-concat`, `string-reverse`, `string-repeat`

**Parsing** (2): `string-words`, `string-lines`

**Padding** (1): `string-pad-left`

### Testing Library (test.lisp)
**Registration**: `define-test` (macro for defining tests)

**Utilities** (3): `print-test-summary`, `print-test-details`, `run-tests`

### HTTP Library (http.lisp)
**Helpers** (3): `http:check-status`, `http:body`, `http:status`

### JSON Module (json.rs, Rust-native)
**Encoding**: `json:encode` - Convert Lisp values to JSON strings

**Decoding**: `json:decode` - Parse JSON strings to Lisp values

**Formatting**: `json:pretty` - Pretty-print JSON with indentation

## Example Programs

The `examples/` directory contains complete programs demonstrating:

1. **quicksort.lisp** - Quicksort algorithm implementation
2. **factorial.lisp** - Recursive and tail-recursive factorial
3. **fibonacci.lisp** - Multiple Fibonacci implementations
4. **functional_programming.lisp** - FP patterns (map, filter, compose)
5. **data_processing.lisp** - Statistical analysis and transformations

Run examples:
```bash
cargo run --release < examples/factorial.lisp
cargo run --release < examples/functional_programming.lisp
```

## Future Enhancements

- [ ] String manipulation functions (split, join, substring)
- [ ] Module system for code organization
- [ ] Syntax highlighting in REPL
- [ ] Auto-completion for built-in functions
- [ ] Debugger/stepper with breakpoints
- [ ] WASM compilation target
- [ ] Concurrent/parallel evaluation
- [ ] HTTP response status codes and headers
- [ ] Custom DNS resolver for network requests
- [ ] File permission controls
- [ ] Process execution (shell commands)

## Contributing

This is a learning project implementing a Lisp interpreter from scratch. Feel free to explore, learn, and experiment!

## License

Educational/Learning project - feel free to use and modify.

## Author

Built with Rust 2024 edition following modern best practices.

---

**Happy Lisping! 🎉**

For detailed implementation notes, see [PHASE8_SUMMARY.md](PHASE8_SUMMARY.md).

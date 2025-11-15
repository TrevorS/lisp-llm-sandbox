# Legacy Error Variants - Detailed Inventory

## src/eval.rs (26 total: 24 Custom, 2 ArityMismatch)

### Custom Errors (24 total)
All related to special forms that need precise argument counts and structure:

**Quote/Quasiquote:**
- Line 74: `quote: expected 1 argument`
- Line 80-81: `quasiquote: expected 1 argument`
- Line 406: `unquote: expected 1 argument`
- Line 414: `quasiquote: expected 1 argument`
- Line 429, 442: Nested quasiquote/unquote errors

**If Form:**
- Line 92: `if: expected 2 or 3 arguments`

**Define Form:**
- Line 203: `define: name must be a symbol`
- Line 222: `define: expected name and body`
- Line 234, 284: `define: invalid format` variants
- Line 294: `define: value expression`
- Line 307, 320: `define: invalid binding in pair` variants

**Let Form:**
- Line 348: `let: expected bindings and body`
- Line 353: `let: bindings must be a list`
- Line 365: `let: binding name must be symbol`
- Line 371: `let: binding must be [name value] pair`

**Defmacro Form:**
- Line 485: `defmacro: expected name and params`
- Line 492: `defmacro: name must be a symbol`
- Line 500: `defmacro: expected name, params, and body`
- Line 505: `defmacro: params must be a list`

### ArityMismatch (2 total)
- Line 153: In eval_function - checking if macro matches arity
- Line 536: In expand_macros - checking if macro matches arity
- Line 952 (TEST): Test assertion checking ArityMismatch

---

## src/tools.rs (2 total: 1 ArityMismatch, 1 TypeError)

- ArityMismatch: Checking argument count in Tool trait implementation
- TypeError: Checking argument type in Tool trait implementation

---

## src/builtins/maps.rs (20 total: 10 ArityMismatch, 10 TypeError)

### ArityMismatch (10 total)
- map-new: Line expects 0 args
- map-get: Lines expect 2-3 args
- map-set: Lines expect 3 args
- map-has?: Lines expect 2 args
- map-keys: Lines expect 1 arg
- map-values: Lines expect 1 arg
- map-entries: Lines expect 1 arg
- map-merge: Lines expect 2 args
- map-remove: Lines expect 2 args
- map-size: Lines expect 1 arg

### TypeError (10 total)
All validating argument types:
- Checking args[0] is Value::Map
- Checking args[1] is Value::Keyword
- Checking combined map/keyword argument types

---

## src/builtins/lists.rs (11 total: 4 ArityMismatch, 5 TypeError, 2 Custom)

### ArityMismatch (4 total)
- cons: Expects 2 args
- car: Expects 1 arg
- cdr: Expects 1 arg
- list: Expects variable args validation

### TypeError (5 total)
- cons: Checking args[1] is List or Nil
- car: Checking args[0] is List
- cdr: Checking args[0] is List
- length: Checking is List
- empty?: Checking is List or Nil

### Custom (2 total)
- Line 70: "car of empty list"
- Line 87: "cdr of empty list"

---

## src/builtins/comparison.rs (12 total: 6 ArityMismatch, 6 TypeError)

### ArityMismatch (6 total)
Validating argument counts for:
- =, <, >, <=, >=, equal?

### TypeError (6 total)
Validating operand types (must be comparable):
- =, <, >, <=, >=, equal?

---

## src/builtins/help.rs (4 total: 2 ArityMismatch, 1 TypeError, 1 Custom)

### ArityMismatch (2 total)
- help: Expects 0 or 1 args
- doc: Expects 1 arg

### TypeError (1 total)
- help: Type checking on argument

### Custom (1 total)
- help: "No help found for 'symbol-name'"

---

## src/builtins/network.rs (7 total: 1 ArityMismatch, 2 TypeError, 4 Custom)

### ArityMismatch (1 total)
- http-request: Expects 1-2 args

### TypeError (2 total)
- http-request: Checking method and URL types

### Custom (4 total)
- Invalid :headers in options
- Body must be a string
- Timeout must be a number
- HTTP request failure context

---

## src/builtins/errors.rs (4 total: 3 ArityMismatch, 1 TypeError)

### ArityMismatch (3 total)
- error: Expects 1 arg
- error?: Expects 1 arg
- error-msg: Expects 1 arg

### TypeError (1 total)
- error-msg: Type validation on argument

---

## src/builtins/types.rs (8 total: 8 ArityMismatch)

All type predicates expect exactly 1 argument:
- number?: ArityMismatch
- string?: ArityMismatch
- bool?: ArityMismatch
- list?: ArityMismatch
- nil?: ArityMismatch
- symbol?: ArityMismatch
- map?: ArityMismatch
- keyword?: ArityMismatch

---

## src/builtins/testing.rs (7 total: 5 ArityMismatch, 1 TypeError, 1 Custom)

### ArityMismatch (5 total)
- test-equal: Expects 2 args
- print-test-summary: Expects 0 args
- print-test-details: Expects 0 args
- run-test: Expects 1 arg (appears twice)

### TypeError (1 total)
- run-test: Type validation

### Custom (1 total)
- run-test: "Test must be a lambda"

---

## src/builtins/logic.rs (3 total: 1 ArityMismatch, 2 TypeError)

### ArityMismatch (1 total)
- not: Expects 1 arg

### TypeError (2 total)
- and: Type checking on arguments
- or: Type checking on arguments

---

## src/builtins/filesystem.rs (12 total: 6 ArityMismatch, 6 TypeError)

### ArityMismatch (6 total)
- read-file: Expects 1 arg
- write-file: Expects 2 args
- file-exists?: Expects 1 arg
- file-size: Expects 1 arg
- list-files: Expects 1 arg (appears multiple times)

### TypeError (6 total)
- Same functions: Validating string arguments for file paths

---

## src/builtins/strings.rs (40 total: 12 ArityMismatch, 27 TypeError, 1 Custom)

### ArityMismatch (12 total)
- string-length: Expects 1 arg
- string-concat: Expects 2 args
- string-append: Expects 2 args
- string-index: Expects 2 args
- string-slice: Expects 2-3 args
- string-split: Expects 2 args (multiple)
- string-trim: Expects 1 arg
- string-upcase: Expects 1 arg
- string-downcase: Expects 1 arg
- string-replace: Expects 3 args
- string-substring: Expects 3 args

### TypeError (27 total)
All validating string/number argument types:
- Checking string arguments
- Checking number arguments for indices
- Checking mixed types

### Custom (1 total)
- string-split: Format error message with actual count

---

## src/stdlib/json.rs (7 total: 3 ArityMismatch, 1 TypeError, 3 Custom)

### ArityMismatch (3 total)
- json:encode: Expects 1 arg
- json:decode: Expects 1 arg
- json:pretty: Expects 1 arg

### TypeError (1 total)
- json:decode: Type validation

### Custom (3 total)
- JSON serialization errors (from serde_json)
- JSON parsing errors (from serde_json)
- JSON pretty-print errors

---

## REPLACEMENT STRATEGY

### ArityMismatch (72 total)
Replace with: `EvalError::arity_error(function_name, "expected_count", actual_count)`
- Most straightforward: just need function name and expected/actual counts
- Example: `Err(EvalError::arity_error("cons", "2", args.len()))`

### TypeError (66 total)
Replace with: `EvalError::type_error(function_name, expected_type, &value, position)`
- Requires: function name, expected type string, actual value, argument position
- Example: `Err(EvalError::type_error("car", "list", &args[0], 1))`

### Custom (40 total)
Replace with: `EvalError::runtime_error(function_name, message)`
- For simple errors: Use runtime_error helper
- For complex messages with context: May need creative formatting
- Example: `Err(EvalError::runtime_error("quote", "expected 1 argument"))`

---

## TOTAL EFFORT ESTIMATE

- **Files to update:** 14 (eval.rs, tools.rs, 12 builtins/*.rs, json.rs)
- **Errors to replace:** 178 total
- **Complexity:** Low-Medium
  - ArityMismatch: Straightforward (just add function name and count)
  - TypeError: Moderate (need to identify position and expected type)
  - Custom: Varies (some are arity/type errors in disguise, some are genuine runtime errors)


# Error Replacement Checklist - Systematic Replacement Guide

This checklist helps you systematically replace all 178 legacy error usages.

---

## File-by-File Replacement Checklist

### 1. src/eval.rs (26 changes required)
- [ ] **24 Custom errors** - Special forms (quote, if, define, let, defmacro, quasiquote)
  - [ ] Lines 74, 80-81: quote/quasiquote errors
  - [ ] Line 92: if argument count
  - [ ] Lines 203, 222, 234, 284, 294, 307, 320: define errors
  - [ ] Lines 348, 353, 365, 371: let binding errors
  - [ ] Lines 406, 414: unquote/quasiquote errors in expansion
  - [ ] Lines 429, 442: nested quasiquote/unquote errors
  - [ ] Lines 485, 492, 500, 505: defmacro errors

- [ ] **2 ArityMismatch** - Macro arity checks
  - [ ] Line 153: eval_function macro arity
  - [ ] Line 536: expand_macros macro arity
  - [ ] Line 952: TEST - update assert for ArityMismatch pattern

**Estimated time:** 20-25 minutes
**Difficulty:** Low (all follow runtime_error or arity_error pattern)

---

### 2. src/tools.rs (2 changes required)
- [ ] **1 ArityMismatch** - Tool trait implementation
- [ ] **1 TypeError** - Tool trait type check

**Estimated time:** 2-3 minutes
**Difficulty:** Low (straightforward replacements)

---

### 3. src/builtins/types.rs (8 changes required)
- [ ] **8 ArityMismatch** - All type predicates
  - [ ] number?: 1 argument
  - [ ] string?: 1 argument
  - [ ] bool?: 1 argument
  - [ ] list?: 1 argument
  - [ ] nil?: 1 argument
  - [ ] symbol?: 1 argument
  - [ ] map?: 1 argument
  - [ ] keyword?: 1 argument

**Pattern:** `Err(EvalError::arity_error("predicate-name", "1", args.len()))`

**Estimated time:** 5 minutes
**Difficulty:** Low (identical pattern repeated 8 times)

---

### 4. src/builtins/lists.rs (11 changes required)
- [ ] **4 ArityMismatch** - cons, car, cdr, list
  - [ ] cons: expects 2 args
  - [ ] car: expects 1 arg
  - [ ] cdr: expects 1 arg
  - [ ] list: variable args validation

- [ ] **5 TypeError** - Type checks
  - [ ] cons: arg[1] must be list or nil
  - [ ] car: arg[0] must be list
  - [ ] cdr: arg[0] must be list
  - [ ] length: arg[0] must be list
  - [ ] empty?: arg[0] must be list or nil

- [ ] **2 Custom** - Runtime errors
  - [ ] "car of empty list" → runtime_error
  - [ ] "cdr of empty list" → runtime_error

**Estimated time:** 10 minutes
**Difficulty:** Low-Medium (straightforward patterns)

---

### 5. src/builtins/comparison.rs (12 changes required)
- [ ] **6 ArityMismatch** - =, <, >, <=, >=, equal?
- [ ] **6 TypeError** - Type validation for same operators

**Pattern:**
- ArityMismatch: `Err(EvalError::arity_error("operator", "2", args.len()))`
- TypeError: `Err(EvalError::type_error("operator", "comparable type", &args[0], 1))`

**Estimated time:** 8 minutes
**Difficulty:** Low (repetitive pattern)

---

### 6. src/builtins/help.rs (4 changes required)
- [ ] **2 ArityMismatch** - help, doc functions
- [ ] **1 TypeError** - help function type check
- [ ] **1 Custom** - "No help found" message

**Pattern:**
- ArityMismatch: `arity_error("help" or "doc", expected, args.len())`
- TypeError: `type_error("help", expected_type, &arg, position)`
- Custom: `runtime_error("help", "no help found for 'name'")`

**Estimated time:** 5 minutes
**Difficulty:** Low

---

### 7. src/builtins/network.rs (7 changes required)
- [ ] **1 ArityMismatch** - http-request argument count
- [ ] **2 TypeError** - http-request type checks (method, URL)
- [ ] **4 Custom** - HTTP-specific errors
  - [ ] "Invalid :headers in options"
  - [ ] "Body must be a string"
  - [ ] "Timeout must be a number"
  - [ ] HTTP request failure with context

**Pattern:** All Custom → runtime_error("http-request", message)

**Estimated time:** 5 minutes
**Difficulty:** Low

---

### 8. src/builtins/errors.rs (4 changes required)
- [ ] **3 ArityMismatch** - error, error?, error-msg
  - [ ] error: 1 argument
  - [ ] error?: 1 argument
  - [ ] error-msg: 1 argument

- [ ] **1 TypeError** - error-msg type check

**Estimated time:** 3 minutes
**Difficulty:** Low

---

### 9. src/builtins/filesystem.rs (12 changes required)
- [ ] **6 ArityMismatch** - read-file, write-file, file-exists?, file-size, list-files
- [ ] **6 TypeError** - Same functions type checks (string arguments)

**Pattern:** Similar to other builtins with uniform arity/type checks

**Estimated time:** 10 minutes
**Difficulty:** Low

---

### 10. src/builtins/strings.rs (40 changes - LARGEST FILE)
- [ ] **12 ArityMismatch** - string-length, string-concat, string-append, string-index, string-slice, string-split, string-trim, string-upcase, string-downcase, string-replace, string-substring

- [ ] **27 TypeError** - Type checks on all string functions (checking for string/number args)

- [ ] **1 Custom** - string-split format error message

**Strategy:** 
1. Start with ArityMismatch (12) - most straightforward
2. Then TypeError (27) - follow existing patterns
3. Last, Custom (1) - runtime_error for message formatting

**Estimated time:** 25-30 minutes
**Difficulty:** Low (repetitive but numerous)

---

### 11. src/builtins/maps.rs (20 changes required)
- [ ] **10 ArityMismatch** - map-new, map-get, map-set, map-has?, map-keys, map-values, map-entries, map-merge, map-remove, map-size
  - [ ] map-new: 0 args
  - [ ] map-get: 2-3 args
  - [ ] map-set: 3 args
  - [ ] map-has?: 2 args
  - [ ] map-keys: 1 arg
  - [ ] map-values: 1 arg
  - [ ] map-entries: 1 arg
  - [ ] map-merge: 2 args
  - [ ] map-remove: 2 args
  - [ ] map-size: 1 arg

- [ ] **10 TypeError** - Type checks (map, keyword validation)

**Estimated time:** 15 minutes
**Difficulty:** Low-Medium (some functions have range checks like "2-3")

---

### 12. src/builtins/testing.rs (7 changes required)
- [ ] **5 ArityMismatch** - test-equal, print-test-summary, print-test-details, run-test
- [ ] **1 TypeError** - run-test type check
- [ ] **1 Custom** - "Test must be a lambda"

**Estimated time:** 5 minutes
**Difficulty:** Low

---

### 13. src/builtins/logic.rs (3 changes required)
- [ ] **1 ArityMismatch** - not function
- [ ] **2 TypeError** - and, or type checks

**Estimated time:** 3 minutes
**Difficulty:** Low

---

### 14. src/stdlib/json.rs (7 changes required)
- [ ] **3 ArityMismatch** - json:encode, json:decode, json:pretty
  - [ ] json:encode: 1 arg
  - [ ] json:decode: 1 arg
  - [ ] json:pretty: 1 arg

- [ ] **1 TypeError** - json:decode type check

- [ ] **3 Custom** - JSON library errors
  - [ ] serde_json::to_string error
  - [ ] serde_json::from_str error
  - [ ] serde_json::to_string_pretty error

**Pattern:** All Custom → `map_err(|e| EvalError::runtime_error("json:function", e.to_string()))`

**Estimated time:** 5 minutes
**Difficulty:** Low

---

## Summary Statistics

**Total Files:** 14
**Total Changes:** 178
- ArityMismatch: 72 replacements
- TypeError: 66 replacements
- Custom: 40 replacements

**Estimated Total Time:** 2.5-3 hours

**Difficulty Distribution:**
- Low: ~80% (mostly repetitive patterns)
- Medium: ~15% (eval.rs special forms, maps.rs range checks)
- High: ~5% (analyzing Custom errors to categorize correctly)

---

## Systematic Approach

### Phase 1: Easy Wins (90 minutes)
1. **src/builtins/types.rs** (8 changes, 5 min) - Identical pattern
2. **src/builtins/errors.rs** (4 changes, 3 min) - Simple
3. **src/builtins/logic.rs** (3 changes, 3 min) - Simple
4. **src/tools.rs** (2 changes, 2 min) - Simple
5. **src/builtins/help.rs** (4 changes, 5 min) - Mostly simple
6. **src/builtins/testing.rs** (7 changes, 5 min) - Straightforward
7. **src/builtins/network.rs** (7 changes, 5 min) - Straightforward
8. **src/stdlib/json.rs** (7 changes, 5 min) - Pattern clear
9. **src/builtins/comparison.rs** (12 changes, 8 min) - Repetitive
10. **src/builtins/lists.rs** (11 changes, 10 min) - Straightforward
11. **src/builtins/filesystem.rs** (12 changes, 10 min) - Repetitive

### Phase 2: Complex (60 minutes)
1. **src/builtins/maps.rs** (20 changes, 15 min) - Some range checks
2. **src/builtins/strings.rs** (40 changes, 30 min) - Largest file

### Phase 3: Special Cases (30 minutes)
1. **src/eval.rs** (26 changes, 25 min) - Special forms analysis needed

---

## Validation Checklist

After all replacements:

- [ ] Run `cargo fmt` - all files formatted
- [ ] Run `cargo clippy` - zero warnings
- [ ] Run `cargo test --all` - all 237 tests pass
- [ ] Verify specific error messages:
  - [ ] `(car 5)` shows "car: expected list, got number at argument 1"
  - [ ] `(quote 1 2)` shows proper arity error
  - [ ] `(+ 1 2 3)` shows proper arithmetic result (if applicable)
- [ ] Check git diff - 178 lines changed across 14 files

---

## Additional Notes

### When Stuck

1. **What error type is this?**
   - Checking argument count? → ArityMismatch
   - Checking argument type? → TypeError
   - Everything else? → Custom (analyze further)

2. **What function context?**
   - Look at function name: `pub fn builtin_XXX()`
   - Function name in doc: `#[builtin(name = "...")]`
   - Special form name: match on symbol string

3. **What position number?**
   - Usually 1-based from user perspective (args[0] = position 1)

### Tools to Help

```bash
# Find all instances in a file
grep -n "EvalError::" src/builtins/strings.rs

# Quick check after editing
cargo check --lib

# Run specific test
cargo test test_name -- --nocapture
```


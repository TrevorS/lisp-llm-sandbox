# Legacy Error Replacement Patterns

## Pattern 1: ArityMismatch (72 occurrences)

### Before Pattern:
```rust
pub fn builtin_cons(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);  // Generic, no context
    }
    // ... function body
}
```

### After Pattern:
```rust
pub fn builtin_cons(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::arity_error("cons", "2", args.len()));  // Rich context
    }
    // ... function body
}
```

### Key Points:
- Function name is added to error message
- Expected count is specified clearly
- Actual count is provided for debugging
- Works for all ArityMismatch cases

### Variations:

**For variable argument counts:**
```rust
// Before: if args.len() < 2 { return Err(EvalError::ArityMismatch); }

// After:
if args.len() < 2 {
    return Err(EvalError::arity_error("map", "at least 2", args.len()));
}
```

**For range of arguments:**
```rust
// Before: if args.len() < 2 || args.len() > 3 { return Err(EvalError::ArityMismatch); }

// After:
if args.len() < 2 || args.len() > 3 {
    return Err(EvalError::arity_error("map-get", "2-3", args.len()));
}
```

---

## Pattern 2: TypeError (66 occurrences)

### Before Pattern:
```rust
pub fn builtin_car(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match &args[0] {
        Value::List(items) if !items.is_empty() => Ok(items[0].clone()),
        Value::List(_) => Err(EvalError::Custom("car of empty list".to_string())),
        _ => return Err(EvalError::TypeError),  // Generic, no context
    }
}
```

### After Pattern:
```rust
pub fn builtin_car(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_error("car", "1", args.len()));
    }

    match &args[0] {
        Value::List(items) if !items.is_empty() => Ok(items[0].clone()),
        Value::List(_) => Err(EvalError::runtime_error("car", "empty list")),
        _ => return Err(EvalError::type_error("car", "list", &args[0], 1)),  // Rich context
    }
}
```

### Key Points:
- Function name is added to error message
- Expected type is specified (e.g., "list", "string", "number", "map", "keyword")
- Actual value is passed (for displaying what was actually provided)
- Position/argument index is specified (usually 1-based from user perspective)
- Value's Display impl shows what user received

### Variations:

**For multiple type checks:**
```rust
// Before:
match &args[0] {
    Value::List(_) => { /* ok */ },
    _ => return Err(EvalError::TypeError),
}

// After:
match &args[0] {
    Value::List(_) => { /* ok */ },
    _ => return Err(EvalError::type_error("function-name", "list", &args[0], 1)),
}
```

**For checking multiple valid types:**
```rust
// Before:
match &args[1] {
    Value::List(_) | Value::Nil => { /* ok */ },
    _ => return Err(EvalError::TypeError),
}

// After:
match &args[1] {
    Value::List(_) | Value::Nil => { /* ok */ },
    _ => return Err(EvalError::type_error("cons", "list or nil", &args[1], 2)),
}
```

---

## Pattern 3: Custom (40 occurrences)

### Type A: Arity-Related Errors

These are often hidden arity checks using Custom instead of ArityMismatch.

**Before:**
```rust
if items.len() != 2 {
    return Err(EvalError::Custom("quote: expected 1 argument".into()));
}
```

**After:**
```rust
if items.len() != 2 {
    return Err(EvalError::arity_error("quote", "1", items.len() - 1));
}
```

---

### Type B: Type-Related Errors

Custom errors that check types but were implemented as Custom instead of TypeError.

**Before:**
```rust
match &args[0] {
    Value::Map(m) => m,
    _ => return Err(EvalError::Custom("Expected map argument".into())),
}
```

**After:**
```rust
match &args[0] {
    Value::Map(m) => m,
    _ => return Err(EvalError::type_error("function-name", "map", &args[0], 1)),
}
```

---

### Type C: Genuine Runtime Errors

These are legitimate runtime errors that don't fit the arity/type pattern.

**Before:**
```rust
Value::List(_) => Err(EvalError::Custom("car of empty list".to_string()))
```

**After:**
```rust
Value::List(_) => Err(EvalError::runtime_error("car", "empty list"))
```

**Example messages:**
- "car of empty list" → runtime_error("car", "empty list")
- "cdr of empty list" → runtime_error("cdr", "empty list")
- "Test must be a lambda" → runtime_error("run-test", "test must be a lambda")
- "No help found for 'foo'" → runtime_error("help", "no help found for 'foo'")

---

### Type D: JSON/HTTP Context Errors

Complex errors that include context from external libraries.

**Before:**
```rust
serde_json::to_string(&json_value)
    .map_err(|e| EvalError::Custom(e.to_string()))?
```

**After:**
```rust
serde_json::to_string(&json_value)
    .map_err(|e| EvalError::runtime_error("json:encode", e.to_string()))?
```

---

## File-by-File Quick Reference

### src/eval.rs
All 24 Custom errors → `runtime_error("form-name", "error message")`
- `quote: expected 1 argument` → `runtime_error("quote", "expected 1 argument")`
- `if: expected 2 or 3 arguments` → `runtime_error("if", "expected 2 or 3 arguments")`
- Etc. for define, let, defmacro, quasiquote, unquote

2 ArityMismatch → `arity_error("eval_function", "2", args.len())`

---

### src/builtins/types.rs (8 ArityMismatch)
```rust
// Before: Err(EvalError::ArityMismatch)
// After: Err(EvalError::arity_error("number?", "1", args.len()))

// All 8 type predicates follow same pattern
```

---

### src/builtins/strings.rs (Largest: 12 ArityMismatch, 27 TypeError, 1 Custom)

**ArityMismatch pattern:**
```rust
if args.len() != 1 {
    return Err(EvalError::arity_error("string-length", "1", args.len()));
}
```

**TypeError pattern:**
```rust
match &args[0] {
    Value::String(s) => Ok(Value::Number(s.len() as f64)),
    _ => return Err(EvalError::type_error("string-length", "string", &args[0], 1)),
}
```

---

### src/builtins/maps.rs (10 ArityMismatch, 10 TypeError)

**ArityMismatch pattern:**
```rust
if !args.is_empty() {
    return Err(EvalError::arity_error("map-new", "0", args.len()));
}

if args.len() < 2 || args.len() > 3 {
    return Err(EvalError::arity_error("map-get", "2-3", args.len()));
}
```

**TypeError pattern:**
```rust
match &args[0] {
    Value::Map(m) => m,
    _ => return Err(EvalError::type_error("map-get", "map", &args[0], 1)),
}

match &args[1] {
    Value::Keyword(k) => k,
    _ => return Err(EvalError::type_error("map-get", "keyword", &args[1], 2)),
}
```

---

### src/builtins/lists.rs (4 ArityMismatch, 5 TypeError, 2 Custom)

Pattern: 4 functions with simple arity checks, 5 with type checks, 2 with custom messages:

```rust
// ArityMismatch
if args.len() != 2 {
    return Err(EvalError::arity_error("cons", "2", args.len()));
}

// TypeError
match &args[1] {
    Value::List(_) => { /* ... */ },
    Value::Nil => { /* ... */ },
    _ => return Err(EvalError::type_error("cons", "list or nil", &args[1], 2)),
}

// Custom (runtime errors)
Value::List(_) => Err(EvalError::runtime_error("car", "empty list"))
```

---

## Helper Function Signatures

From src/error.rs:

```rust
/// Create a type mismatch error with full context
pub fn type_error(function: &str, expected: &str, actual: &Value, position: usize) -> Self

/// Create an arity error with expected and actual counts
pub fn arity_error(function: &str, expected: impl Into<String>, actual: usize) -> Self

/// Create a runtime error with function context
pub fn runtime_error(function: &str, message: impl Into<String>) -> Self
```

---

## Testing & Validation Checklist

After replacements:
- [ ] All 178 legacy error usages have been replaced
- [ ] Run `cargo clippy` - should still have zero warnings
- [ ] Run `cargo test --all` - all 237 tests should pass
- [ ] Verify error messages still make sense to users
- [ ] Check that position indices in TypeError are accurate (1-based)
- [ ] Ensure function names match what user sees

## Automation Tips

Could create sed/regex patterns:
```bash
# Find all ArityMismatch: grep -n "EvalError::ArityMismatch" src/**/*.rs
# Find all TypeError: grep -n "EvalError::TypeError" src/**/*.rs
# Find all Custom: grep -n "EvalError::Custom" src/**/*.rs
```

Or write a small script to batch-convert them systematically.


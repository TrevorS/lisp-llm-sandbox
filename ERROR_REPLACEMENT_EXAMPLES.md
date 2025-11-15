# Concrete Examples: Line-by-Line Replacements from Actual Code

This document provides real examples from the codebase showing exactly what to replace.

---

## EXAMPLE 1: src/builtins/types.rs - ArityMismatch Pattern

### Current Code (Line ~18)
```rust
#[builtin(name = "number?", category = "Type predicates")]
pub fn is_number(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }
    Ok(Value::Bool(matches!(args[0], Value::Number(_))))
}
```

### Replacement Code
```rust
#[builtin(name = "number?", category = "Type predicates")]
pub fn is_number(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_error("number?", "1", args.len()));
    }
    Ok(Value::Bool(matches!(args[0], Value::Number(_))))
}
```

### Why
- Function name "number?" provides context
- "1" is the expected argument count
- `args.len()` is the actual count
- User gets: "number?: expected 1 argument, got 0" instead of generic message

---

## EXAMPLE 2: src/builtins/lists.rs - TypeError Pattern

### Current Code (Line ~63)
```rust
pub fn builtin_car(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match &args[0] {
        Value::List(items) if !items.is_empty() => Ok(items[0].clone()),
        Value::List(_) => Err(EvalError::Custom("car of empty list".to_string())),
        _ => return Err(EvalError::TypeError),
    }
}
```

### Replacement Code
```rust
pub fn builtin_car(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_error("car", "1", args.len()));
    }

    match &args[0] {
        Value::List(items) if !items.is_empty() => Ok(items[0].clone()),
        Value::List(_) => Err(EvalError::runtime_error("car", "empty list")),
        _ => return Err(EvalError::type_error("car", "list", &args[0], 1)),
    }
}
```

### Why
- Line 65: ArityMismatch → arity_error with function name and counts
- Line 69: Custom → runtime_error (genuine runtime error, not a type/arity issue)
- Line 71: TypeError → type_error with expected type and actual value
- User gets: "car: expected list, got number at argument 1" instead of generic error

---

## EXAMPLE 3: src/builtins/maps.rs - Multiple TypeError Checks

### Current Code (Lines ~42-54)
```rust
pub fn map_get(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(EvalError::ArityMismatch);
    }

    let map = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    let key = match &args[1] {
        Value::Keyword(k) => k,
        _ => return Err(EvalError::TypeError),
    };
    // ... rest of function
}
```

### Replacement Code
```rust
pub fn map_get(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(EvalError::arity_error("map-get", "2-3", args.len()));
    }

    let map = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::type_error("map-get", "map", &args[0], 1)),
    };

    let key = match &args[1] {
        Value::Keyword(k) => k,
        _ => return Err(EvalError::type_error("map-get", "keyword", &args[1], 2)),
    };
    // ... rest of function
}
```

### Why
- Line 43: ArityMismatch → arity_error("map-get", "2-3", args.len())
  - Shows expected range "2-3"
- Line 48: TypeError → type_error("map-get", "map", &args[0], 1)
  - Position 1 = first argument
  - Expected type "map"
  - Actual value shown via Display impl
- Line 53: TypeError → type_error("map-get", "keyword", &args[1], 2)
  - Position 2 = second argument
  - Expected type "keyword"

---

## EXAMPLE 4: src/builtins/strings.rs - Mixed Error Types

### Current Code (Lines ~32-48)
```rust
pub fn string_length(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Number(s.len() as f64)),
        _ => return Err(EvalError::TypeError),
    }
}
```

### Replacement Code
```rust
pub fn string_length(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_error("string-length", "1", args.len()));
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Number(s.len() as f64)),
        _ => return Err(EvalError::type_error("string-length", "string", &args[0], 1)),
    }
}
```

---

## EXAMPLE 5: src/eval.rs - Special Forms with Custom Errors

### Current Code (Lines ~72-76)
```rust
Value::Symbol(s) if s == "quote" => {
    if items.len() != 2 {
        return Err(EvalError::Custom("quote: expected 1 argument".into()));
    }
    return Ok(items[1].clone());
}
```

### Replacement Code
```rust
Value::Symbol(s) if s == "quote" => {
    if items.len() != 2 {
        return Err(EvalError::arity_error("quote", "1", items.len() - 1));
    }
    return Ok(items[1].clone());
}
```

### Why
- "quote" is the special form name
- "1" is expected arguments (items.len() - 1 because items[0] is "quote")
- items.len() - 1 is actual argument count
- Maintains precise error but uses proper error variant

---

## EXAMPLE 6: src/eval.rs - Special Forms with Structural Errors

### Current Code (Lines ~348-353)
```rust
let bindings_list = match &items[1] {
    Value::List(bindings) => bindings,
    _ => return Err(EvalError::Custom("let: bindings must be a list".into())),
};

for (i, binding) in bindings_list.iter().enumerate() {
    let pair = match binding {
        Value::List(pair) if pair.len() == 2 => pair,
        _ => return Err(EvalError::Custom("let: binding must be [name value] pair".into())),
    };
```

### Replacement Code
```rust
let bindings_list = match &items[1] {
    Value::List(bindings) => bindings,
    _ => return Err(EvalError::type_error("let", "list", &items[1], 1)),
};

for (i, binding) in bindings_list.iter().enumerate() {
    let pair = match binding {
        Value::List(pair) if pair.len() == 2 => pair,
        _ => return Err(EvalError::runtime_error("let", 
            format!("binding must be [name value] pair, got {}", binding.type_name()))),
    };
```

### Why
- Line 351: The bindings parameter must be a list → type_error (not custom)
- Line 355: The binding structure check is a runtime constraint → runtime_error (not type or arity)

---

## EXAMPLE 7: src/builtins/network.rs - HTTP Error Context

### Current Code (Lines ~195-210)
```rust
// Invalid headers validation
_ => return Err(EvalError::Custom("Invalid :headers in options".to_string())),

// Body validation
_ => return Err(EvalError::Custom("Body must be a string".to_string())),

// Timeout validation
_ => return Err(EvalError::Custom("Timeout must be a number".to_string())),

// HTTP request failure
return Err(EvalError::Custom(
    format!("HTTP {} request to '{}' failed: {}", method, url, e)
));
```

### Replacement Code
```rust
// Invalid headers validation
_ => return Err(EvalError::runtime_error("http-request", "invalid :headers in options")),

// Body validation
_ => return Err(EvalError::runtime_error("http-request", "body must be a string")),

// Timeout validation
_ => return Err(EvalError::runtime_error("http-request", "timeout must be a number")),

// HTTP request failure
return Err(EvalError::runtime_error(
    "http-request",
    format!("HTTP {} request to '{}' failed: {}", method, url, e)
));
```

### Why
- All are runtime constraint violations, not type/arity issues
- runtime_error provides the function context
- Messages remain informative

---

## EXAMPLE 8: src/stdlib/json.rs - External Library Errors

### Current Code (Lines ~50-55)
```rust
serde_json::to_string(&json_value)
    .map_err(|e| EvalError::Custom(e.to_string()))?;

// ... later ...

serde_json::from_str(json_str)
    .map_err(|e| EvalError::Custom(e.to_string()))?;
```

### Replacement Code
```rust
serde_json::to_string(&json_value)
    .map_err(|e| EvalError::runtime_error("json:encode", e.to_string()))?;

// ... later ...

serde_json::from_str(json_str)
    .map_err(|e| EvalError::runtime_error("json:decode", e.to_string()))?;
```

### Why
- External library errors are runtime errors
- Function name "json:encode" or "json:decode" provides context
- Error messages from serde_json are preserved

---

## EXAMPLE 9: src/builtins/comparison.rs - Operator Type Checks

### Current Code (Lines ~25-40)
```rust
pub fn builtin_equal(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a == b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a == b)),
        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
        _ => return Err(EvalError::TypeError),
    }
}
```

### Replacement Code
```rust
pub fn builtin_equal(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::arity_error("=", "2", args.len()));
    }

    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a == b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a == b)),
        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
        _ => {
            // Both arguments must be same comparable type
            if args[0].type_name() != args[1].type_name() {
                return Err(EvalError::runtime_error("=", 
                    format!("cannot compare {} and {}", args[0].type_name(), args[1].type_name())));
            }
            return Err(EvalError::type_error("=", "number, string, or bool", &args[0], 1));
        }
    }
}
```

### Why
- Line 28: ArityMismatch → arity_error("=", "2", args.len())
- Line 38: TypeError → Could be either:
  - Different types → runtime_error with explanation
  - Unsupported type → type_error with what's supported

---

## QUICK FIND & REPLACE GUIDE

### For ArityMismatch (72 occurrences)

Search: `return Err\(EvalError::ArityMismatch\);`

Replace with: Need to add function name and expected count (context-specific)

Example regex substitution script:
```bash
# Manual approach: Find each instance and replace with context
# No single regex works for all cases
```

---

### For TypeError (66 occurrences)

Search: `return Err\(EvalError::TypeError\);`

Replace with: `return Err(EvalError::type_error("function_name", "expected_type", &value, position));`

Key: Determine expected type and which argument position (usually 1-based)

---

### For Custom (40 occurrences)

Search: `EvalError::Custom\(`

Replace with: Either:
1. `EvalError::arity_error(...)` - if checking argument count
2. `EvalError::type_error(...)` - if checking type
3. `EvalError::runtime_error(...)` - for genuine runtime errors

Key: Analyze the message to determine which category

---

## Testing Commands

After each replacement, verify:

```bash
# Check specific file compiles
cargo check --lib

# Run all tests
cargo test --all

# Check for clippy warnings
cargo clippy

# Full validation
cargo fmt && cargo clippy && cargo test --all
```

All should pass with zero warnings.


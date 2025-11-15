# Unified Runtime Error System Design

## Problem Statement

Current error handling has several critical issues:
1. **Lost Context**: Generic errors like "Type error in operation" don't tell users which function failed
2. **No Type Information**: Type errors don't show what was expected vs received
3. **No Arity Information**: Arity errors don't show expected vs actual count
4. **Inconsistent Formatting**: Different error prefixes across REPL, script mode, stdlib loading
5. **Double Prefixes**: Parser errors get "Parse error:" prefix twice
6. **Unhelpful Messages**: Users can't determine what to fix

### Examples of Current Problems

```lisp
;; Current behavior:
(+ 1 "hello")  → Error: Type error in operation
;; Should be: Type error in +: expected number, got string at argument 2

(cons 1 2 3)   → Error: Arity mismatch: incorrect number of arguments
;; Should be: Arity error in cons: expected 2 arguments, got 3

(/ 10 0)       → Error: Division by zero
;; Should be: Arithmetic error in /: division by zero
```

## Design Goals

1. **Context-Rich**: Every error knows which function/operation failed
2. **Actionable**: Users can understand what went wrong and how to fix it
3. **Incremental**: Can adopt gradually without massive refactoring
4. **Backwards Compatible**: Existing error returns still work
5. **Low Overhead**: Minimal performance impact
6. **Easy to Use**: Builtin authors can easily create good errors

## Proposed Solution: Three-Phase Approach

### Phase 1: Enhanced Error Types (Immediate - 2-4 hours)

Enhance `EvalError` enum with richer variants while keeping existing ones:

```rust
#[derive(Error, Debug, Clone)]
pub enum EvalError {
    // Enhanced variants with context
    #[error("{function}: expected {expected}, got {actual} at argument {position}")]
    TypeMismatch {
        function: String,
        expected: String,
        actual: String,
        position: usize,
    },

    #[error("{function}: expected {expected} argument{}, got {actual}")]
    ArityError {
        function: String,
        expected: String,  // "2", "1-3", "at least 1"
        actual: usize,
    },

    #[error("{function}: {message}")]
    RuntimeError {
        function: String,
        message: String,
    },

    // Legacy variants (keep for backwards compatibility)
    #[error("Undefined symbol: {0}")]
    UndefinedSymbol(String),

    #[error("Value is not callable")]
    NotCallable,

    #[error("Type error in operation")]
    TypeError,

    #[error("Arity mismatch: incorrect number of arguments")]
    ArityMismatch,

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("{0}")]
    Custom(String),
}
```

### Phase 2: Helper Functions & Macros (2-3 hours)

Create ergonomic helpers for common error patterns:

```rust
impl EvalError {
    /// Create type mismatch error
    pub fn type_error(function: &str, expected: &str, actual: &Value, position: usize) -> Self {
        EvalError::TypeMismatch {
            function: function.to_string(),
            expected: expected.to_string(),
            actual: actual.type_name(),
            position,
        }
    }

    /// Create arity error
    pub fn arity_error(function: &str, expected: impl Into<String>, actual: usize) -> Self {
        EvalError::ArityError {
            function: function.to_string(),
            expected: expected.into(),
            actual,
        }
    }

    /// Create runtime error with context
    pub fn runtime_error(function: &str, message: impl Into<String>) -> Self {
        EvalError::RuntimeError {
            function: function.to_string(),
            message: message.into(),
        }
    }

    /// Add function context to any error
    pub fn in_function(self, function: &str) -> Self {
        match self {
            // Already has context
            EvalError::TypeMismatch { .. }
            | EvalError::ArityError { .. }
            | EvalError::RuntimeError { .. } => self,

            // Add context to legacy errors
            EvalError::TypeError => EvalError::RuntimeError {
                function: function.to_string(),
                message: "type error in operation".to_string(),
            },
            EvalError::ArityMismatch => EvalError::RuntimeError {
                function: function.to_string(),
                message: "arity mismatch".to_string(),
            },
            EvalError::Custom(msg) => EvalError::RuntimeError {
                function: function.to_string(),
                message: msg,
            },
            other => other,
        }
    }
}

impl Value {
    /// Get user-friendly type name
    pub fn type_name(&self) -> String {
        match self {
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Symbol(_) => "symbol".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::List(_) => "list".to_string(),
            Value::Map(_) => "map".to_string(),
            Value::Keyword(_) => "keyword".to_string(),
            Value::Lambda { .. } => "function".to_string(),
            Value::Macro { .. } => "macro".to_string(),
            Value::BuiltIn(_) => "builtin function".to_string(),
            Value::Error(_) => "error".to_string(),
            Value::Nil => "nil".to_string(),
        }
    }
}
```

### Phase 3: Builtin Call Wrapper (1-2 hours)

Wrap builtin function calls in eval.rs to auto-inject context:

```rust
// In eval.rs, around line 167:
Value::BuiltIn(f) => {
    // Get function name if we're calling from a symbol
    let function_name = if let Value::List(list) = &expr {
        if let Some(Value::Symbol(name)) = list.first() {
            name.as_str()
        } else {
            "<anonymous>"
        }
    } else {
        "<anonymous>"
    };

    // Call function and add context on error
    return f(&args).map_err(|e| e.in_function(function_name));
}
```

### Phase 4: Fix REPL Error Display (30 minutes)

Standardize error output in main.rs:

```rust
// In REPL (main.rs around line 186):
Err(e) => {
    // Don't add "Error:" prefix - error already formats itself
    eprintln!("{}", e);
}

// In parser error (main.rs around line 191):
Err(e) => {
    // Parser already adds "Parse error:" prefix
    eprintln!("{}", e);
}
```

### Phase 5: Update Builtins Gradually (4-8 hours)

Update builtins to use new error helpers. Example for arithmetic.rs:

```rust
// Before:
pub fn builtin_add(args: &[Value]) -> Result<Value, EvalError> {
    let mut sum = 0.0;
    for arg in args {
        match arg {
            Value::Number(n) => sum += n,
            _ => return Err(EvalError::TypeError),  // ❌ Generic
        }
    }
    Ok(Value::Number(sum))
}

// After:
pub fn builtin_add(args: &[Value]) -> Result<Value, EvalError> {
    let mut sum = 0.0;
    for (i, arg) in args.iter().enumerate() {
        match arg {
            Value::Number(n) => sum += n,
            _ => return Err(EvalError::type_error("+", "number", arg, i + 1)),  // ✅ Contextful
        }
    }
    Ok(Value::Number(sum))
}
```

## Migration Path

### Week 1: Foundation (4-6 hours)
- [ ] Enhance EvalError enum
- [ ] Add helper methods to EvalError
- [ ] Add type_name() to Value
- [ ] Fix REPL error display
- [ ] Add builtin call wrapper in eval.rs

### Week 2: High-Impact Builtins (6-8 hours)
- [ ] Update arithmetic operations (8 functions)
- [ ] Update string operations (16 functions)
- [ ] Update list operations (6 functions)
- [ ] Update map operations (10 functions)
- [ ] Update type predicates (8 functions)

### Week 3: Remaining Builtins (4-6 hours)
- [ ] Update comparison operations
- [ ] Update logic operations
- [ ] Update I/O operations
- [ ] Update error handling functions

### Week 4: Polish & Documentation (2-3 hours)
- [ ] Add error handling guide to CLAUDE.md
- [ ] Create examples of good error messages
- [ ] Add tests for error formatting
- [ ] Document patterns for future builtin authors

## Benefits

### Before
```lisp
lisp> (+ 1 "hello")
Error: Type error in operation

lisp> (cons 1 2 3)
Error: Arity mismatch: incorrect number of arguments

lisp> (map-get)
Error: Arity mismatch: incorrect number of arguments
```

### After
```lisp
lisp> (+ 1 "hello")
+: expected number, got string at argument 2

lisp> (cons 1 2 3)
cons: expected 2 arguments, got 3

lisp> (map-get)
map-get: expected 2 arguments, got 0

lisp> (/ 10 0)
/: division by zero

lisp> (substring "hello" 10)
substring: index 10 out of bounds for string of length 5
```

## Testing Strategy

1. **Existing Tests**: All 237 tests should pass (backwards compatibility)
2. **New Error Tests**: Add tests for new error formats
3. **Error Message Tests**: Verify error messages are user-friendly
4. **Regression Tests**: Ensure no error messages got worse

## Future Enhancements

### Error Codes (Optional)
```rust
#[error("[E001] {function}: expected {expected}, got {actual} at argument {position}")]
```

### Suggestions (Optional)
```rust
#[error("{message}\n  hint: did you mean '{suggestion}'?")]
```

### Stack Traces (Optional)
Track call stack for nested function calls.

### Colorized Errors (Optional)
Use terminal colors to highlight error parts.

## Implementation Priority

**High Priority (Immediate Impact):**
1. ✅ Fix double "Parse error:" prefix (5 min)
2. ✅ Add builtin call wrapper (30 min)
3. ✅ Enhance EvalError enum (1 hour)
4. ✅ Add helper methods (1 hour)
5. ✅ Update arithmetic operations (1 hour)

**Medium Priority (Next Sprint):**
6. Update string operations
7. Update list/map operations
8. Update I/O operations

**Low Priority (Nice to Have):**
9. Add error codes
10. Add suggestions
11. Add stack traces

## Success Metrics

- **User Satisfaction**: Errors help users fix their code
- **Support Reduction**: Fewer "what does this error mean?" questions
- **Code Quality**: Easier to debug Lisp programs
- **Developer Experience**: Easier to add new builtins with good errors

## Conclusion

This design provides:
- **Immediate value** with Phase 1-4 (can complete in 1 day)
- **Incremental adoption** without breaking changes
- **Clear migration path** spread over 4 weeks
- **10x better error messages** for users
- **Easy maintenance** for future contributors

The key insight is wrapping builtin calls in eval.rs to auto-inject function context, combined with richer error variants that preserve information through the stack.

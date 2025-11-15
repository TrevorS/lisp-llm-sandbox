# Legacy Error Variants Audit - Complete Documentation Index

This folder contains comprehensive documentation for systematically replacing all 178 usages of legacy error variants (TypeError, ArityMismatch, Custom) throughout the codebase.

---

## Quick Summary

**Total Legacy Error Usages:** 178
- **ArityMismatch:** 72 occurrences
- **TypeError:** 66 occurrences  
- **Custom:** 40 occurrences

**Files to Update:** 14
- src/eval.rs (26 changes)
- src/tools.rs (2 changes)
- src/builtins/ (10 files, 116 changes)
- src/stdlib/json.rs (7 changes)

**Estimated Time:** 2.5-3 hours
**Difficulty:** Low-Medium (mostly repetitive patterns)

---

## Documentation Files

### 1. ERROR_INVENTORY_SUMMARY.txt
**Purpose:** Quick reference overview of all error usages
**Use When:** You need a bird's-eye view of the scope

**Contains:**
- High-level count by file
- Total statistics (178 total, breakdown by type)
- Grand totals

---

### 2. ERROR_INVENTORY_DETAILED.md
**Purpose:** Comprehensive detailed breakdown with context
**Use When:** You need to understand what each error category means

**Contains:**
- File-by-file detailed inventory
- Error type breakdown for each file
- Description of what each error is checking
- 14 files fully documented

**Read this first** for complete understanding of the scope.

---

### 3. ERROR_REPLACEMENT_GUIDE.md
**Purpose:** Pattern reference for how to replace each error type
**Use When:** You need to know HOW to do the replacements

**Contains:**
- Pattern 1: ArityMismatch replacement strategy (72 occurrences)
- Pattern 2: TypeError replacement strategy (66 occurrences)
- Pattern 3: Custom replacement strategy (40 occurrences)
- Helper function signatures from src/error.rs
- File-by-file quick reference guide
- Validation checklist

**Key reference during implementation.**

---

### 4. ERROR_REPLACEMENT_EXAMPLES.md
**Purpose:** Concrete before/after code examples from actual codebase
**Use When:** You need concrete examples for your specific file

**Contains:**
- 9 detailed real-world examples:
  1. src/builtins/types.rs - ArityMismatch pattern
  2. src/builtins/lists.rs - TypeError pattern
  3. src/builtins/maps.rs - Multiple TypeError checks
  4. src/builtins/strings.rs - Mixed error types
  5. src/eval.rs - Special forms with Custom
  6. src/eval.rs - Structural errors
  7. src/builtins/network.rs - HTTP context errors
  8. src/stdlib/json.rs - External library errors
  9. src/builtins/comparison.rs - Operator type checks

**Most useful** for copy-paste reference during actual edits.

---

### 5. ERROR_REPLACEMENT_CHECKLIST.md
**Purpose:** Systematic step-by-step checklist for tracking progress
**Use When:** You're actively doing the replacements

**Contains:**
- File-by-file checklist with estimated times
- Specific line numbers and change descriptions
- Phased approach (Easy Wins → Complex → Special Cases)
- Summary statistics
- Validation checklist (pre-commit QA)
- Troubleshooting tips

**Use during implementation to track progress.**

---

### 6. ERROR_SYSTEM_DESIGN.md
**Purpose:** Context and design rationale (if exists)
**Use When:** You want to understand the larger error handling design

---

## Reading Order

### For Quick Understanding (15 minutes)
1. Read this file
2. Read ERROR_INVENTORY_SUMMARY.txt
3. Scan ERROR_REPLACEMENT_GUIDE.md sections for your target files

### For Complete Understanding (45 minutes)
1. Read this file
2. Read ERROR_INVENTORY_DETAILED.md (full context)
3. Read ERROR_REPLACEMENT_GUIDE.md (patterns)
4. Scan ERROR_REPLACEMENT_EXAMPLES.md (concrete examples)

### For Implementation (Active Work)
1. Use ERROR_REPLACEMENT_CHECKLIST.md as main guide
2. Reference ERROR_REPLACEMENT_EXAMPLES.md for copy-paste patterns
3. Reference ERROR_REPLACEMENT_GUIDE.md for edge cases
4. Use cargo check/test after each file

---

## The Three Error Variants

### 1. ArityMismatch (72 occurrences)
**Current:** `Err(EvalError::ArityMismatch)`
**Problem:** No context - users don't know which function failed or what counts were expected

**Replacement:** `Err(EvalError::arity_error("function_name", "expected", actual))`
**Benefit:** Rich context - users see "function_name: expected X arguments, got Y"

**Example:**
```rust
// Before
if args.len() != 2 {
    return Err(EvalError::ArityMismatch);
}

// After
if args.len() != 2 {
    return Err(EvalError::arity_error("cons", "2", args.len()));
}
```

---

### 2. TypeError (66 occurrences)
**Current:** `Err(EvalError::TypeError)`
**Problem:** Generic error - users don't know what type was expected or which argument is wrong

**Replacement:** `Err(EvalError::type_error("function_name", "expected_type", &value, position))`
**Benefit:** Detailed context - users see "function_name: expected list, got number at argument 1"

**Example:**
```rust
// Before
match &args[0] {
    Value::List(_) => { /* ... */ },
    _ => return Err(EvalError::TypeError),
}

// After
match &args[0] {
    Value::List(_) => { /* ... */ },
    _ => return Err(EvalError::type_error("car", "list", &args[0], 1)),
}
```

---

### 3. Custom (40 occurrences)
**Current:** `Err(EvalError::Custom("message"))`
**Problem:** Can represent anything - hard to know if it's arity, type, or genuine runtime error

**Replacement:** Choose best fit:
- **Arity-related:** `EvalError::arity_error("name", "expected", actual)`
- **Type-related:** `EvalError::type_error("name", "type", &value, pos)`
- **Runtime error:** `EvalError::runtime_error("name", "message")`

**Benefit:** Semantic correctness - errors are categorized appropriately

**Example:**
```rust
// Before
if items.len() != 2 {
    return Err(EvalError::Custom("quote: expected 1 argument".into()));
}

// After
if items.len() != 2 {
    return Err(EvalError::arity_error("quote", "1", items.len() - 1));
}
```

---

## Implementation Strategy

### Phase 1: Easy Wins (90 minutes)
Start with files that have simple, repetitive patterns:
- types.rs (8 changes, identical pattern)
- errors.rs (4 changes, simple)
- logic.rs (3 changes, simple)
- tools.rs (2 changes, simple)
- help.rs, testing.rs, network.rs, json.rs (each 4-7 changes)
- comparison.rs, lists.rs, filesystem.rs (each 10-12 changes)

**Strategy:** Use find/replace with context-specific analysis

### Phase 2: Complex (60 minutes)
- maps.rs (20 changes, some range checks like "2-3")
- strings.rs (40 changes, largest file but repetitive)

**Strategy:** Process systematically, following existing patterns

### Phase 3: Special Cases (30 minutes)
- eval.rs (26 changes, special forms need careful analysis)

**Strategy:** Analyze each special form's error carefully, distinguish arity vs structure

---

## Validation

After completing all replacements:

1. **Compilation Check:**
   ```bash
   cargo check --lib
   ```

2. **Full Test Suite:**
   ```bash
   cargo test --all
   ```
   Expected: All 237 tests pass

3. **Linting:**
   ```bash
   cargo clippy
   ```
   Expected: Zero warnings (maintained)

4. **Formatting:**
   ```bash
   cargo fmt
   ```

5. **Git Validation:**
   ```bash
   git diff --stat
   ```
   Expected: 14 files changed, ~178 insertions/deletions

---

## Key Points to Remember

1. **Position indices are 1-based** from user perspective
   - args[0] = position 1
   - args[1] = position 2
   - etc.

2. **Function names matter** - they provide context
   - Use the builtin name from `#[builtin(name = "...")]`
   - Or the special form name from symbol matching

3. **Expected type strings should be readable** 
   - "list" ✓
   - "string" ✓
   - "list or nil" ✓
   - "comparable type" ✓
   - "map" ✓
   - "keyword" ✓

4. **Runtime errors are for genuine failures**
   - "empty list" (not a type/arity issue)
   - "no help found for..." (lookup failure)
   - "invalid JSON" (external library error)

---

## Troubleshooting

**Q: How do I know which helper to use?**
A: Ask "What is the user doing wrong?"
- Wrong number of args? → arity_error
- Wrong type of arg? → type_error
- Something else (validation, lookup, etc)? → runtime_error

**Q: What if it's a range like "2-3 arguments"?**
A: Pass the range as a string:
```rust
EvalError::arity_error("map-get", "2-3", args.len())
```

**Q: What about special forms in eval.rs?**
A: Most are Custom errors with arg count info embedded.
- If checking arg count: Convert to arity_error
- If structural validation: Use runtime_error with descriptive message

**Q: Will tests fail?**
A: No - error messages have improved Display impl, but semantics are unchanged.
The test assertion `matches!(result, Err(EvalError::ArityMismatch))` changes to
`matches!(result, Err(EvalError::ArityError {...}))` - update these assertions.

---

## File Locations

All documentation files are in this directory:
```
/home/user/lisp-llm-sandbox/
├── ERROR_AUDIT_INDEX.md (this file)
├── ERROR_INVENTORY_SUMMARY.txt
├── ERROR_INVENTORY_DETAILED.md
├── ERROR_REPLACEMENT_GUIDE.md
├── ERROR_REPLACEMENT_EXAMPLES.md
├── ERROR_REPLACEMENT_CHECKLIST.md
└── ERROR_SYSTEM_DESIGN.md
```

---

## Questions?

If something is unclear:
1. Check ERROR_REPLACEMENT_EXAMPLES.md for a similar case
2. Look at ERROR_REPLACEMENT_GUIDE.md for the pattern
3. Review src/error.rs for the exact function signatures
4. Run `cargo test` to validate your changes

---

## Next Steps

1. Start with ERROR_REPLACEMENT_CHECKLIST.md
2. Follow the Phase 1 → Phase 2 → Phase 3 approach
3. Use ERROR_REPLACEMENT_EXAMPLES.md for reference
4. Validate with `cargo test --all` after each phase
5. Commit with meaningful message about error audit

Good luck!

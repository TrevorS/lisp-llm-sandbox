# V2 Arc Migration - Status Update

## What We're Doing

Refactoring from single-threaded `Rc<Environment>` to thread-safe `Arc<Environment>` to enable true concurrent execution with `spawn`.

## Changes Made So Far

### ✅ Completed

1. **src/env.rs** - Core environment refactoring
   - Changed `Rc<Environment>` → `Arc<Environment>`
   - Changed `RefCell<HashMap>` → `HashMap` (immutable)
   - Added `extend()` method (returns new environment)
   - Deprecated `define()` with panic to catch old usage
   - Updated all tests to use new pattern
   - Added test for immutability verification

2. **src/value.rs** - Lambda environment update
   - Changed `Lambda.env` from `Rc<Environment>` to `Arc<Environment>`
   - Removed `use std::rc::Rc` (no longer needed)

3. **Documentation**
   - Created CONCURRENCY_V2_DESIGN.md with full architecture
   - Created V2_MIGRATION_STATUS.md (this file)

### 🚧 In Progress

4. **Compilation fixes** - Updating all code to work with Arc
   - Need to update `src/eval.rs` (main evaluator)
   - Need to update `src/builtins/mod.rs` (builtin registration)
   - Need to update `src/builtins/help.rs` (help registration)
   - Need to update `src/builtins/testing.rs` (test framework)
   - Many more files will need updates

## Compiler Errors To Fix

Current errors (as of last build):
- `eval.rs`: Multiple type mismatches (Rc vs Arc)
- `eval.rs`: Uses of deprecated `define()` method
- `testing.rs`: Type mismatch in Environment::new()
- `mod.rs`: Uses of deprecated `define()` method
- `help.rs`: Uses of deprecated `define()` method

## Migration Pattern

### Old Pattern (Mutable)
```rust
let env = Environment::new();
env.define("x".to_string(), Value::Number(42.0));
// env is mutated in place
```

### New Pattern (Immutable/Functional)
```rust
let env = Environment::new();
let env = env.extend("x".to_string(), Value::Number(42.0));
// env is shadowed with new environment
```

## Next Steps

1. Update `src/eval.rs` to use `Arc<Environment>` throughout
2. Replace all `env.define()` calls with `env.extend()` pattern
3. Update function signatures to return new environments where needed
4. Fix all compilation errors
5. Run test suite
6. Implement `spawn` builtin once everything compiles
7. Add `spawn-link`, `parallel-map`, etc.

## Design Decisions

### Why Immutable Environments?

1. **Thread-safe without locks** - No RefCell, no RwLock needed
2. **Functional purity** - Easier to reason about
3. **Snapshot semantics** - Each spawn gets immutable snapshot
4. **No race conditions** - Can't mutate what doesn't exist

### Why Arc Instead of Rc?

1. **Send + Sync** - Can cross thread boundaries
2. **Atomic refcounting** - Thread-safe reference counting
3. **Small overhead** - ~2-5% vs Rc, worth it for concurrency

### The Evaluator Pattern

The evaluator becomes a pure function:
- Input: (Expression, Environment)
- Output: Value
- Side effects: Only through channels/atoms (explicit)

For `define`, we need to thread the new environment through:
```rust
// Pseudo-code pattern
fn eval_define(expr, env) -> (Value, Environment) {
    let value = eval(expr, env.clone());
    let new_env = env.extend(name, value);
    (value, new_env)  // Return both!
}
```

## Estimated Completion

- **Phase 1** (Arc refactoring): 80% done, ~2-4 hours remaining
- **Phase 2** (spawn implementation): ~2 hours after Phase 1
- **Phase 3** (advanced primitives): ~4-6 hours

**Total**: ~8-12 hours to production-ready concurrent Lisp

## Testing Strategy

After each fix:
1. Run `cargo build` - fix compilation errors
2. Run `cargo test` - fix test failures
3. Run REPL - manual smoke testing
4. Proceed to next component

## Why This Matters

Once this is done, we can:
- Execute Lisp code concurrently (true parallelism)
- Make 100 API calls in parallel (vs sequential)
- Process thousands of files concurrently
- Build real ETL pipelines
- Create LLM-friendly automation tools

This transforms the interpreter from a toy to a production tool for real-world automation.

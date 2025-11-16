# Concurrency Roadmap: From Channels to Spawn

## Executive Summary

This document outlines the path from our current Go-style channels implementation to full concurrent execution with `spawn`. We analyze what needs to change, design trade-offs, and propose an elegant Lisp-first approach.

## Current State (V1)

### What Works ✅
- **Channels**: Thread-safe MPMC channels using Arc-wrapped crossbeam
- **Channel operations**: make-channel, channel-send, channel-recv, channel-close, channel?
- **Values**: Most values clone easily and are Send-compatible

### What Doesn't Work ❌
- **No spawn**: Can't execute code concurrently
- **Not Send/Sync**: Environment and Lambda values can't cross thread boundaries
- **Thread-local state**: Help registry, sandbox, test registry tied to main thread

## Architecture Analysis

### 1. Environment System (src/env.rs)

**Current Implementation:**
```rust
pub struct Environment {
    bindings: RefCell<HashMap<String, Value>>,  // ❌ Not Sync
    parent: Option<Rc<Environment>>,             // ❌ Not Send
}
```

**Problems:**
- `Rc<Environment>` - Not Send (can't move across threads)
- `RefCell<_>` - Not Sync (interior mutability without thread safety)
- Lambda closures capture `Rc<Environment>`, making them non-Send

**Solution Options:**

#### Option A: Arc + RwLock (Shared Mutable State)
```rust
pub struct Environment {
    bindings: RwLock<HashMap<String, Value>>,   // ✅ Sync
    parent: Option<Arc<Environment>>,            // ✅ Send + Sync
}
```
- **Pros**: True shared state across threads, familiar model
- **Cons**: Lock contention, potential deadlocks, performance overhead
- **Tradeoff**: Complexity vs. power

#### Option B: Arc + Immutable Snapshots (Erlang-style)
```rust
pub struct Environment {
    bindings: HashMap<String, Value>,            // Immutable
    parent: Option<Arc<Environment>>,
}
```
- **Pros**: No locks, no contention, simple reasoning, functional purity
- **Cons**: Can't share mutable state, need channels for communication
- **Tradeoff**: Simplicity vs. shared state

#### Option C: Hybrid (Recommended)
```rust
pub struct Environment {
    bindings: HashMap<String, Value>,            // Immutable snapshot
    parent: Option<Arc<Environment>>,
    shared: Arc<RwLock<HashMap<String, Arc<AtomicValue>>>>,  // Explicit shared state
}
```
- **Pros**: Immutable by default, opt-in shared state, clear semantics
- **Cons**: Two-tier system adds complexity
- **Tradeoff**: Balanced approach

### 2. Lambda Values (src/value.rs)

**Current Implementation:**
```rust
Lambda {
    params: Vec<String>,
    body: Box<Value>,
    env: Rc<Environment>,        // ❌ Not Send
    docstring: Option<String>,
}
```

**Solution:**
```rust
Lambda {
    params: Vec<String>,
    body: Box<Value>,
    env: Arc<Environment>,       // ✅ Send + Sync
    docstring: Option<String>,
}
```
- Straightforward change once Environment is Arc-based
- All existing code continues to work

### 3. MacroRegistry (src/macros.rs)

**Current Implementation:**
```rust
pub struct MacroRegistry {
    macros: HashMap<String, (Vec<String>, Value)>,  // Not thread-safe
}
```

**Two Approaches:**

#### Approach A: Thread-Local Macros
- Each thread gets its own macro registry
- Macros defined before spawn are cloned to new threads
- Simple, no synchronization needed
- Matches "snapshot" semantics

#### Approach B: Shared Macros
```rust
pub struct MacroRegistry {
    macros: Arc<RwLock<HashMap<String, (Vec<String>, Value)>>>,
}
```
- Macros can be defined at runtime across threads
- More complex, requires locking
- Probably overkill - macros are compile-time

**Recommendation:** Thread-local cloning (Approach A)

### 4. Thread-Local Storage

**Current Thread-Locals:**
- `HELP_REGISTRY` (src/help.rs) - Documentation
- `SANDBOX` (src/builtins/mod.rs) - I/O security
- `TEST_REGISTRY` (src/builtins/testing.rs) - Test framework
- `CURRENT_ENV` (src/parser.rs) - Symbol table for parsing

**Solutions:**

1. **Help Registry**: Make it Arc-based and clone to threads
   ```rust
   static HELP_REGISTRY: LazyLock<Arc<RwLock<HashMap<String, HelpEntry>>>> = ...;
   ```

2. **Sandbox**: Clone sandbox config to each thread
   ```rust
   // Each spawned thread gets its own Sandbox with same restrictions
   ```

3. **Test Registry**: Keep thread-local (tests don't need concurrency)

4. **Parser Env**: Already stateless, no changes needed

## Design Philosophy: The Lisp Way

### Principles

1. **Message Passing Over Shared State**
   - Channels are first-class (✅ already have)
   - Values are immutable by default
   - Communicate by sending, not by sharing

2. **Explicit Over Implicit**
   - Shared mutable state requires explicit opt-in (atoms/refs)
   - Default spawn behavior is "snapshot and isolate"
   - Clear semantics prevent race conditions

3. **Functional Purity**
   - Most code is pure and side-effect free
   - Side effects (I/O, mutation) are explicit
   - Concurrency doesn't break referential transparency

### Proposed Spawn Semantics

#### Basic Spawn (Immutable Snapshot)
```lisp
;; Child gets immutable snapshot of environment
;; Communicates via channels only
(define ch (make-channel))
(spawn (lambda ()
  (channel-send ch (* x 2))))  ; x captured from parent env
(channel-recv ch)
```

#### Spawn with Shared State (Explicit)
```lisp
;; Create explicit shared atomic reference
(define counter (atom 0))

;; Multiple spawns can access shared counter
(spawn (lambda ()
  (atom-swap! counter (lambda (x) (+ x 1)))))

(spawn (lambda ()
  (atom-swap! counter (lambda (x) (+ x 1)))))

;; Wait a bit, then read
(sleep 100)
(atom-deref counter)  ; => 2
```

### New Primitives for V2

#### Core Concurrency
- `spawn` - Execute lambda in new thread, returns nil
- `spawn-link` - Execute lambda, returns channel for result/errors

#### Shared State (Optional)
- `atom` - Create atomic reference (like Clojure)
- `atom-deref` - Read atom value
- `atom-swap!` - Update atom with function
- `atom-reset!` - Set atom value

#### Advanced
- `sleep` - Sleep current thread (milliseconds)
- `thread-id` - Get current thread ID
- `join` - Wait for spawned thread (if we track handles)

## Implementation Roadmap

### Phase 1: Environment Refactoring (Breaking Change)
1. Change `Rc<Environment>` → `Arc<Environment>`
2. Change `RefCell<HashMap>` → `HashMap` (immutable)
3. Update all environment creation/cloning
4. Fix Lambda to use Arc
5. Run full test suite

**Estimated Effort:** 2-4 hours
**Risk:** Medium (breaks all environment code)

### Phase 2: Thread-Local Elimination
1. Convert HELP_REGISTRY to Arc-based
2. Make Sandbox clonable/Arc-based
3. Test multi-threaded access patterns

**Estimated Effort:** 1-2 hours
**Risk:** Low

### Phase 3: Spawn Implementation
1. Implement basic spawn builtin
2. Clone environment snapshot for thread
3. Clone macro registry for thread
4. Handle thread lifecycle
5. Write comprehensive tests

**Estimated Effort:** 2-3 hours
**Risk:** Low (builds on previous phases)

### Phase 4: Shared State Primitives (Optional)
1. Implement Atom value type
2. Implement atom primitives
3. Add RwLock wrapper for Atom
4. Test concurrent modifications

**Estimated Effort:** 3-4 hours
**Risk:** Medium (new concurrency patterns)

### Phase 5: Testing & Documentation
1. Concurrency test suite (race conditions, deadlocks)
2. Update CLAUDE.md
3. Example programs (producer-consumer, parallel map, etc.)

**Estimated Effort:** 2-3 hours
**Risk:** Low

## Design Alternatives Considered

### Alternative 1: No Shared State (Pure Erlang)
- Only channels, no atoms/refs
- Simpler implementation
- More restrictive, less powerful
- **Rejected:** Too limiting for real-world use

### Alternative 2: Full STM (Software Transactional Memory)
- Clojure-style refs with transactions
- Very powerful, complex implementation
- **Rejected:** Too complex for initial version, can add later

### Alternative 3: Actor Model
- Spawn returns actor handle
- Send messages to actors (not just channels)
- Process mailboxes
- **Deferred:** Great idea, but channels are more primitive

## Code Examples: Before and After

### Current (V1) - Channels Only
```lisp
(define ch (make-channel 5))

;; Can't spawn, so sequential only
(channel-send ch 1)
(channel-send ch 2)
(channel-send ch 3)

(define sum 0)
(set! sum (+ sum (channel-recv ch)))
(set! sum (+ sum (channel-recv ch)))
(set! sum (+ sum (channel-recv ch)))
sum  ; => 6
```

### Future (V2) - With Spawn
```lisp
(define results (make-channel 3))

;; Spawn 3 workers in parallel
(spawn (lambda () (channel-send results (* 10 10))))
(spawn (lambda () (channel-send results (* 20 20))))
(spawn (lambda () (channel-send results (* 30 30))))

;; Collect results
(define sum 0)
(set! sum (+ sum (channel-recv results)))
(set! sum (+ sum (channel-recv results)))
(set! sum (+ sum (channel-recv results)))
sum  ; => 1400 (100 + 400 + 900)
```

### With Atoms (V2+)
```lisp
(define counter (atom 0))
(define workers 10)
(define results (make-channel workers))

;; Spawn 10 workers that share a counter
(let loop ((i 0))
  (if (< i workers)
    (begin
      (spawn (lambda ()
        (atom-swap! counter (lambda (x) (+ x 1)))
        (channel-send results #t)))
      (loop (+ i 1)))))

;; Wait for all workers
(let loop ((i 0))
  (if (< i workers)
    (begin
      (channel-recv results)
      (loop (+ i 1)))))

(atom-deref counter)  ; => 10
```

## Performance Considerations

### Arc vs Rc Overhead
- `Arc` has atomic reference counting (slightly slower)
- Negligible for most use cases (<5% overhead)
- Worth it for Send/Sync capability

### Lock Contention
- If using RwLock for atoms/shared state:
  - Readers can access concurrently (cheap)
  - Writers need exclusive access (expensive)
- Design programs to minimize shared mutable state

### Channel Performance
- Crossbeam channels are highly optimized
- Buffered channels reduce blocking
- MPMC support enables many patterns

### Thread Spawning Cost
- OS thread creation is ~100µs
- Consider thread pools for high-frequency spawning
- Could add `spawn-pool` later for optimization

## Migration Strategy

### Backward Compatibility
- Phases 1-2 are breaking changes
- All existing code continues to work after refactoring
- Channel API stays identical

### Testing Strategy
1. Run existing 281 tests after each phase
2. Add concurrency-specific tests (race detection)
3. Use `cargo test --release` for timing-sensitive tests
4. Consider `loom` for model checking (optional)

### Rollout Plan
1. Create `concurrency-v2` branch
2. Implement phases 1-3
3. Beta test with example programs
4. Merge to main when stable
5. Phases 4-5 can be incremental

## Open Questions

1. **Should spawn return anything?**
   - Option A: Returns nil (simple, matches Go)
   - Option B: Returns thread handle (enables join)
   - **Recommendation:** Start with nil, add handles later

2. **How to handle panics in spawned threads?**
   - Option A: Silent (thread dies, no notification)
   - Option B: Send error to channel (if linked)
   - Option C: Global panic handler
   - **Recommendation:** Option A for V2, add linking later

3. **Do we need thread pools?**
   - Probably not initially
   - Can add `spawn-pool` optimization later
   - Start simple with OS threads

4. **Should we support async/await?**
   - Interesting but very different model
   - Tokio integration would be complex
   - **Deferred:** OS threads are simpler

## Success Metrics

A successful V2 implementation will:
- ✅ Pass all existing 281 tests
- ✅ Allow spawning concurrent threads
- ✅ Maintain Lisp elegance and simplicity
- ✅ Enable real concurrent programs (web servers, parallel processing)
- ✅ Be well-documented with examples
- ✅ Have <10% performance regression for single-threaded code

## Conclusion

The path to concurrent execution is clear and achievable:

1. **Phase 1-2** (Environment refactoring): ~4-6 hours of focused work
2. **Phase 3** (Basic spawn): ~2-3 hours
3. **Total**: ~6-9 hours for working concurrent execution

The main barrier is the breaking change from Rc to Arc, but this is:
- Mechanically straightforward (search/replace + type fixes)
- Well-isolated (env.rs and value.rs)
- Low-risk (compiler will catch all issues)

The reward is a truly concurrent Lisp that maintains elegance through:
- Message passing over shared state
- Immutable snapshots by default
- Explicit shared state when needed
- Clear, predictable semantics

**Recommendation:** Proceed with Phases 1-3 as a unified effort. This gives us spawn with snapshot semantics, which unlocks 90% of concurrent programming use cases. Phase 4 (atoms) can be added later based on user needs.

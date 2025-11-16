# Concurrency V2: Production-Ready Design for LLM Automation

## Vision: A Concurrent Lisp for Real-World Automation

This isn't just academic concurrency - it's a practical tool for LLM-driven automation. The kind of tasks that need this:

```bash
# What LLMs do today (ad-hoc, error-prone):
curl api1 & curl api2 & curl api3 & wait
find . -name "*.log" -exec grep ERROR {} \; | parallel process
```

```lisp
;; What they should be able to do (elegant, safe):
(parallel-map http-get (list url1 url2 url3))
(parallel-grep "ERROR" (glob "**/*.log"))
```

## Design Principles

### 1. Safety First
- **Immutable by default** - no accidental shared state
- **Explicit sharing** - atoms/refs only when needed
- **Timeout everything** - no infinite hangs
- **Error propagation** - failures don't disappear

### 2. LLM-Friendly
- **Clear semantics** - easy to reason about
- **Composable** - primitives combine naturally
- **Debuggable** - good error messages
- **Predictable** - no race conditions by default

### 3. Production-Ready
- **Resource limits** - max threads, memory bounds
- **Graceful degradation** - handle failures
- **Observability** - track what's running
- **Backpressure** - don't overwhelm systems

## Core Primitives (Phase 1)

### spawn - Basic Concurrency
```lisp
;; Execute lambda in new thread, returns immediately
(spawn (lambda ()
  (channel-send results (http-get url))))

;; Captures environment snapshot (immutable)
(define timeout 5000)
(spawn (lambda ()
  (http-get url :timeout timeout)))  ; timeout captured
```

**Semantics:**
- Takes lambda with 0 arguments
- Captures immutable environment snapshot
- Returns nil immediately (fire-and-forget)
- Thread dies silently on error (V2.0)
- No return value (use channels to communicate)

### spawn-link - Supervised Concurrency
```lisp
;; Returns channel that receives result or error
(define result-ch (spawn-link (lambda ()
  (http-get url))))

(define result (channel-recv result-ch))
(if (error? result)
  (println "Failed:" (error-msg result))
  (println "Success:" result))
```

**Semantics:**
- Takes lambda with 0 arguments
- Returns channel for result
- Sends `{:ok value}` or `{:error message}` to channel
- Timeout support (optional)
- Errors don't crash parent thread

### parallel-map - Data Parallelism
```lisp
;; Map function over list in parallel
(define urls (list url1 url2 url3))
(define results (parallel-map http-get urls))
;; results = (list response1 response2 response3)

;; With error handling
(define results (parallel-map
  (lambda (url)
    (http-get url :timeout 5000))
  urls
  :max-threads 10))  ; Limit concurrency
```

**Semantics:**
- Spawns one thread per item (up to max-threads)
- Waits for all to complete
- Returns results in same order as inputs
- Propagates errors as error values
- Optional timeout per task

### parallel-filter - Concurrent Filtering
```lisp
;; Filter list using predicate in parallel
(define large-files
  (parallel-filter
    (lambda (path)
      (> (file-size path) 1000000))
    (glob "**/*.log")))
```

### timeout - Time-Bounded Execution
```lisp
;; Execute with timeout (milliseconds)
(define result
  (timeout 5000
    (lambda () (http-get slow-url))))

;; result is either the value or {:error "timeout"}
```

## Shared State Primitives (Phase 2)

### atom - Atomic References
```lisp
;; Create atomic reference
(define counter (atom 0))

;; Read atomically
(atom-deref counter)  ; => 0

;; Update atomically with function
(atom-swap! counter (lambda (x) (+ x 1)))
(atom-deref counter)  ; => 1

;; Set directly
(atom-reset! counter 100)
(atom-deref counter)  ; => 100
```

**Use Cases:**
- Shared counters
- Progress tracking
- Accumulating results
- Cache/memoization

### Example: Progress Counter
```lisp
(define processed (atom 0))
(define total (length urls))

(parallel-map
  (lambda (url)
    (define result (http-get url))
    (atom-swap! processed (lambda (x) (+ x 1)))
    (println "Progress:" (atom-deref processed) "/" total)
    result)
  urls)
```

## Advanced Primitives (Phase 3)

### select - Channel Multiplexing
```lisp
;; Wait on multiple channels, return first available
(define ch1 (spawn-link task1))
(define ch2 (spawn-link task2))

(define result (select ch1 ch2))
;; Returns {:channel ch1 :value val} or {:channel ch2 :value val}
```

### worker-pool - Thread Pool Pattern
```lisp
;; Create worker pool
(define pool (make-worker-pool 10))

;; Submit tasks
(worker-pool-submit pool (lambda () (process-file f1)))
(worker-pool-submit pool (lambda () (process-file f2)))

;; Wait for all and shutdown
(worker-pool-shutdown pool)
```

### stream-map - Lazy Parallel Processing
```lisp
;; Process stream of items in parallel
(define results-ch (stream-map
  process-item
  input-ch
  :workers 5))

;; Results arrive as they complete
(let loop ()
  (define result (channel-recv results-ch))
  (unless (error? result)
    (println result)
    (loop)))
```

## Real-World Examples

### Example 1: Parallel API Calls with Error Handling
```lisp
(define apis (list
  "https://api1.com/data"
  "https://api2.com/data"
  "https://api3.com/data"))

(define fetch-with-retry
  (lambda (url)
    (let retry ((attempts 3))
      (define result (timeout 5000
        (lambda () (http-get url))))
      (if (error? result)
        (if (> attempts 0)
          (retry (- attempts 1))
          result)
        result))))

(define results (parallel-map fetch-with-retry apis))

;; Filter successes
(define successes
  (filter (lambda (r) (not (error? r))) results))

(println "Got" (length successes) "out of" (length apis))
```

### Example 2: Concurrent File Processing
```lisp
(define log-files (glob "logs/**/*.log"))

(define process-log
  (lambda (path)
    (define content (read-file path))
    (define errors (filter
      (lambda (line) (string-contains? line "ERROR"))
      (string-lines content)))
    {:file path :errors (length errors)}))

;; Process 100 files at a time
(define results (parallel-map
  process-log
  log-files
  :max-threads 100))

;; Aggregate results
(define total-errors
  (reduce +
    (map (lambda (r) (map-get r :errors)) results)
    0))

(println "Total errors:" total-errors)
```

### Example 3: Producer-Consumer Pipeline
```lisp
(define work-queue (make-channel 100))
(define results (make-channel 100))

;; Producer: Read files and queue work
(spawn (lambda ()
  (define files (glob "data/*.csv"))
  (map (lambda (f) (channel-send work-queue f)) files)
  (channel-close work-queue)))

;; Consumers: Process files
(define workers 10)
(let loop ((i 0))
  (if (< i workers)
    (begin
      (spawn (lambda ()
        (let work-loop ()
          (define file (channel-recv work-queue))
          (unless (error? file)  ; Channel closed = error
            (define result (process-csv file))
            (channel-send results result)
            (work-loop)))))
      (loop (+ i 1)))))

;; Collector: Aggregate results
(let collect ((count 0))
  (if (< count (length files))
    (begin
      (define result (channel-recv results))
      (println "Processed:" result)
      (collect (+ count 1)))))
```

### Example 4: Rate-Limited API Scraping
```lisp
(define rate-limiter (atom {:count 0 :reset-time (now)}))
(define max-per-second 10)

(define rate-limited-fetch
  (lambda (url)
    ;; Check rate limit
    (atom-swap! rate-limiter
      (lambda (state)
        (define current-time (now))
        (define elapsed (- current-time (map-get state :reset-time)))
        (if (> elapsed 1000)  ; Reset every second
          {:count 1 :reset-time current-time}
          (if (< (map-get state :count) max-per-second)
            {:count (+ (map-get state :count) 1)
             :reset-time (map-get state :reset-time)}
            state))))

    ;; Wait if needed
    (let wait-loop ()
      (define state (atom-deref rate-limiter))
      (if (>= (map-get state :count) max-per-second)
        (begin
          (sleep 100)
          (wait-loop))))

    ;; Fetch
    (http-get url)))

(define urls (range 1 100))  ; 100 URLs
(define results (parallel-map rate-limited-fetch urls))
```

### Example 5: Concurrent Database Queries
```lisp
(define queries (list
  "SELECT COUNT(*) FROM users"
  "SELECT AVG(age) FROM users"
  "SELECT MAX(created_at) FROM users"))

(define results (parallel-map
  (lambda (query)
    {:query query
     :result (db-query db query)
     :time (now)})
  queries))

(map (lambda (r)
  (println (map-get r :query) "=>" (map-get r :result)))
  results)
```

## Implementation Architecture

### 1. Thread-Safe Environment (Arc-based)

```rust
pub struct Environment {
    bindings: HashMap<String, Value>,  // Immutable snapshot
    parent: Option<Arc<Environment>>,
}

impl Environment {
    // Returns new environment (functional)
    pub fn extend(&self, name: String, value: Value) -> Arc<Environment> {
        let mut new_bindings = self.bindings.clone();
        new_bindings.insert(name, value);
        Arc::new(Environment {
            bindings: new_bindings,
            parent: self.parent.clone(),
        })
    }
}
```

### 2. Spawn Implementation

```rust
fn spawn(args: &[Value]) -> Result<Value, EvalError> {
    let lambda = match &args[0] {
        Value::Lambda { params, body, env, .. } => {
            if !params.is_empty() {
                return Err(EvalError::Custom(
                    "spawn: lambda must take 0 arguments".to_string()
                ));
            }
            (body.clone(), Arc::clone(env))
        }
        _ => return Err(EvalError::Custom("spawn: expected lambda".to_string())),
    };

    thread::spawn(move || {
        let (body, env) = lambda;
        let mut macro_reg = MacroRegistry::new();
        let _ = eval::eval_with_macros(*body, env, &mut macro_reg);
    });

    Ok(Value::Nil)
}
```

### 3. Spawn-Link Implementation

```rust
fn spawn_link(args: &[Value]) -> Result<Value, EvalError> {
    let lambda = /* extract lambda */;
    let result_ch = make_channel_internal(1); // Buffered(1)

    let sender = result_ch.sender.clone();
    thread::spawn(move || {
        let (body, env) = lambda;
        let mut macro_reg = MacroRegistry::new();

        let result = match eval::eval_with_macros(*body, env, &mut macro_reg) {
            Ok(val) => create_map(vec![
                (":ok".to_string(), Value::Bool(true)),
                (":value".to_string(), val),
            ]),
            Err(e) => create_map(vec![
                (":error".to_string(), Value::String(format!("{:?}", e))),
            ]),
        };

        let _ = sender.send(result);
    });

    Ok(result_ch)
}
```

### 4. Atom Implementation

```rust
pub struct Atom {
    value: Arc<RwLock<Value>>,
}

fn atom_new(args: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::Atom {
        value: Arc::new(RwLock::new(args[0].clone()))
    })
}

fn atom_deref(args: &[Value]) -> Result<Value, EvalError> {
    match &args[0] {
        Value::Atom { value } => {
            Ok(value.read().unwrap().clone())
        }
        _ => Err(EvalError::Custom("atom-deref: expected atom".to_string())),
    }
}

fn atom_swap(args: &[Value]) -> Result<Value, EvalError> {
    let (atom, func) = /* extract */;

    match atom {
        Value::Atom { value } => {
            let mut guard = value.write().unwrap();
            let old_val = guard.clone();
            let new_val = apply_function(func, vec![old_val])?;
            *guard = new_val.clone();
            Ok(new_val)
        }
        _ => Err(EvalError::Custom("atom-swap!: expected atom".to_string())),
    }
}
```

## Error Handling Strategy

### 1. Errors are Values
```lisp
;; Errors don't throw, they return error values
(define result (http-get bad-url))
(error? result)  ; => #t
(error-msg result)  ; => "Connection refused"
```

### 2. Timeout Errors
```lisp
(define result (timeout 1000 (lambda () (http-get slow-url))))
;; result = {:error "timeout after 1000ms"}
```

### 3. Channel Errors
```lisp
(define ch (make-channel))
(channel-close ch)
(define val (channel-recv ch))
;; val = {:error "channel closed"}
```

### 4. Spawn-Link Errors
```lisp
(define result-ch (spawn-link (lambda ()
  (/ 1 0))))  ; Division by zero

(define result (channel-recv result-ch))
;; result = {:error "division by zero"}
```

## Resource Management

### 1. Thread Limits
```lisp
;; Global thread limit (set at startup)
(set-max-threads! 100)

;; Per-operation limits
(parallel-map fn items :max-threads 10)
```

### 2. Memory Limits
```lisp
;; Limit channel buffer sizes
(make-channel 1000)  ; Max 1000 items

;; Limit atom values (optional)
(atom {:max-size 1000000})
```

### 3. Timeout Defaults
```lisp
;; Set default timeout for operations
(set-default-timeout! 30000)  ; 30 seconds

;; Override per operation
(timeout 5000 (lambda () ...))
```

## Testing Strategy

### 1. Unit Tests
- Test each primitive in isolation
- Verify error handling
- Check resource cleanup

### 2. Integration Tests
- Test realistic workflows
- Verify no deadlocks
- Check error propagation

### 3. Stress Tests
- Spawn 1000s of threads
- Fill channels to capacity
- Concurrent atom updates

### 4. Race Condition Detection
- Use `loom` for model checking
- Property-based testing
- Randomized execution orders

## Migration Path

### Phase 1: Foundation (Week 1)
- [ ] Refactor Environment to Arc
- [ ] Make evaluator thread-safe
- [ ] Basic spawn implementation
- [ ] Update all 281 tests

### Phase 2: Core Primitives (Week 2)
- [ ] spawn-link with error handling
- [ ] parallel-map with limits
- [ ] timeout primitive
- [ ] Comprehensive testing

### Phase 3: Shared State (Week 3)
- [ ] Atom implementation
- [ ] atom-deref, atom-swap!, atom-reset!
- [ ] Thread-safe guarantees
- [ ] Performance testing

### Phase 4: Advanced Features (Week 4)
- [ ] select for channel multiplexing
- [ ] worker-pool pattern
- [ ] stream-map for lazy processing
- [ ] Real-world examples

### Phase 5: Production Hardening (Week 5)
- [ ] Resource limits enforcement
- [ ] Better error messages
- [ ] Performance optimization
- [ ] Documentation

## Success Metrics

A successful V2 will enable:
- ✅ Fetching 100 APIs in parallel (< 1 second vs 100 seconds)
- ✅ Processing 10k files concurrently (limited by CPU)
- ✅ Building real ETL pipelines
- ✅ Safe concurrent web scraping
- ✅ Parallel data analysis
- ✅ No deadlocks or race conditions
- ✅ Clear error messages
- ✅ Resource limits prevent runaway usage

## Comparison with Other Languages

### Go
```go
// Go version
results := make([]Result, len(urls))
var wg sync.WaitGroup
for i, url := range urls {
    wg.Add(1)
    go func(i int, url string) {
        defer wg.Done()
        results[i] = fetch(url)
    }(i, url)
}
wg.Wait()
```

```lisp
;; Our version (simpler!)
(define results (parallel-map http-get urls))
```

### Python asyncio
```python
# Python version
async def fetch_all(urls):
    tasks = [fetch(url) for url in urls]
    return await asyncio.gather(*tasks)
```

```lisp
;; Our version (no async/await complexity)
(define results (parallel-map http-get urls))
```

### Clojure
```clojure
;; Clojure version (very similar!)
(def results (pmap http-get urls))
```

```lisp
;; Our version
(define results (parallel-map http-get urls))
```

## Next Steps

1. **Start the refactoring** - Begin Phase 1 immediately
2. **Build incrementally** - Test after each change
3. **Real examples first** - Focus on practical use cases
4. **Iterate on API** - Adjust based on usage
5. **Document thoroughly** - Clear examples for every primitive

This is the foundation for a truly useful concurrent Lisp that LLMs can leverage for real automation tasks!

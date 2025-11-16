# Spawn Implementation Proof of Concept

## Key Changes Required

This document shows the specific code changes needed to enable `spawn`.

## 1. Environment Refactoring (env.rs)

### Before (Current - Rc/RefCell)
```rust
use std::cell::RefCell;
use std::rc::Rc;

pub struct Environment {
    bindings: RefCell<HashMap<String, Value>>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Rc<Self> {
        Rc::new(Environment {
            bindings: RefCell::new(HashMap::new()),
            parent: None,
        })
    }

    pub fn with_parent(parent: Rc<Environment>) -> Rc<Self> {
        Rc::new(Environment {
            bindings: RefCell::new(HashMap::new()),
            parent: Some(parent),
        })
    }

    pub fn define(&self, name: String, value: Value) {
        self.bindings.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.bindings.borrow().get(name) {
            return Some(value.clone());
        }
        if let Some(ref parent) = self.parent {
            return parent.get(name);
        }
        None
    }
}
```

### After (Immutable Snapshot - Arc only)
```rust
use std::sync::Arc;

pub struct Environment {
    bindings: HashMap<String, Value>,      // Immutable!
    parent: Option<Arc<Environment>>,
}

impl Environment {
    pub fn new() -> Arc<Self> {
        Arc::new(Environment {
            bindings: HashMap::new(),
            parent: None,
        })
    }

    pub fn with_parent(parent: Arc<Environment>) -> Arc<Self> {
        Arc::new(Environment {
            bindings: HashMap::new(),
            parent: Some(parent),
        })
    }

    // Changed: Returns new environment with binding added
    pub fn extend(&self, name: String, value: Value) -> Arc<Environment> {
        let mut new_bindings = self.bindings.clone();
        new_bindings.insert(name, value);
        Arc::new(Environment {
            bindings: new_bindings,
            parent: self.parent.clone(),
        })
    }

    // Same as before - reading is still simple
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.bindings.get(name) {
            return Some(value.clone());
        }
        if let Some(ref parent) = self.parent {
            return parent.get(name);
        }
        None
    }
}
```

**Key Changes:**
1. `Rc` → `Arc` (Send + Sync)
2. `RefCell<HashMap>` → `HashMap` (immutable)
3. `define()` → `extend()` (returns new env)
4. `get()` stays the same!

**Impact:**
- ✅ `Arc<Environment>` is Send + Sync
- ✅ No locks needed (immutable)
- ✅ Safe to clone across threads
- ❌ Can't mutate in place (need new pattern)

## 2. Define Becomes Functional (eval.rs)

### Before
```rust
fn eval_define(args: &[Value], env: Rc<Environment>) -> Result<Value, EvalError> {
    // ... validation ...
    let value = eval(args[1].clone(), env.clone())?;
    env.define(name, value.clone());  // Mutates env in place
    Ok(value)
}
```

### After
```rust
fn eval_define(
    args: &[Value],
    env: Arc<Environment>,
) -> Result<(Value, Arc<Environment>), EvalError> {
    // ... validation ...
    let value = eval(args[1].clone(), env.clone())?;
    let new_env = env.extend(name, value.clone());  // Returns new env
    Ok((value, new_env))
}
```

**Key Insight:** Evaluator becomes a state transformer
- Input: (Expression, Environment)
- Output: (Value, NewEnvironment)
- Functional programming pattern!

## 3. Lambda Captures Arc (value.rs)

### Before
```rust
Lambda {
    params: Vec<String>,
    body: Box<Value>,
    env: Rc<Environment>,        // ❌ Not Send
    docstring: Option<String>,
}
```

### After
```rust
Lambda {
    params: Vec<String>,
    body: Box<Value>,
    env: Arc<Environment>,       // ✅ Send + Sync
    docstring: Option<String>,
}
```

**That's it!** Once Environment is Arc, Lambda becomes Send automatically.

## 4. Spawn Implementation (builtins/concurrency.rs)

```rust
use std::thread;

#[builtin(
    name = "spawn",
    signature = "(spawn function)",
    description = "Execute function concurrently in a new thread",
    category = "Concurrency"
)]
fn spawn_fn(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::Custom("spawn: expected 1 argument".to_string()));
    }

    match &args[0] {
        Value::Lambda { params, body, env, .. } => {
            // Clone everything needed for the thread
            let params = params.clone();
            let body = body.clone();
            let env = Arc::clone(env);  // ✅ Arc clones are cheap and Send

            // Spawn OS thread
            thread::spawn(move || {
                // Create a new macro registry for this thread
                let mut macro_reg = MacroRegistry::new();

                // Evaluate the lambda body in the captured environment
                if params.is_empty() {
                    let _ = eval::eval_with_macros(*body, env, &mut macro_reg);
                } else {
                    eprintln!("spawn: lambda parameters not yet supported");
                }
            });

            Ok(Value::Nil)
        }
        _ => Err(EvalError::Custom(
            "spawn: argument must be a lambda".to_string()
        )),
    }
}
```

**That's the whole implementation!**

## 5. Example Usage

```lisp
;; Define a shared channel
(define results (make-channel 3))

;; Spawn 3 concurrent workers
(spawn (lambda ()
  (channel-send results (* 10 10))))

(spawn (lambda ()
  (channel-send results (* 20 20))))

(spawn (lambda ()
  (channel-send results (* 30 30))))

;; Collect results (order may vary!)
(println (channel-recv results))  ; Might print: 100
(println (channel-recv results))  ; Might print: 900
(println (channel-recv results))  ; Might print: 400
```

## 6. Migration Pattern

The refactoring follows a mechanical pattern:

### Pattern 1: Environment Usage
```rust
// Before
let env = Environment::new();
env.define("x".to_string(), Value::Number(42.0));

// After
let env = Environment::new();
let env = env.extend("x".to_string(), Value::Number(42.0));
```

### Pattern 2: Evaluator Functions
```rust
// Before
fn eval(expr: Value, env: Rc<Environment>) -> Result<Value, EvalError>

// After
fn eval(expr: Value, env: Arc<Environment>) -> Result<Value, EvalError>
```

### Pattern 3: Lambda Creation
```rust
// Before
Value::Lambda {
    params,
    body,
    env: Rc::clone(&env),
    docstring,
}

// After
Value::Lambda {
    params,
    body,
    env: Arc::clone(&env),  // Same syntax!
    docstring,
}
```

## 7. Testing Strategy

### Test 1: Basic Spawn
```rust
#[test]
fn test_spawn_basic() {
    let env = setup();
    let code = r#"
        (begin
            (define ch (make-channel))
            (spawn (lambda () (channel-send ch 42)))
            (channel-recv ch))
    "#;
    let result = eval_expr(code, env).unwrap();
    assert!(matches!(result, Value::Number(n) if n == 42.0));
}
```

### Test 2: Multiple Spawns
```rust
#[test]
fn test_spawn_multiple() {
    let env = setup();
    let code = r#"
        (begin
            (define ch (make-channel 3))
            (spawn (lambda () (channel-send ch 1)))
            (spawn (lambda () (channel-send ch 2)))
            (spawn (lambda () (channel-send ch 3)))
            (define a (channel-recv ch))
            (define b (channel-recv ch))
            (define c (channel-recv ch))
            (+ a (+ b c)))
    "#;
    let result = eval_expr(code, env).unwrap();
    assert!(matches!(result, Value::Number(n) if n == 6.0));
}
```

### Test 3: Captured Variables
```rust
#[test]
fn test_spawn_closure() {
    let env = setup();
    let code = r#"
        (begin
            (define x 100)
            (define ch (make-channel))
            (spawn (lambda () (channel-send ch (* x 2))))
            (channel-recv ch))
    "#;
    let result = eval_expr(code, env).unwrap();
    assert!(matches!(result, Value::Number(n) if n == 200.0));
}
```

## 8. Performance Impact

### Memory
- Arc has 2 atomic counters vs Rc's 1 counter
- Overhead: ~16 bytes per Environment
- Negligible for typical programs

### Speed
- Atomic operations are slightly slower than non-atomic
- Impact: ~2-5% for single-threaded code
- Trade-off: Worth it for concurrency capability

### Cloning
- `Arc::clone(&env)` is O(1) - just increments counter
- Cheap enough to clone freely
- No deep copying of HashMap

## 9. Alternative: Keep Both

If we want to minimize breaking changes, we could support both:

```rust
#[cfg(feature = "concurrent")]
type EnvRef = Arc<Environment>;

#[cfg(not(feature = "concurrent"))]
type EnvRef = Rc<Environment>;
```

**Pros:**
- No breaking changes for users who don't need concurrency
- Can opt-in via feature flag

**Cons:**
- Maintaining two code paths
- Testing complexity
- Confusing for users

**Recommendation:** Just use Arc always. Simpler.

## 10. Rollout Checklist

- [ ] Phase 1: Refactor env.rs to use Arc + immutable HashMap
- [ ] Phase 2: Update all env.define() calls to use extend()
- [ ] Phase 3: Update Value::Lambda to use Arc<Environment>
- [ ] Phase 4: Fix all compilation errors (type checker helps!)
- [ ] Phase 5: Run test suite, fix any logic errors
- [ ] Phase 6: Implement spawn builtin
- [ ] Phase 7: Write concurrency tests
- [ ] Phase 8: Update documentation
- [ ] Phase 9: Create example programs
- [ ] Phase 10: Merge and release

**Estimated Total Time:** 6-10 hours of focused work

## Conclusion

The changes needed are:
1. **Mechanical** - Search/replace Rc→Arc in most cases
2. **Architectural** - Immutable environments (bigger shift)
3. **Straightforward** - Compiler guides you through errors

The result is a truly concurrent Lisp interpreter that maintains elegance through functional patterns and message passing.

**Next Steps:**
1. Create `concurrency-v2` branch
2. Start with env.rs refactoring
3. Let the compiler be your guide
4. Test incrementally
5. Enjoy parallel Lisp!

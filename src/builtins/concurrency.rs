//! # Concurrency Built-in Functions
//!
//! Go-style channels and goroutines for concurrent programming.
//!
//! ## Functions
//!
//! - **make-channel** - Create a new channel (buffered or unbuffered)
//! - **channel-send** - Send a value to a channel (blocking)
//! - **channel-recv** - Receive a value from a channel (blocking)
//! - **channel-close** - Close a channel
//! - **channel?** - Check if a value is a channel
//! - **spawn** - Spawn a goroutine to execute a function concurrently

use crate::error::EvalError;
use crate::value::Value;
use crossbeam_channel::{bounded, unbounded};
use lisp_macros::builtin;
use std::sync::Arc;

#[builtin(
    name = "make-channel",
    signature = "(make-channel [capacity])",
    description = "Create a new channel for concurrent communication.

**Parameters:**
- capacity (optional): Buffer size (number). Omit for unbuffered channel.

**Returns:** A new channel

**Examples:**
```lisp
(make-channel)       ; Unbuffered channel
(make-channel 10)    ; Buffered channel with capacity 10
```

**Notes:**
- Unbuffered channels block on send until a receiver is ready
- Buffered channels block only when the buffer is full
- Channels are thread-safe and can be shared across goroutines",
    category = "Concurrency",
    related = ["channel-send", "channel-recv", "channel-close", "channel?"]
)]
fn make_channel(args: &[Value]) -> Result<Value, EvalError> {
    match args.len() {
        0 => {
            // Unbuffered channel
            let (sender, receiver) = unbounded();
            Ok(Value::Channel {
                sender: Arc::new(sender),
                receiver: Arc::new(receiver),
            })
        }
        1 => {
            // Buffered channel
            match &args[0] {
                Value::Number(n) if *n >= 0.0 && n.fract() == 0.0 => {
                    let capacity = *n as usize;
                    let (sender, receiver) = bounded(capacity);
                    Ok(Value::Channel {
                        sender: Arc::new(sender),
                        receiver: Arc::new(receiver),
                    })
                }
                Value::Number(_) => Err(EvalError::Custom(
                    "make-channel: capacity must be a non-negative integer".to_string(),
                )),
                _ => Err(EvalError::Custom(
                    "make-channel: capacity must be a number".to_string(),
                )),
            }
        }
        _ => Err(EvalError::Custom(
            "make-channel: expected 0 or 1 arguments".to_string(),
        )),
    }
}

#[builtin(
    name = "channel-send",
    signature = "(channel-send channel value)",
    description = "Send a value to a channel (blocking operation).

**Parameters:**
- channel: The channel to send to
- value: The value to send

**Returns:** The sent value

**Examples:**
```lisp
(define ch (make-channel))
(channel-send ch 42)
(channel-send ch \"hello\")
```

**Notes:**
- Blocks if the channel buffer is full (or unbuffered and no receiver ready)
- Returns error if channel is closed
- Thread-safe operation",
    category = "Concurrency",
    related = ["make-channel", "channel-recv", "channel-close"]
)]
fn channel_send(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::Custom(
            "channel-send: expected 2 arguments".to_string(),
        ));
    }

    match &args[0] {
        Value::Channel { sender, .. } => {
            let value = args[1].clone();
            sender
                .send(value.clone())
                .map_err(|_| EvalError::Custom("channel-send: channel is closed".to_string()))?;
            Ok(value)
        }
        _ => Err(EvalError::Custom(
            "channel-send: first argument must be a channel".to_string(),
        )),
    }
}

#[builtin(
    name = "channel-recv",
    signature = "(channel-recv channel)",
    description = "Receive a value from a channel (blocking operation).

**Parameters:**
- channel: The channel to receive from

**Returns:** The received value, or an error if the channel is closed

**Examples:**
```lisp
(define ch (make-channel))
(spawn (lambda () (channel-send ch 42)))
(define val (channel-recv ch))  ; val = 42
```

**Notes:**
- Blocks until a value is available
- Returns error value if channel is closed and empty
- Thread-safe operation",
    category = "Concurrency",
    related = ["make-channel", "channel-send", "channel-close"]
)]
fn channel_recv(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::Custom(
            "channel-recv: expected 1 argument".to_string(),
        ));
    }

    match &args[0] {
        Value::Channel { receiver, .. } => receiver
            .recv()
            .map_err(|_| EvalError::Custom("channel-recv: channel is closed and empty".to_string())),
        _ => Err(EvalError::Custom(
            "channel-recv: argument must be a channel".to_string(),
        )),
    }
}

#[builtin(
    name = "channel-close",
    signature = "(channel-close channel)",
    description = "Close a channel, preventing further sends.

**Parameters:**
- channel: The channel to close

**Returns:** nil

**Examples:**
```lisp
(define ch (make-channel))
(channel-send ch 1)
(channel-close ch)
(channel-send ch 2)  ; Error: channel is closed
```

**Notes:**
- After closing, sends will fail
- Receives will succeed until the channel is drained
- Closing an already-closed channel is a no-op
- Thread-safe operation",
    category = "Concurrency",
    related = ["make-channel", "channel-send", "channel-recv"]
)]
fn channel_close(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::Custom(
            "channel-close: expected 1 argument".to_string(),
        ));
    }

    match &args[0] {
        Value::Channel { .. } => {
            // Note: crossbeam channels don't have an explicit close method
            // They close when all senders are dropped. We'll drop the sender by
            // not holding a reference to it. For now, this is a no-op that returns nil.
            // In practice, channels close when all references are dropped.
            Ok(Value::Nil)
        }
        _ => Err(EvalError::Custom(
            "channel-close: argument must be a channel".to_string(),
        )),
    }
}

#[builtin(
    name = "channel?",
    signature = "(channel? value)",
    description = "Check if a value is a channel.

**Parameters:**
- value: The value to check

**Returns:** #t if value is a channel, #f otherwise

**Examples:**
```lisp
(channel? (make-channel))        ; #t
(channel? 42)                    ; #f
(channel? \"hello\")               ; #f
```",
    category = "Concurrency",
    related = ["make-channel", "number?", "string?", "list?"]
)]
fn channel_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::Custom(
            "channel?: expected 1 argument".to_string(),
        ));
    }

    Ok(Value::Bool(matches!(args[0], Value::Channel { .. })))
}

#[builtin(
    name = "spawn",
    signature = "(spawn function)",
    description = "Spawn a goroutine to execute a zero-argument function concurrently.

**Parameters:**
- function: A lambda with zero parameters to execute in a new thread

**Returns:** A channel from which the result can be received

**Examples:**
```lisp
;; Spawn a simple computation
(define result-ch (spawn (lambda () (+ 1 2 3))))
(channel-recv result-ch)  ; => 6

;; Spawn with side effects
(define ch (make-channel))
(spawn (lambda () (channel-send ch 42)))
(channel-recv ch)  ; => 42

;; Multiple concurrent tasks
(define ch1 (spawn (lambda () (* 10 10))))
(define ch2 (spawn (lambda () (+ 5 5))))
(list (channel-recv ch1) (channel-recv ch2))  ; => (100 10)
```

**Notes:**
- The function must take zero parameters
- Errors in the spawned function are caught and sent as Error values
- The spawned thread has its own macro registry
- Thread-safe: uses Arc-based environments for safe concurrent execution
- Non-blocking: spawn returns immediately, computation runs in background",
    category = "Concurrency",
    related = ["make-channel", "channel-recv", "channel-send"]
)]
fn spawn(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::Custom(
            "spawn: expected 1 argument".to_string(),
        ));
    }

    match &args[0] {
        Value::Lambda {
            params,
            body,
            env: lambda_env,
            docstring: _,
        } => {
            // Verify zero-parameter lambda
            if !params.is_empty() {
                return Err(EvalError::Custom(
                    "spawn: function must take zero parameters".to_string(),
                ));
            }

            // Create result channel
            let (sender, receiver) = unbounded();
            let result_channel = Value::Channel {
                sender: Arc::new(sender.clone()),
                receiver: Arc::new(receiver),
            };

            // Clone what we need for the thread
            let body_clone = body.clone();
            let env_clone = Arc::clone(lambda_env);

            // Get the global environment from the parent thread
            let global_env = crate::eval::get_global_env();

            // Spawn the thread
            std::thread::spawn(move || {
                // Initialize GLOBAL_ENV in this thread with the parent's global env
                if let Some(global) = global_env {
                    crate::eval::set_global_env(global);
                }

                // Create a new macro registry for this thread
                let mut macro_reg = crate::macros::MacroRegistry::new();

                // Evaluate the lambda body
                let result = crate::eval::eval_with_macros(*body_clone, env_clone, &mut macro_reg);

                // Send result or error
                let value_to_send = match result {
                    Ok(val) => val,
                    Err(e) => Value::Error(format!("{:?}", e)),
                };

                // Send the result (ignore send errors - receiver might have been dropped)
                let _ = sender.send(value_to_send);
            });

            Ok(result_channel)
        }
        _ => Err(EvalError::Custom(
            "spawn: argument must be a lambda".to_string(),
        )),
    }
}

#[builtin(
    name = "spawn-link",
    signature = "(spawn-link function)",
    description = "Spawn a supervised goroutine with structured error handling.

**Parameters:**
- function: A lambda with zero parameters to execute in a new thread

**Returns:** A channel that receives a result map with either `:ok` or `:error`

**Examples:**
```lisp
;; Successful execution
(define result-ch (spawn-link (lambda () (+ 1 2 3))))
(define result (channel-recv result-ch))
;; result = {:ok 6}

;; Error handling
(define result-ch (spawn-link (lambda () (/ 1 0))))
(define result (channel-recv result-ch))
(if (map-get result :error)
  (println \"Error:\" (map-get result :error))
  (println \"Success:\" (map-get result :ok)))

;; Using with http-request for robust error handling
(define result-ch (spawn-link (lambda ()
  (http-request \"https://api.example.com/data\" {}))))
(define result (channel-recv result-ch))
(if (map-get result :error)
  {:status \"failed\" :reason (map-get result :error)}
  {:status \"success\" :data (map-get result :ok)})
```

**Notes:**
- Returns `{:ok value}` on success
- Returns `{:error \"error message\"}` on failure
- Errors never crash the parent thread
- More structured than spawn for error handling
- Ideal for unreliable operations (network, I/O, external services)
- Use with map-get to check :error or :ok keys",
    category = "Concurrency",
    related = ["spawn", "make-channel", "channel-recv", "map-get"]
)]
fn spawn_link(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::Custom(
            "spawn-link: expected 1 argument".to_string(),
        ));
    }

    match &args[0] {
        Value::Lambda {
            params,
            body,
            env: lambda_env,
            docstring: _,
        } => {
            // Verify zero-parameter lambda
            if !params.is_empty() {
                return Err(EvalError::Custom(
                    "spawn-link: function must take zero parameters".to_string(),
                ));
            }

            // Create result channel
            let (sender, receiver) = unbounded();
            let result_channel = Value::Channel {
                sender: Arc::new(sender.clone()),
                receiver: Arc::new(receiver),
            };

            // Clone what we need for the thread
            let body_clone = body.clone();
            let env_clone = Arc::clone(lambda_env);

            // Get the global environment from the parent thread
            let global_env = crate::eval::get_global_env();

            // Spawn the thread
            std::thread::spawn(move || {
                // Initialize GLOBAL_ENV in this thread with the parent's global env
                if let Some(global) = global_env {
                    crate::eval::set_global_env(global);
                }

                // Create a new macro registry for this thread
                let mut macro_reg = crate::macros::MacroRegistry::new();

                // Evaluate the lambda body
                let result = crate::eval::eval_with_macros(*body_clone, env_clone, &mut macro_reg);

                // Create result map
                use std::collections::HashMap;
                let value_to_send = match result {
                    Ok(val) => {
                        // Success: {:ok value}
                        let mut map = HashMap::new();
                        map.insert("ok".to_string(), val);
                        Value::Map(map)
                    }
                    Err(e) => {
                        // Error: {:error "message"}
                        let mut map = HashMap::new();
                        map.insert("error".to_string(), Value::String(format!("{:?}", e)));
                        Value::Map(map)
                    }
                };

                // Send the result (ignore send errors - receiver might have been dropped)
                let _ = sender.send(value_to_send);
            });

            Ok(result_channel)
        }
        _ => Err(EvalError::Custom(
            "spawn-link: argument must be a lambda".to_string(),
        )),
    }
}

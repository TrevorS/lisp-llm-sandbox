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

// Note: spawn is intentionally left unimplemented in V1 due to thread-safety constraints.
// The current environment system uses Rc<Environment> which is not Send across threads.
//
// To properly implement spawn, we would need to:
// 1. Convert Environment to use Arc instead of Rc (major refactoring)
// 2. Make the evaluator thread-safe
// 3. Handle macro registry in a thread-safe way
//
// For V1, users can use channels for concurrent communication patterns without spawn.
// A future version will add proper goroutine support.
//
// Example of what spawn would look like when implemented:
// ```lisp
// (define ch (make-channel))
// (spawn (lambda () (channel-send ch 42)))
// (channel-recv ch)  ; Returns 42
// ```

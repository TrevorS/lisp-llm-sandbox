//! Arithmetic operations: +, -, *, /, %
//!
//! Basic mathematical operations supporting variadic arguments where applicable.
//!
//! - `+`: Sum of all arguments (identity: 0)
//! - `-`: Subtract subsequent args from first, or negate if single arg
//! - `*`: Product of all arguments (identity: 1)
//! - `/`: Divide first by subsequent args, or reciprocal if single arg
//! - `%`: Remainder operation (modulo) - exactly 2 args required

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;

#[builtin(name = "+", category = "Arithmetic", related(-, *, /))]
/// Returns the sum of all arguments.
///
/// # Examples
///
/// ```lisp
/// (+ 1 2 3) => 6
/// (+ 10) => 10
/// (+) => 0
/// ```
///
/// # See Also
///
/// -, *, /
pub fn builtin_add(args: &[Value]) -> Result<Value, EvalError> {
    let mut sum = 0.0;
    for arg in args {
        match arg {
            Value::Number(n) => sum += n,
            _ => return Err(EvalError::TypeError),
        }
    }
    Ok(Value::Number(sum))
}

#[builtin(name = "-", category = "Arithmetic", related(+, *, /))]
/// Subtracts subsequent arguments from the first.
///
/// With one argument, returns its negation.
///
/// # Examples
///
/// ```lisp
/// (- 10 3 2) => 5
/// (- 5) => -5
/// ```
///
/// # See Also
///
/// +, *, /
pub fn builtin_sub(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::ArityMismatch);
    }

    let first = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    if args.len() == 1 {
        return Ok(Value::Number(-first));
    }

    let mut result = first;
    for arg in &args[1..] {
        match arg {
            Value::Number(n) => result -= n,
            _ => return Err(EvalError::TypeError),
        }
    }
    Ok(Value::Number(result))
}

#[builtin(name = "*", category = "Arithmetic", related(+, -, /))]
/// Returns the product of all arguments.
///
/// # Examples
///
/// ```lisp
/// (* 2 3 4) => 24
/// (* 5) => 5
/// (*) => 1
/// ```
///
/// # See Also
///
/// +, -, /
pub fn builtin_mul(args: &[Value]) -> Result<Value, EvalError> {
    let mut product = 1.0;
    for arg in args {
        match arg {
            Value::Number(n) => product *= n,
            _ => return Err(EvalError::TypeError),
        }
    }
    Ok(Value::Number(product))
}

#[builtin(name = "/", category = "Arithmetic", related(+, -, *, %))]
/// Divides the first argument by subsequent arguments.
///
/// Integer division in Lisp.
///
/// # Examples
///
/// ```lisp
/// (/ 20 4) => 5
/// (/ 100 2 5) => 10
/// ```
///
/// # See Also
///
/// +, -, *, %
pub fn builtin_div(args: &[Value]) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::ArityMismatch);
    }

    let first = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    if args.len() == 1 {
        if first == 0.0 {
            return Err(EvalError::Custom("Division by zero".to_string()));
        }
        return Ok(Value::Number(1.0 / first));
    }

    let mut result = first;
    for arg in &args[1..] {
        match arg {
            Value::Number(n) => {
                if *n == 0.0 {
                    return Err(EvalError::Custom("Division by zero".to_string()));
                }
                result /= n;
            }
            _ => return Err(EvalError::TypeError),
        }
    }
    Ok(Value::Number(result))
}

#[builtin(name = "%", category = "Arithmetic", related(/))]
/// Returns the remainder when num1 is divided by num2.
///
/// # Examples
///
/// ```lisp
/// (% 17 5) => 2
/// (% 10 3) => 1
/// ```
///
/// # See Also
///
/// /
pub fn builtin_mod(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let a = match args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    let b = match args[1] {
        Value::Number(n) => {
            if n == 0.0 {
                return Err(EvalError::Custom("Division by zero".to_string()));
            }
            n
        }
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Number(a % b))
}

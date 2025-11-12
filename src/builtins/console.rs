//! Console I/O operations: print, println
//!
//! Functions for output to standard output.
//!
//! - `print`: Output value without newline
//! - `println`: Output value with trailing newline
//!
//! Both return nil

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;

#[builtin(name = "print", category = "Console I/O", related(println))]
/// Prints values to stdout without newline. Returns nil.
///
/// # Examples
///
/// ```lisp
/// (print "hello") => outputs: hello
/// (print 1 2 3) => outputs: 1 2 3
/// ```
///
/// # See Also
///
/// println
pub fn builtin_print(args: &[Value]) -> Result<Value, EvalError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        match arg {
            Value::String(s) => print!("{}", s),
            other => print!("{}", other),
        }
    }
    Ok(Value::Nil)
}

#[builtin(name = "println", category = "Console I/O", related(print))]
/// Prints values to stdout with newline at end. Returns nil.
///
/// # Examples
///
/// ```lisp
/// (println "hello") => outputs: hello
/// (println "a" "b") => outputs: a b
/// ```
///
/// # See Also
///
/// print
pub fn builtin_println(args: &[Value]) -> Result<Value, EvalError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        match arg {
            Value::String(s) => print!("{}", s),
            other => print!("{}", other),
        }
    }
    println!();
    Ok(Value::Nil)
}

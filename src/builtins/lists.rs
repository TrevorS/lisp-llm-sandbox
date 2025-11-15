//! List operations: cons, car, cdr, list, length, empty?
//!
//! Functions for building and manipulating lists.
//!
//! - `cons`: Construct a list by prepending element to list
//! - `car`: Get first element of list
//! - `cdr`: Get rest of list (all but first element)
//! - `list`: Create a list from arguments
//! - `length`: Get number of elements in list
//! - `empty?`: Test if list is empty

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;

#[builtin(name = "cons", category = "List operations", related(car, cdr, list))]
/// Constructs a new list by prepending elem to list.
///
/// Returns a new list; original is not modified.
///
/// # Examples
///
/// ```lisp
/// (cons 1 '(2 3)) => (1 2 3)
/// (cons 'a '(b c)) => (a b c)
/// (cons 1 nil) => (1)
/// ```
///
/// # See Also
///
/// car, cdr, list
pub fn builtin_cons(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::arity_error("cons", "2", args.len()));
    }

    let mut result = vec![args[0].clone()];

    match &args[1] {
        Value::List(items) => result.extend(items.clone()),
        Value::Nil => (),
        _ => return Err(EvalError::type_error("cons", "list", &args[1], 2)),
    }

    Ok(Value::List(result))
}

#[builtin(name = "car", category = "List operations", related(cdr, cons))]
/// Returns the first element of a list. Also called 'head'.
///
/// Throws error on empty list or non-list.
///
/// # Examples
///
/// ```lisp
/// (car '(1 2 3)) => 1
/// (car '(a)) => a
/// ```
///
/// # See Also
///
/// cdr, cons
pub fn builtin_car(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_error("car", "1", args.len()));
    }

    match &args[0] {
        Value::List(items) if !items.is_empty() => Ok(items[0].clone()),
        Value::List(_) => Err(EvalError::runtime_error("car", "empty list")),
        _ => Err(EvalError::type_error("car", "list", &args[0], 1)),
    }
}

#[builtin(name = "cdr", category = "List operations", related(car, cons))]
/// Returns all elements except the first. Also called 'tail'.
///
/// Returns nil for single-element list.
///
/// # Examples
///
/// ```lisp
/// (cdr '(1 2 3)) => (2 3)
/// (cdr '(a b)) => (b)
/// (cdr '(1)) => nil
/// ```
///
/// # See Also
///
/// car, cons
pub fn builtin_cdr(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_error("cdr", "1", args.len()));
    }

    match &args[0] {
        Value::List(items) if !items.is_empty() => {
            if items.len() == 1 {
                Ok(Value::Nil)
            } else {
                Ok(Value::List(items[1..].to_vec()))
            }
        }
        Value::List(_) => Err(EvalError::runtime_error("cdr", "empty list")),
        _ => Err(EvalError::type_error("cdr", "list", &args[0], 1)),
    }
}

#[builtin(name = "list", category = "List operations", related(cons, car, cdr))]
/// Creates a new list containing the given elements in order.
///
/// # Examples
///
/// ```lisp
/// (list 1 2 3) => (1 2 3)
/// (list 'a 'b) => (a b)
/// (list) => nil
/// ```
///
/// # See Also
///
/// cons, car, cdr
pub fn builtin_list(args: &[Value]) -> Result<Value, EvalError> {
    Ok(Value::List(args.to_vec()))
}

#[builtin(name = "length", category = "List operations", related(empty?, list))]
/// Returns the number of elements in a list.
///
/// # Examples
///
/// ```lisp
/// (length '(1 2 3)) => 3
/// (length '()) => 0
/// (length '(a)) => 1
/// ```
///
/// # See Also
///
/// empty?, list
pub fn builtin_length(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_error("length", "1", args.len()));
    }

    match &args[0] {
        Value::List(items) => Ok(Value::Number(items.len() as f64)),
        Value::Nil => Ok(Value::Number(0.0)),
        _ => Err(EvalError::type_error("length", "list", &args[0], 1)),
    }
}

#[builtin(name = "empty?", category = "List operations", related(length, nil?))]
/// Tests if a list is empty (nil or ()).
///
/// Returns #t for empty lists, #f otherwise.
///
/// # Examples
///
/// ```lisp
/// (empty? nil) => #t
/// (empty? '()) => #t
/// (empty? '(1)) => #f
/// ```
///
/// # See Also
///
/// length, nil?
pub fn builtin_empty_q(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_error("empty?", "1", args.len()));
    }

    match &args[0] {
        Value::List(items) => Ok(Value::Bool(items.is_empty())),
        Value::Nil => Ok(Value::Bool(true)),
        _ => Err(EvalError::type_error("empty?", "list", &args[0], 1)),
    }
}

//! Type predicates: number?, string?, list?, nil?, symbol?, bool?, map?, keyword?
//!
//! Functions for checking the type of a value.
//!
//! - `number?`: Test if value is a numeric (f64)
//! - `string?`: Test if value is a string
//! - `list?`: Test if value is a list
//! - `nil?`: Test if value is nil
//! - `symbol?`: Test if value is a symbol
//! - `bool?`: Test if value is a boolean (#t or #f)
//! - `map?`: Test if value is a map (hashmap)
//! - `keyword?`: Test if value is a keyword (:name)
//!
//! All return boolean (#t or #f)

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;

#[builtin(name = "number?", category = "Type predicates", related(string?, symbol?, list?))]
/// Tests if val is a number (integer or float).
///
/// # Examples
///
/// ```lisp
/// (number? 42) => #t
/// (number? 3.14) => #t
/// (number? "42") => #f
/// ```
///
/// # See Also
///
/// string?, symbol?, list?
pub fn builtin_number_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Number(_))))
}

#[builtin(name = "string?", category = "Type predicates", related(number?, symbol?))]
/// Tests if val is a string.
///
/// # Examples
///
/// ```lisp
/// (string? "hello") => #t
/// (string? 42) => #f
/// (string? 'hello) => #f
/// ```
///
/// # See Also
///
/// number?, symbol?
pub fn builtin_string_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::String(_))))
}

#[builtin(name = "list?", category = "Type predicates", related(number?, string?, nil?))]
/// Tests if val is a list (including nil).
///
/// # Examples
///
/// ```lisp
/// (list? '(1 2 3)) => #t
/// (list? nil) => #t
/// (list? 42) => #f
/// ```
///
/// # See Also
///
/// number?, string?, nil?
pub fn builtin_list_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::List(_))))
}

#[builtin(name = "nil?", category = "Type predicates", related(empty?, list?))]
/// Tests if val is nil (empty list).
///
/// # Examples
///
/// ```lisp
/// (nil? nil) => #t
/// (nil? '()) => #t
/// (nil? 0) => #f
/// ```
///
/// # See Also
///
/// empty?, list?
pub fn builtin_nil_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}

#[builtin(name = "symbol?", category = "Type predicates", related(string?, number?))]
/// Tests if val is a symbol (e.g., from 'hello or var names).
///
/// # Examples
///
/// ```lisp
/// (symbol? 'hello) => #t
/// (symbol? "hello") => #f
/// ```
///
/// # See Also
///
/// string?, number?
pub fn builtin_symbol_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Symbol(_))))
}

#[builtin(name = "bool?", category = "Type predicates", related(number?, string?))]
/// Tests if val is a boolean (#t or #f).
///
/// # Examples
///
/// ```lisp
/// (bool? #t) => #t
/// (bool? #f) => #t
/// (bool? 1) => #f
/// ```
///
/// # See Also
///
/// number?, string?
pub fn builtin_bool_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
}

#[builtin(name = "map?", category = "Type predicates", related(list?, keyword?))]
/// Tests if val is a map (hashmap).
///
/// # Examples
///
/// ```lisp
/// (map? {:a 1 :b 2}) => #t
/// (map? {}) => #t
/// (map? '(1 2 3)) => #f
/// (map? "string") => #f
/// ```
///
/// # See Also
///
/// list?, keyword?
pub fn builtin_map_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Map(_))))
}

#[builtin(name = "keyword?", category = "Type predicates", related(symbol?, map?))]
/// Tests if val is a keyword (:name).
///
/// # Examples
///
/// ```lisp
/// (keyword? :a) => #t
/// (keyword? :foo) => #t
/// (keyword? 'a) => #f
/// (keyword? "a") => #f
/// ```
///
/// # See Also
///
/// symbol?, map?
pub fn builtin_keyword_p(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    Ok(Value::Bool(matches!(args[0], Value::Keyword(_))))
}

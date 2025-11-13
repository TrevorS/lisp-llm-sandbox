//! Map operations: map-get, map-set, map-has?, map-keys, map-values, map-entries, map-merge, map-remove, map-empty?, map-size
//!
//! Functions for working with key-value maps

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;
use std::collections::HashMap;

#[builtin(name = "map-new", category = "Maps", related(map-get, map-set))]
/// Creates a new empty map.
///
/// # Examples
///
/// ```lisp
/// (map-new) => {}
/// ```
///
/// # See Also
///
/// map-set, map-get
pub fn map_new(args: &[Value]) -> Result<Value, EvalError> {
    if !args.is_empty() {
        return Err(EvalError::ArityMismatch);
    }
    Ok(Value::Map(HashMap::new()))
}

#[builtin(name = "map-get", category = "Maps", related(map-set, map-has?))]
/// Get value from map by keyword key. Returns nil if key not found.
///
/// # Examples
///
/// ```lisp
/// (map-get {:name "Alice" :age 30} :name) => "Alice"
/// (map-get {:x 1} :y) => nil
/// ```
///
/// # See Also
///
/// map-set, map-has?
pub fn map_get(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(EvalError::ArityMismatch);
    }

    let map = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    let key = match &args[1] {
        Value::Keyword(k) => k,
        _ => return Err(EvalError::TypeError),
    };

    let default = if args.len() == 3 {
        &args[2]
    } else {
        &Value::Nil
    };

    Ok(map.get(key).cloned().unwrap_or_else(|| default.clone()))
}

#[builtin(name = "map-set", category = "Maps", related(map-get, map-remove))]
/// Returns a new map with the key set to value (immutable operation).
///
/// # Examples
///
/// ```lisp
/// (map-set {:x 1} :y 2) => {:x 1 :y 2}
/// (map-set {} :name "Bob") => {:name "Bob"}
/// ```
///
/// # See Also
///
/// map-get, map-remove
pub fn map_set(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 3 {
        return Err(EvalError::ArityMismatch);
    }

    let map = match &args[0] {
        Value::Map(m) => m.clone(),
        _ => return Err(EvalError::TypeError),
    };

    let key = match &args[1] {
        Value::Keyword(k) => k.clone(),
        _ => return Err(EvalError::TypeError),
    };

    let value = args[2].clone();

    let mut new_map = map;
    new_map.insert(key, value);
    Ok(Value::Map(new_map))
}

#[builtin(name = "map-has?", category = "Maps", related(map-get, map-keys))]
/// Check if map contains a key.
///
/// # Examples
///
/// ```lisp
/// (map-has? {:x 1 :y 2} :x) => #t
/// (map-has? {:x 1} :z) => #f
/// ```
///
/// # See Also
///
/// map-get, map-keys
pub fn map_has_q(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let map = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    let key = match &args[1] {
        Value::Keyword(k) => k,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Bool(map.contains_key(key)))
}

#[builtin(name = "map-keys", category = "Maps", related(map-values, map-entries))]
/// Get list of all keys in map as keywords.
///
/// # Examples
///
/// ```lisp
/// (map-keys {:x 1 :y 2}) => (:x :y)
/// (map-keys {}) => ()
/// ```
///
/// # See Also
///
/// map-values, map-entries
pub fn map_keys(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let map = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    let mut keys: Vec<_> = map.keys().map(|k| Value::Keyword(k.clone())).collect();
    keys.sort_by(|a, b| match (a, b) {
        (Value::Keyword(k1), Value::Keyword(k2)) => k1.cmp(k2),
        _ => std::cmp::Ordering::Equal,
    });

    Ok(Value::List(keys))
}

#[builtin(name = "map-values", category = "Maps", related(map-keys, map-entries))]
/// Get list of all values in map.
///
/// # Examples
///
/// ```lisp
/// (map-values {:x 1 :y 2}) => (1 2)
/// (map-values {}) => ()
/// ```
///
/// # See Also
///
/// map-keys, map-entries
pub fn map_values(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let map = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    // Sort by keys for consistent ordering
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by_key(|(k, _)| *k);

    let values: Vec<_> = entries.into_iter().map(|(_, v)| v.clone()).collect();

    Ok(Value::List(values))
}

#[builtin(name = "map-entries", category = "Maps", related(map-keys, map-values))]
/// Get list of [key value] pairs from map.
///
/// # Examples
///
/// ```lisp
/// (map-entries {:x 1 :y 2}) => ((:x 1) (:y 2))
/// ```
///
/// # See Also
///
/// map-keys, map-values
pub fn map_entries(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let map = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    let mut entries: Vec<_> = map
        .iter()
        .map(|(k, v)| {
            Value::List(vec![Value::Keyword(k.clone()), v.clone()])
        })
        .collect();

    entries.sort_by(|a, b| match (a, b) {
        (Value::List(l1), Value::List(l2)) => match (&l1[0], &l2[0]) {
            (Value::Keyword(k1), Value::Keyword(k2)) => k1.cmp(k2),
            _ => std::cmp::Ordering::Equal,
        },
        _ => std::cmp::Ordering::Equal,
    });

    Ok(Value::List(entries))
}

#[builtin(name = "map-merge", category = "Maps", related(map-set))]
/// Merge two maps, with second map's values taking precedence.
///
/// # Examples
///
/// ```lisp
/// (map-merge {:x 1} {:y 2}) => {:x 1 :y 2}
/// (map-merge {:x 1} {:x 2}) => {:x 2}
/// ```
///
/// # See Also
///
/// map-set
pub fn map_merge(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let map1 = match &args[0] {
        Value::Map(m) => m.clone(),
        _ => return Err(EvalError::TypeError),
    };

    let map2 = match &args[1] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    let mut result = map1;
    for (k, v) in map2 {
        result.insert(k.clone(), v.clone());
    }

    Ok(Value::Map(result))
}

#[builtin(name = "map-remove", category = "Maps", related(map-set, map-has?))]
/// Returns a new map with the key removed.
///
/// # Examples
///
/// ```lisp
/// (map-remove {:x 1 :y 2} :x) => {:y 2}
/// ```
///
/// # See Also
///
/// map-set, map-has?
pub fn map_remove(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let map = match &args[0] {
        Value::Map(m) => m.clone(),
        _ => return Err(EvalError::TypeError),
    };

    let key = match &args[1] {
        Value::Keyword(k) => k,
        _ => return Err(EvalError::TypeError),
    };

    let mut new_map = map;
    new_map.remove(key);
    Ok(Value::Map(new_map))
}

#[builtin(name = "map-empty?", category = "Maps", related(map-size))]
/// Check if map is empty.
///
/// # Examples
///
/// ```lisp
/// (map-empty? {}) => #t
/// (map-empty? {:x 1}) => #f
/// ```
///
/// # See Also
///
/// map-size
pub fn map_empty_q(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let map = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Bool(map.is_empty()))
}

#[builtin(name = "map-size", category = "Maps", related(map-empty?))]
/// Get the number of key-value pairs in map.
///
/// # Examples
///
/// ```lisp
/// (map-size {:x 1 :y 2}) => 2
/// (map-size {}) => 0
/// ```
///
/// # See Also
///
/// map-empty?
pub fn map_size(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let map = match &args[0] {
        Value::Map(m) => m,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Number(map.len() as f64))
}

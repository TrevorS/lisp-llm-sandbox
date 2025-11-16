//! JSON encoding and decoding module
//!
//! Provides functions for converting between Lisp values and JSON strings.
//!
//! Type mapping:
//! - Lisp Map ↔ JSON object
//! - Lisp List ↔ JSON array
//! - Lisp Number ↔ JSON number
//! - Lisp String ↔ JSON string
//! - Lisp Bool ↔ JSON boolean
//! - Lisp Nil ↔ JSON null
//! - Lisp Keyword → JSON string (strip the :)

use crate::env::Environment;
use crate::error::{EvalError, ARITY_ONE};
use crate::help::HelpEntry;
use crate::value::Value;
use serde_json;
use std::collections::HashMap;
use std::rc::Rc;

/// Convert Lisp Value to serde_json::Value
fn value_to_json(value: &Value) -> Result<serde_json::Value, EvalError> {
    match value {
        Value::Number(n) => {
            if let Some(num) = serde_json::Number::from_f64(*n) {
                Ok(serde_json::Value::Number(num))
            } else {
                Err(EvalError::runtime_error(
                    "json:encode",
                    format!("cannot convert number {} to JSON", n),
                ))
            }
        }
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Keyword(k) => Ok(serde_json::Value::String(k.clone())),
        Value::Nil => Ok(serde_json::Value::Null),
        Value::List(items) => {
            let json_items: Result<Vec<_>, _> = items.iter().map(value_to_json).collect();
            Ok(serde_json::Value::Array(json_items?))
        }
        Value::Map(map) => {
            let mut json_map = serde_json::Map::new();
            for (key, val) in map {
                json_map.insert(key.clone(), value_to_json(val)?);
            }
            Ok(serde_json::Value::Object(json_map))
        }
        _ => Err(EvalError::runtime_error(
            "json:encode",
            format!("cannot convert {} to JSON", value),
        )),
    }
}

/// Convert serde_json::Value to Lisp Value
fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Value::Number(f)
            } else {
                Value::Nil // Shouldn't happen
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => Value::List(arr.iter().map(json_to_value).collect()),
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (key, val) in obj {
                map.insert(key.clone(), json_to_value(val));
            }
            Value::Map(map)
        }
    }
}

/// json:encode - Encode Lisp value to JSON string
fn json_encode(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_error("json:encode", ARITY_ONE, args.len()));
    }

    let json_value = value_to_json(&args[0])?;
    let json_string = serde_json::to_string(&json_value)
        .map_err(|e| EvalError::runtime_error("json:encode", e.to_string()))?;

    Ok(Value::String(json_string))
}

/// json:decode - Decode JSON string to Lisp value
fn json_decode(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_error("json:decode", ARITY_ONE, args.len()));
    }

    let json_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::type_error("json:decode", "string", &args[0], 1)),
    };

    let json_value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| EvalError::runtime_error("json:decode", e.to_string()))?;

    Ok(json_to_value(&json_value))
}

/// json:pretty - Encode Lisp value to pretty-printed JSON string
fn json_pretty(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::arity_error("json:pretty", ARITY_ONE, args.len()));
    }

    let json_value = value_to_json(&args[0])?;
    let json_string = serde_json::to_string_pretty(&json_value)
        .map_err(|e| EvalError::runtime_error("json:pretty", e.to_string()))?;

    Ok(Value::String(json_string))
}

/// Register json module functions in the environment
pub fn register(env: &Rc<Environment>) {
    // Register functions with json: namespace
    env.define("json:encode".to_string(), Value::BuiltIn(json_encode));
    env.define("json:decode".to_string(), Value::BuiltIn(json_decode));
    env.define("json:pretty".to_string(), Value::BuiltIn(json_pretty));

    // Register help entries
    crate::help::register_help(HelpEntry {
        name: "json:encode".to_string(),
        signature: "(json:encode value)".to_string(),
        description: "Encode a Lisp value to a JSON string.

**Type Mapping:**
- Map → JSON object
- List → JSON array
- Number → JSON number
- String → JSON string
- Bool → JSON boolean
- Nil → JSON null
- Keyword → JSON string (without :)

**Parameters:**
- value: Any Lisp value to encode

**Returns:** JSON string representation

**Examples:**
```lisp
(json:encode {:name \"Alice\" :age 30})
=> \"{\\\"name\\\":\\\"Alice\\\",\\\"age\\\":30}\"

(json:encode '(1 2 3))
=> \"[1,2,3]\"

(json:encode {:tags '(\"rust\" \"lisp\") :active #t})
=> \"{\\\"tags\\\":[\\\"rust\\\",\\\"lisp\\\"],\\\"active\\\":true}\"
```

**Notes:** Functions, lambdas, macros, and builtins cannot be encoded to JSON."
            .to_string(),
        examples: vec![
            "(json:encode {:name \"Alice\"}) => \"{\\\"name\\\":\\\"Alice\\\"}\"".to_string(),
            "(json:encode '(1 2 3)) => \"[1,2,3]\"".to_string(),
        ],
        related: vec!["json:decode".to_string(), "json:pretty".to_string()],
        category: "JSON".to_string(),
    });

    crate::help::register_help(HelpEntry {
        name: "json:decode".to_string(),
        signature: "(json:decode json-string)".to_string(),
        description: "Decode a JSON string to a Lisp value.

**Type Mapping:**
- JSON object → Map
- JSON array → List
- JSON number → Number
- JSON string → String
- JSON boolean → Bool
- JSON null → Nil

**Parameters:**
- json-string: Valid JSON string

**Returns:** Lisp value

**Examples:**
```lisp
(json:decode \"{\\\"name\\\":\\\"Bob\\\"}\")
=> {:name \"Bob\"}

(json:decode \"[1,2,3]\")
=> (1 2 3)

(json:decode \"null\")
=> nil
```

**Error Conditions:**
- Invalid JSON syntax will produce an error"
            .to_string(),
        examples: vec![
            "(json:decode \"{\\\"x\\\":1}\") => {:x 1}".to_string(),
            "(json:decode \"[1,2,3]\") => (1 2 3)".to_string(),
        ],
        related: vec!["json:encode".to_string()],
        category: "JSON".to_string(),
    });

    crate::help::register_help(HelpEntry {
        name: "json:pretty".to_string(),
        signature: "(json:pretty value)".to_string(),
        description: "Encode a Lisp value to a pretty-printed JSON string.

Same as json:encode but with indentation and newlines for readability.

**Parameters:**
- value: Any Lisp value to encode

**Returns:** Pretty-printed JSON string

**Examples:**
```lisp
(json:pretty {:name \"Alice\" :hobbies '(\"coding\" \"reading\")})
=> \"{
  \\\"name\\\": \\\"Alice\\\",
  \\\"hobbies\\\": [
    \\\"coding\\\",
    \\\"reading\\\"
  ]
}\"
```"
        .to_string(),
        examples: vec!["(json:pretty {:x 1 :y 2}) => pretty JSON".to_string()],
        related: vec!["json:encode".to_string()],
        category: "JSON".to_string(),
    });
}

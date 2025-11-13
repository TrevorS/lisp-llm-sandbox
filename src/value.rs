// ABOUTME: Value types representing Lisp data structures and expressions

use crate::env::Environment;
use crate::error::EvalError;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Symbol(String),
    Keyword(String), // For :key syntax - keywords are self-evaluating
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>), // Key-value maps
    Lambda {
        params: Vec<String>,
        body: Box<Value>,
        env: Rc<Environment>,
        docstring: Option<String>,
    },
    Macro {
        params: Vec<String>,
        body: Box<Value>,
    },
    BuiltIn(fn(&[Value]) -> Result<Value, EvalError>),
    Error(String), // Error values that can be caught
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => {
                // Format numbers cleanly - if it's a whole number, display without decimal
                if n.fract() == 0.0 && n.is_finite() {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Bool(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::Keyword(k) => write!(f, ":{}", k),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            Value::Map(map) => {
                write!(f, "{{")?;
                let mut entries: Vec<_> = map.iter().collect();
                entries.sort_by_key(|(k, _)| *k); // Sort for consistent display
                for (i, (key, value)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, ":{} {}", key, value)?;
                }
                write!(f, "}}")
            }
            Value::Lambda { .. } => write!(f, "#<lambda>"),
            Value::Macro { .. } => write!(f, "#<macro>"),
            Value::BuiltIn(_) => write!(f, "#<builtin>"),
            Value::Error(msg) => write!(f, "#<error: {}>", msg),
            Value::Nil => write!(f, "nil"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_display() {
        let whole = Value::Number(42.0);
        assert_eq!(format!("{}", whole), "42");

        let decimal = Value::Number(-2.5);
        assert_eq!(format!("{}", decimal), "-2.5");

        let zero = Value::Number(0.0);
        assert_eq!(format!("{}", zero), "0");
    }

    #[test]
    fn test_bool_display() {
        let t = Value::Bool(true);
        assert_eq!(format!("{}", t), "#t");

        let f = Value::Bool(false);
        assert_eq!(format!("{}", f), "#f");
    }

    #[test]
    fn test_list_display_with_nested_lists() {
        let simple = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        assert_eq!(format!("{}", simple), "(1 2 3)");

        let nested = Value::List(vec![
            Value::Number(1.0),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
            Value::Number(4.0),
        ]);
        assert_eq!(format!("{}", nested), "(1 (2 3) 4)");

        let empty = Value::List(vec![]);
        assert_eq!(format!("{}", empty), "()");
    }

    #[test]
    fn test_nil_display() {
        let nil = Value::Nil;
        assert_eq!(format!("{}", nil), "nil");
    }

    #[test]
    fn test_symbol_and_string_display() {
        let symbol = Value::Symbol("foo".to_string());
        assert_eq!(format!("{}", symbol), "foo");

        let string = Value::String("hello".to_string());
        assert_eq!(format!("{}", string), "\"hello\"");
    }
}

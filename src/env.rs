// ABOUTME: Environment module for managing variable bindings and scopes

use crate::error::EvalError;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Environment {
    bindings: RefCell<HashMap<String, Value>>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    /// Creates a new global environment with no parent
    pub fn new() -> Rc<Self> {
        Rc::new(Environment {
            bindings: RefCell::new(HashMap::new()),
            parent: None,
        })
    }

    /// Creates a new child environment with a parent
    #[allow(dead_code)]
    pub fn with_parent(parent: Rc<Environment>) -> Rc<Self> {
        Rc::new(Environment {
            bindings: RefCell::new(HashMap::new()),
            parent: Some(parent),
        })
    }

    /// Defines a binding in THIS scope (doesn't walk parent chain)
    pub fn define(&self, name: String, value: Value) {
        self.bindings.borrow_mut().insert(name, value);
    }

    /// Looks up a symbol in THIS scope and parent scopes recursively
    pub fn get(&self, name: &str) -> Option<Value> {
        // First check this scope
        if let Some(value) = self.bindings.borrow().get(name) {
            return Some(value.clone());
        }

        // Then check parent scope
        if let Some(ref parent) = self.parent {
            return parent.get(name);
        }

        None
    }

    /// Updates an existing binding (for later use with set!)
    #[allow(dead_code)]
    pub fn set(&self, name: &str, value: Value) -> Result<(), EvalError> {
        // Check if it exists in this scope
        if self.bindings.borrow().contains_key(name) {
            self.bindings.borrow_mut().insert(name.to_string(), value);
            return Ok(());
        }

        // Check parent scope
        if let Some(ref parent) = self.parent {
            return parent.set(name, value);
        }

        Err(EvalError::UndefinedSymbol(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_get() {
        let env = Environment::new();
        env.define("x".to_string(), Value::Number(42.0));

        match env.get("x") {
            Some(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_undefined_symbol() {
        let env = Environment::new();
        assert!(env.get("undefined").is_none());
    }

    #[test]
    fn test_shadowing() {
        let parent = Environment::new();
        parent.define("x".to_string(), Value::Number(42.0));

        let child = Environment::with_parent(parent);
        child.define("x".to_string(), Value::Number(100.0));

        // Child should see its own value
        match child.get("x") {
            Some(Value::Number(n)) => assert_eq!(n, 100.0),
            _ => panic!("Expected Number(100.0)"),
        }
    }

    #[test]
    fn test_parent_lookup() {
        let parent = Environment::new();
        parent.define("x".to_string(), Value::Number(42.0));

        let child = Environment::with_parent(parent);

        // Child should see parent's value
        match child.get("x") {
            Some(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_multiple_levels() {
        // Grandparent
        let grandparent = Environment::new();
        grandparent.define("a".to_string(), Value::Number(1.0));

        // Parent
        let parent = Environment::with_parent(grandparent);
        parent.define("b".to_string(), Value::Number(2.0));

        // Child
        let child = Environment::with_parent(parent);
        child.define("c".to_string(), Value::Number(3.0));

        // Child can see all three levels
        match child.get("a") {
            Some(Value::Number(n)) => assert_eq!(n, 1.0),
            _ => panic!("Expected Number(1.0)"),
        }

        match child.get("b") {
            Some(Value::Number(n)) => assert_eq!(n, 2.0),
            _ => panic!("Expected Number(2.0)"),
        }

        match child.get("c") {
            Some(Value::Number(n)) => assert_eq!(n, 3.0),
            _ => panic!("Expected Number(3.0)"),
        }
    }
}

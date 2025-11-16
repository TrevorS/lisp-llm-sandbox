// ABOUTME: Environment module for managing variable bindings and scopes
//
// V2 Architecture: Thread-safe immutable environments using Arc
// - Environments are immutable (HashMap not RefCell<HashMap>)
// - Use Arc<Environment> instead of Rc<Environment> for Send + Sync
// - extend() method returns new environment (functional/immutable pattern)
// - Thread-local GLOBAL_ENV in eval.rs manages global definitions
// - This enables safe concurrent execution with spawn

use crate::error::EvalError;
use crate::value::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Environment {
    bindings: HashMap<String, Value>,  // Immutable!
    parent: Option<Arc<Environment>>,
}

impl Environment {
    /// Creates a new global environment with no parent
    pub fn new() -> Arc<Self> {
        Arc::new(Environment {
            bindings: HashMap::new(),
            parent: None,
        })
    }

    /// Creates a new child environment with a parent
    #[allow(dead_code)]
    pub fn with_parent(parent: Arc<Environment>) -> Arc<Self> {
        Arc::new(Environment {
            bindings: HashMap::new(),
            parent: Some(parent),
        })
    }

    /// Extends environment with a new binding, returning new environment (functional)
    /// This is the primary way to add bindings in the immutable V2 architecture
    pub fn extend(&self, name: String, value: Value) -> Arc<Environment> {
        let mut new_bindings = self.bindings.clone();
        new_bindings.insert(name, value);
        Arc::new(Environment {
            bindings: new_bindings,
            parent: self.parent.clone(),
        })
    }

    /// Looks up a symbol in THIS scope and parent scopes recursively
    pub fn get(&self, name: &str) -> Option<Value> {
        // First check this scope
        if let Some(value) = self.bindings.get(name) {
            return Some(value.clone());
        }

        // Then check parent scope
        if let Some(ref parent) = self.parent {
            return parent.get(name);
        }

        None
    }

    /// Updates an existing binding (for later use with set!)
    /// Note: This needs to return a new environment in V2
    #[allow(dead_code)]
    pub fn set(&self, name: &str, value: Value) -> Result<Arc<Environment>, EvalError> {
        // Check if it exists in this scope
        if self.bindings.contains_key(name) {
            return Ok(self.extend(name.to_string(), value));
        }

        // Check parent scope
        if let Some(ref parent) = self.parent {
            let new_parent = parent.set(name, value)?;
            return Ok(Arc::new(Environment {
                bindings: self.bindings.clone(),
                parent: Some(new_parent),
            }));
        }

        Err(EvalError::UndefinedSymbol(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extend_and_get() {
        let env = Environment::new();
        let env = env.extend("x".to_string(), Value::Number(42.0));

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
        let parent = parent.extend("x".to_string(), Value::Number(42.0));

        let child = Environment::with_parent(parent);
        let child = child.extend("x".to_string(), Value::Number(100.0));

        // Child should see its own value
        match child.get("x") {
            Some(Value::Number(n)) => assert_eq!(n, 100.0),
            _ => panic!("Expected Number(100.0)"),
        }
    }

    #[test]
    fn test_parent_lookup() {
        let parent = Environment::new();
        let parent = parent.extend("x".to_string(), Value::Number(42.0));

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
        let grandparent = grandparent.extend("a".to_string(), Value::Number(1.0));

        // Parent
        let parent = Environment::with_parent(grandparent);
        let parent = parent.extend("b".to_string(), Value::Number(2.0));

        // Child
        let child = Environment::with_parent(parent);
        let child = child.extend("c".to_string(), Value::Number(3.0));

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

    #[test]
    fn test_immutability() {
        // Verify that extend doesn't mutate the original
        let env1 = Environment::new();
        let env2 = env1.extend("x".to_string(), Value::Number(42.0));

        // env1 should not have x
        assert!(env1.get("x").is_none());

        // env2 should have x
        match env2.get("x") {
            Some(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }
}

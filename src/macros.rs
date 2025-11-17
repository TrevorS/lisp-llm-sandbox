// ABOUTME: Macro registry for storing and retrieving macro definitions

use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MacroRegistry {
    macros: HashMap<String, (Vec<String>, Option<String>, Value)>,
}

impl Default for MacroRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MacroRegistry {
    pub fn new() -> Self {
        MacroRegistry {
            macros: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, params: Vec<String>, rest_param: Option<String>, body: Value) {
        self.macros.insert(name, (params, rest_param, body));
    }

    pub fn get(&self, name: &str) -> Option<(Vec<String>, Option<String>, Value)> {
        self.macros.get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_registry_define_and_get() {
        let mut registry = MacroRegistry::new();

        let params = vec!["x".to_string()];
        let body = Value::Symbol("x".to_string());

        registry.define("test-macro".to_string(), params.clone(), None, body.clone());

        let result = registry.get("test-macro");
        assert!(result.is_some());

        let (retrieved_params, retrieved_rest, _retrieved_body) = result.unwrap();
        assert_eq!(retrieved_params, params);
        assert_eq!(retrieved_rest, None);
    }

    #[test]
    fn test_macro_registry_get_undefined() {
        let registry = MacroRegistry::new();
        assert!(registry.get("undefined").is_none());
    }
}

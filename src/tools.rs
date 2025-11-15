// ABOUTME: Tool trait system for extending Lisp functionality with Rust code

use crate::error::EvalError;
use crate::value::Value;

/// A tool is a Rust function that can be called from Lisp
/// This provides an extensible way to add native capabilities to the interpreter
#[allow(dead_code)]
pub trait Tool: Send + Sync {
    /// Execute the tool with the given arguments
    fn call(&self, args: &[Value]) -> Result<Value, EvalError>;

    /// Get the name of the tool
    fn name(&self) -> &str;

    /// Get the arity (number of expected arguments)
    /// None = variadic (any number of args)
    /// Some(n) = exactly n arguments required
    fn arity(&self) -> Option<usize>;

    /// Get help text describing what this tool does
    fn help(&self) -> &str;
}

/// Simple tool wrapper for function pointers
/// This makes it easy to wrap existing Rust functions as tools
#[allow(dead_code)]
pub struct SimpleTool {
    name: String,
    arity: Option<usize>,
    help: String,
    func: fn(&[Value]) -> Result<Value, EvalError>,
}

impl SimpleTool {
    /// Create a new simple tool from a function pointer
    #[allow(dead_code)]
    pub fn new(
        name: &str,
        arity: Option<usize>,
        help: &str,
        func: fn(&[Value]) -> Result<Value, EvalError>,
    ) -> Self {
        SimpleTool {
            name: name.to_string(),
            arity,
            help: help.to_string(),
            func,
        }
    }
}

impl Tool for SimpleTool {
    fn call(&self, args: &[Value]) -> Result<Value, EvalError> {
        // Validate arity if specified
        if let Some(expected_arity) = self.arity {
            if args.len() != expected_arity {
                return Err(EvalError::arity_error(&self.name, expected_arity.to_string(), args.len()));
            }
        }
        (self.func)(args)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn arity(&self) -> Option<usize> {
        self.arity
    }

    fn help(&self) -> &str {
        &self.help
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_add(args: &[Value]) -> Result<Value, EvalError> {
        let mut sum = 0.0;
        for (i, arg) in args.iter().enumerate() {
            match arg {
                Value::Number(n) => sum += n,
                _ => return Err(EvalError::type_error("add", "number", arg, i + 1)),
            }
        }
        Ok(Value::Number(sum))
    }

    #[test]
    fn test_simple_tool_creation() {
        let tool = SimpleTool::new("add", None, "Add numbers together", test_add);

        assert_eq!(tool.name(), "add");
        assert_eq!(tool.arity(), None);
        assert_eq!(tool.help(), "Add numbers together");
    }

    #[test]
    fn test_simple_tool_call() {
        let tool = SimpleTool::new("add", None, "Add numbers together", test_add);

        let args = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let result = tool.call(&args).unwrap();

        match result {
            Value::Number(n) => assert_eq!(n, 6.0),
            _ => panic!("Expected Number(6.0)"),
        }
    }

    #[test]
    fn test_arity_check() {
        fn fixed_arity_fn(args: &[Value]) -> Result<Value, EvalError> {
            Ok(args[0].clone())
        }

        let tool = SimpleTool::new("identity", Some(1), "Return the argument", fixed_arity_fn);

        // Correct arity should work
        let result = tool.call(&[Value::Number(42.0)]);
        assert!(result.is_ok());

        // Wrong arity should fail
        let result = tool.call(&[Value::Number(1.0), Value::Number(2.0)]);
        assert!(matches!(result, Err(EvalError::ArityError { .. })));
    }
}

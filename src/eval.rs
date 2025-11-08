// ABOUTME: Evaluator module for executing parsed Lisp expressions

use crate::env::Environment;
use crate::error::EvalError;
use crate::macros::MacroRegistry;
use crate::value::Value;
use std::rc::Rc;

/// Main evaluation function with tail call optimization
#[allow(dead_code)]
pub fn eval(expr: Value, env: Rc<Environment>) -> Result<Value, EvalError> {
    eval_with_macros(expr, env, &mut MacroRegistry::new())
}

/// Evaluation function with macro registry support
pub fn eval_with_macros(
    mut expr: Value,
    env: Rc<Environment>,
    macro_reg: &mut MacroRegistry,
) -> Result<Value, EvalError> {
    let mut current_env = env;
    loop {
        // First expand macros
        expr = expand_macros(expr.clone(), macro_reg, current_env.clone())?;

        match &expr {
            // Self-evaluating values
            Value::Number(_) | Value::Bool(_) | Value::String(_) | Value::Nil => {
                return Ok(expr.clone());
            }

            // Symbol lookup
            Value::Symbol(name) => {
                // Special case: 'nil' as a symbol evaluates to Nil value
                if name == "nil" {
                    return Ok(Value::Nil);
                }
                return current_env
                    .get(name)
                    .ok_or_else(|| EvalError::UndefinedSymbol(name.clone()));
            }

            // Empty list evaluates to nil
            Value::List(items) if items.is_empty() => return Ok(Value::Nil),

            // Non-empty list: special forms or function application
            Value::List(items) => {
                match &items[0] {
                    Value::Symbol(s) if s == "define" => {
                        return eval_define(&items[1..], current_env, macro_reg);
                    }
                    Value::Symbol(s) if s == "lambda" => {
                        return eval_lambda(&items[1..], current_env);
                    }
                    Value::Symbol(s) if s == "quote" => {
                        if items.len() != 2 {
                            return Err(EvalError::Custom("quote: expected 1 argument".into()));
                        }
                        return Ok(items[1].clone());
                    }
                    Value::Symbol(s) if s == "quasiquote" => {
                        if items.len() != 2 {
                            return Err(EvalError::Custom(
                                "quasiquote: expected 1 argument".into(),
                            ));
                        }
                        return eval_quasiquote(items[1].clone(), 1, current_env, macro_reg);
                    }
                    Value::Symbol(s) if s == "defmacro" => {
                        return eval_defmacro(&items[1..], current_env, macro_reg);
                    }
                    Value::Symbol(s) if s == "if" => {
                        // Tail-optimized if: evaluate condition, then loop on branch
                        if items.len() < 3 || items.len() > 4 {
                            return Err(EvalError::Custom("if: expected 2 or 3 arguments".into()));
                        }

                        let condition =
                            eval_with_macros(items[1].clone(), current_env.clone(), macro_reg)?;
                        let is_true = match condition {
                            Value::Bool(b) => b,
                            Value::Nil => false,
                            _ => true, // Everything except #f and nil is truthy
                        };

                        if is_true {
                            expr = items[2].clone();
                            // Continue loop for tail call
                        } else if items.len() > 3 {
                            expr = items[3].clone();
                            // Continue loop for tail call
                        } else {
                            return Ok(Value::Nil);
                        }
                    }
                    Value::Symbol(s) if s == "begin" => {
                        // Tail-optimized begin: evaluate all but last, then loop on last
                        if items.len() == 1 {
                            return Ok(Value::Nil);
                        }

                        // Evaluate all items except the last
                        for item in items.iter().skip(1).take(items.len() - 2) {
                            eval_with_macros(item.clone(), current_env.clone(), macro_reg)?;
                        }

                        expr = items[items.len() - 1].clone();
                        // Continue loop for tail call
                    }
                    Value::Symbol(s) if s == "let" => {
                        return eval_let(&items[1..], current_env, macro_reg);
                    }
                    _ => {
                        // Function application - check if it's a lambda for TCO
                        let func =
                            eval_with_macros(items[0].clone(), current_env.clone(), macro_reg)?;

                        // Evaluate arguments
                        let args: Result<Vec<_>, _> = items[1..]
                            .iter()
                            .map(|arg| {
                                eval_with_macros(arg.clone(), current_env.clone(), macro_reg)
                            })
                            .collect();
                        let args = args?;

                        match func {
                            Value::Lambda {
                                params,
                                body,
                                env: lambda_env,
                                docstring: _,
                            } => {
                                // Check arity
                                if params.len() != args.len() {
                                    return Err(EvalError::ArityMismatch);
                                }

                                // Create new environment for lambda
                                let new_env = Environment::with_parent(lambda_env);
                                for (param, arg) in params.iter().zip(args.iter()) {
                                    new_env.define(param.clone(), arg.clone());
                                }

                                // Tail call: set up for next iteration
                                expr = *body;
                                current_env = new_env;
                                // Continue loop
                            }
                            Value::BuiltIn(f) => {
                                return f(&args);
                            }
                            _ => {
                                return Err(EvalError::NotCallable);
                            }
                        }
                    }
                }
            }

            // Lambda, Macro, BuiltIn, and Error are also self-evaluating (though rarely evaluated directly)
            Value::Lambda { .. } | Value::Macro { .. } | Value::BuiltIn(_) | Value::Error(_) => {
                return Ok(expr.clone());
            }
        }
    }
}

/// Evaluate a define special form
/// Handles:
/// - (define x 42) - variable definition
/// - (define (f x) body) - function definition (syntactic sugar for lambda)
fn eval_define(
    args: &[Value],
    env: Rc<Environment>,
    macro_reg: &mut MacroRegistry,
) -> Result<Value, EvalError> {
    if args.len() < 2 {
        return Err(EvalError::Custom(
            "define requires at least 2 arguments".to_string(),
        ));
    }

    match &args[0] {
        // Variable definition: (define x 42)
        Value::Symbol(name) => {
            let value = eval_with_macros(args[1].clone(), env.clone(), macro_reg)?;
            env.define(name.clone(), value);
            Ok(Value::Symbol(name.clone()))
        }

        // Function definition: (define (f x y) body)
        Value::List(func_def) if !func_def.is_empty() => {
            // Extract function name
            let name = match &func_def[0] {
                Value::Symbol(n) => n.clone(),
                _ => {
                    return Err(EvalError::Custom(
                        "Function name must be a symbol".to_string(),
                    ));
                }
            };

            // Extract parameters
            let mut params = Vec::new();
            for param in &func_def[1..] {
                match param {
                    Value::Symbol(p) => params.push(p.clone()),
                    _ => {
                        return Err(EvalError::Custom(
                            "Function parameters must be symbols".to_string(),
                        ));
                    }
                }
            }

            // Extract docstring if present: (define (f x) "doc" body)
            let (docstring, body) = match &args[1] {
                Value::String(s) if args.len() > 2 => (Some(s.clone()), Box::new(args[2].clone())),
                _ => (None, Box::new(args[1].clone())),
            };

            // Create lambda
            let lambda = Value::Lambda {
                params,
                body,
                env: env.clone(),
                docstring,
            };

            // Define it
            env.define(name.clone(), lambda);
            Ok(Value::Symbol(name))
        }

        _ => Err(EvalError::Custom(
            "define requires a symbol or list as first argument".to_string(),
        )),
    }
}

/// Evaluate a lambda expression
/// (lambda (x y z) body) or (lambda (x y z) "docstring" body)
fn eval_lambda(args: &[Value], env: Rc<Environment>) -> Result<Value, EvalError> {
    if args.len() < 2 {
        return Err(EvalError::Custom(
            "lambda requires at least 2 arguments (params and body)".to_string(),
        ));
    }

    // Extract parameters from args[0]
    let params = match &args[0] {
        Value::List(param_list) => {
            let mut params = Vec::new();
            for param in param_list {
                match param {
                    Value::Symbol(name) => params.push(name.clone()),
                    _ => {
                        return Err(EvalError::Custom(
                            "Lambda parameters must be symbols".to_string(),
                        ));
                    }
                }
            }
            params
        }
        _ => {
            return Err(EvalError::Custom(
                "Lambda parameters must be a list".to_string(),
            ));
        }
    };

    // Extract docstring if present: (lambda (x y) "doc" body)
    let (docstring, body) = match &args[1] {
        Value::String(s) if args.len() > 2 => (Some(s.clone()), Box::new(args[2].clone())),
        _ => (None, Box::new(args[1].clone())),
    };

    Ok(Value::Lambda {
        params,
        body,
        env,
        docstring,
    })
}

/// Evaluate a let special form
/// (let ((x 1) (y 2)) body)
fn eval_let(
    args: &[Value],
    env: Rc<Environment>,
    macro_reg: &mut MacroRegistry,
) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::Custom("let: expected bindings and body".into()));
    }

    let bindings = match &args[0] {
        Value::List(items) => items,
        _ => return Err(EvalError::Custom("let: bindings must be a list".into())),
    };

    // Create new environment as child of current env
    let new_env = Environment::with_parent(env);

    // Evaluate bindings and add to new environment
    for binding in bindings {
        match binding {
            Value::List(pair) if pair.len() == 2 => {
                let name = match &pair[0] {
                    Value::Symbol(s) => s.clone(),
                    _ => return Err(EvalError::Custom("let: binding name must be symbol".into())),
                };
                let value = eval_with_macros(pair[1].clone(), new_env.clone(), macro_reg)?;
                new_env.define(name, value);
            }
            _ => {
                return Err(EvalError::Custom(
                    "let: binding must be [symbol value]".into(),
                ));
            }
        }
    }

    // Evaluate body in new environment
    let mut result = Value::Nil;
    for expr in &args[1..] {
        result = eval_with_macros(expr.clone(), new_env.clone(), macro_reg)?;
    }
    Ok(result)
}

/// Evaluate a quasiquote expression
/// Depth tracks nesting level: depth 1 means we're inside one quasiquote
fn eval_quasiquote(
    arg: Value,
    depth: usize,
    env: Rc<Environment>,
    macro_reg: &mut MacroRegistry,
) -> Result<Value, EvalError> {
    match arg {
        // Self-evaluating values
        Value::Number(_) | Value::Bool(_) | Value::String(_) | Value::Nil => Ok(arg),

        // Symbols stay as symbols
        Value::Symbol(_) => Ok(arg),

        Value::List(ref items) if !items.is_empty() => {
            match &items[0] {
                // (unquote expr) at depth 1 → evaluate expr
                Value::Symbol(s) if s == "unquote" && depth == 1 => {
                    if items.len() != 2 {
                        return Err(EvalError::Custom("unquote: expected 1 argument".into()));
                    }
                    eval_with_macros(items[1].clone(), env, macro_reg)
                }

                // (quasiquote ...) → increase depth and recurse
                Value::Symbol(s) if s == "quasiquote" => {
                    if items.len() != 2 {
                        return Err(EvalError::Custom("quasiquote: expected 1 argument".into()));
                    }
                    let inner = eval_quasiquote(items[1].clone(), depth + 1, env, macro_reg)?;
                    Ok(Value::List(vec![Value::Symbol("quasiquote".into()), inner]))
                }

                // Regular list - recurse on all items, handling unquote-splicing
                _ => {
                    let mut new_items = Vec::new();

                    for item in items {
                        match item {
                            Value::List(parts) if !parts.is_empty() => match &parts[0] {
                                Value::Symbol(s) if s == "unquote-splicing" && depth == 1 => {
                                    if parts.len() != 2 {
                                        return Err(EvalError::Custom(
                                            "unquote-splicing: expected 1 argument".into(),
                                        ));
                                    }
                                    match eval_with_macros(
                                        parts[1].clone(),
                                        env.clone(),
                                        macro_reg,
                                    )? {
                                        Value::List(splice) => {
                                            new_items.extend(splice);
                                        }
                                        _ => {
                                            return Err(EvalError::Custom(
                                                "unquote-splicing requires a list".into(),
                                            ));
                                        }
                                    }
                                }
                                _ => {
                                    let evaled = eval_quasiquote(
                                        item.clone(),
                                        depth,
                                        env.clone(),
                                        macro_reg,
                                    )?;
                                    new_items.push(evaled);
                                }
                            },
                            _ => {
                                let evaled =
                                    eval_quasiquote(item.clone(), depth, env.clone(), macro_reg)?;
                                new_items.push(evaled);
                            }
                        }
                    }

                    Ok(Value::List(new_items))
                }
            }
        }

        Value::List(_) => Ok(Value::Nil),

        _ => Ok(arg),
    }
}

/// Evaluate a defmacro special form
/// (defmacro name (params) body)
fn eval_defmacro(
    args: &[Value],
    _env: Rc<Environment>,
    macro_reg: &mut MacroRegistry,
) -> Result<Value, EvalError> {
    if args.len() < 3 {
        return Err(EvalError::Custom(
            "defmacro: expected name, params, and body".into(),
        ));
    }

    let name = match &args[0] {
        Value::Symbol(n) => n.clone(),
        _ => return Err(EvalError::Custom("defmacro: name must be a symbol".into())),
    };

    let params = match &args[1] {
        Value::List(p) => p
            .iter()
            .map(|v| match v {
                Value::Symbol(s) => Ok(s.clone()),
                _ => Err(EvalError::Custom(
                    "defmacro: parameter must be symbol".into(),
                )),
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err(EvalError::Custom("defmacro: params must be a list".into())),
    };

    // Body is the remaining args, wrapped in begin if multiple
    let body = if args.len() > 3 {
        let mut body_items = vec![Value::Symbol("begin".into())];
        body_items.extend_from_slice(&args[2..]);
        Value::List(body_items)
    } else {
        args[2].clone()
    };

    macro_reg.define(name.clone(), params, body);
    Ok(Value::Symbol(name))
}

/// Expand macros in an expression
fn expand_macros(
    expr: Value,
    macro_reg: &MacroRegistry,
    env: Rc<Environment>,
) -> Result<Value, EvalError> {
    match expr {
        Value::List(ref items) if !items.is_empty() => {
            match &items[0] {
                Value::Symbol(name) => {
                    if let Some((params, body)) = macro_reg.get(name) {
                        // Bind arguments to parameters
                        let args = &items[1..];

                        if params.len() != args.len() {
                            return Err(EvalError::ArityMismatch);
                        }

                        let macro_env = Environment::with_parent(env.clone());
                        for (param, arg) in params.iter().zip(args.iter()) {
                            // Arguments to macros are NOT evaluated yet
                            macro_env.define(param.clone(), arg.clone());
                        }

                        // Evaluate body in macro environment (this handles quasiquote expansion)
                        let mut temp_reg = MacroRegistry::new();
                        let expanded = eval_with_macros(body, macro_env, &mut temp_reg)?;

                        // Recursively expand if result is a macro call
                        expand_macros(expanded, macro_reg, env)
                    } else {
                        Ok(expr)
                    }
                }
                _ => Ok(expr),
            }
        }
        _ => Ok(expr),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_number() {
        let env = Environment::new();
        let result = eval(Value::Number(42.0), env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_eval_bool() {
        let env = Environment::new();

        let result = eval(Value::Bool(true), env.clone()).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = eval(Value::Bool(false), env).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_eval_string() {
        let env = Environment::new();
        let result = eval(Value::String("hello".to_string()), env).unwrap();
        match result {
            Value::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String(\"hello\")"),
        }
    }

    #[test]
    fn test_eval_symbol_lookup() {
        let env = Environment::new();
        env.define("x".to_string(), Value::Number(42.0));

        let result = eval(Value::Symbol("x".to_string()), env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_eval_undefined_symbol() {
        let env = Environment::new();
        let result = eval(Value::Symbol("undefined".to_string()), env);

        match result {
            Err(EvalError::UndefinedSymbol(name)) => assert_eq!(name, "undefined"),
            _ => panic!("Expected UndefinedSymbol error"),
        }
    }

    #[test]
    fn test_eval_nil() {
        let env = Environment::new();
        let result = eval(Value::Nil, env).unwrap();
        match result {
            Value::Nil => (),
            _ => panic!("Expected Nil"),
        }
    }

    #[test]
    fn test_eval_define_variable() {
        let env = Environment::new();

        // (define x 42)
        let define_expr = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::Symbol("x".to_string()),
            Value::Number(42.0),
        ]);

        let result = eval(define_expr, env.clone()).unwrap();

        // Should return the symbol name
        match result {
            Value::Symbol(s) => assert_eq!(s, "x"),
            _ => panic!("Expected Symbol(\"x\")"),
        }

        // Check that x is now defined
        match env.get("x") {
            Some(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected x to be defined as Number(42.0)"),
        }
    }

    #[test]
    fn test_eval_define_function() {
        let env = Environment::new();

        // (define (f x) x)
        let define_expr = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::List(vec![
                Value::Symbol("f".to_string()),
                Value::Symbol("x".to_string()),
            ]),
            Value::Symbol("x".to_string()),
        ]);

        let result = eval(define_expr, env.clone()).unwrap();

        // Should return the function name
        match result {
            Value::Symbol(s) => assert_eq!(s, "f"),
            _ => panic!("Expected Symbol(\"f\")"),
        }

        // Check that f is now defined as a lambda
        match env.get("f") {
            Some(Value::Lambda { params, body, .. }) => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], "x");
                match *body {
                    Value::Symbol(ref s) => assert_eq!(s, "x"),
                    _ => panic!("Expected body to be Symbol(\"x\")"),
                }
            }
            _ => panic!("Expected f to be defined as a Lambda"),
        }
    }

    #[test]
    fn test_eval_after_define() {
        let env = Environment::new();

        // (define x 42)
        let define_expr = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::Symbol("x".to_string()),
            Value::Number(42.0),
        ]);
        eval(define_expr, env.clone()).unwrap();

        // Now eval the symbol x
        let result = eval(Value::Symbol("x".to_string()), env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_shadowing_in_eval() {
        let parent = Environment::new();
        parent.define("x".to_string(), Value::Number(10.0));

        let child = Environment::with_parent(parent);

        // Define x in child scope
        let define_expr = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::Symbol("x".to_string()),
            Value::Number(20.0),
        ]);
        eval(define_expr, child.clone()).unwrap();

        // Child should see its own value
        let result = eval(Value::Symbol("x".to_string()), child).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 20.0),
            _ => panic!("Expected Number(20.0)"),
        }
    }

    // ========================================================================
    // Lambda and Function Application Tests
    // ========================================================================

    #[test]
    fn test_eval_lambda() {
        let env = Environment::new();

        // (lambda (x) x)
        let lambda_expr = Value::List(vec![
            Value::Symbol("lambda".to_string()),
            Value::List(vec![Value::Symbol("x".to_string())]),
            Value::Symbol("x".to_string()),
        ]);

        let result = eval(lambda_expr, env).unwrap();
        match result {
            Value::Lambda { params, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], "x");
            }
            _ => panic!("Expected Lambda"),
        }
    }

    #[test]
    fn test_lambda_application_identity() {
        let env = Environment::new();

        // ((lambda (x) x) 42)
        let expr = Value::List(vec![
            Value::List(vec![
                Value::Symbol("lambda".to_string()),
                Value::List(vec![Value::Symbol("x".to_string())]),
                Value::Symbol("x".to_string()),
            ]),
            Value::Number(42.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_lambda_with_multiple_params() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());

        // ((lambda (x y) (+ x y)) 10 20)
        let expr = Value::List(vec![
            Value::List(vec![
                Value::Symbol("lambda".to_string()),
                Value::List(vec![
                    Value::Symbol("x".to_string()),
                    Value::Symbol("y".to_string()),
                ]),
                Value::List(vec![
                    Value::Symbol("+".to_string()),
                    Value::Symbol("x".to_string()),
                    Value::Symbol("y".to_string()),
                ]),
            ]),
            Value::Number(10.0),
            Value::Number(20.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 30.0),
            _ => panic!("Expected Number(30.0)"),
        }
    }

    #[test]
    fn test_arity_mismatch() {
        let env = Environment::new();

        // ((lambda (x) x) 1 2) - too many args
        let expr = Value::List(vec![
            Value::List(vec![
                Value::Symbol("lambda".to_string()),
                Value::List(vec![Value::Symbol("x".to_string())]),
                Value::Symbol("x".to_string()),
            ]),
            Value::Number(1.0),
            Value::Number(2.0),
        ]);

        let result = eval(expr, env);
        assert!(matches!(result, Err(EvalError::ArityMismatch)));
    }

    #[test]
    fn test_closure_captures_environment() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());

        // (define x 10)
        let define_x = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::Symbol("x".to_string()),
            Value::Number(10.0),
        ]);
        eval(define_x, env.clone()).unwrap();

        // (define f (lambda (y) (+ x y)))
        let define_f = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::Symbol("f".to_string()),
            Value::List(vec![
                Value::Symbol("lambda".to_string()),
                Value::List(vec![Value::Symbol("y".to_string())]),
                Value::List(vec![
                    Value::Symbol("+".to_string()),
                    Value::Symbol("x".to_string()),
                    Value::Symbol("y".to_string()),
                ]),
            ]),
        ]);
        eval(define_f, env.clone()).unwrap();

        // (f 5) should be 15
        let call_f = Value::List(vec![Value::Symbol("f".to_string()), Value::Number(5.0)]);
        let result = eval(call_f, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 15.0),
            _ => panic!("Expected Number(15.0)"),
        }
    }

    #[test]
    fn test_nested_function_calls() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());

        // (* (+ 1 2) 3) should be 9
        let expr = Value::List(vec![
            Value::Symbol("*".to_string()),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Number(1.0),
                Value::Number(2.0),
            ]),
            Value::Number(3.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 9.0),
            _ => panic!("Expected Number(9.0)"),
        }
    }

    #[test]
    fn test_higher_order_function() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());

        // (define (make-adder n) (lambda (x) (+ x n)))
        let define_maker = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::List(vec![
                Value::Symbol("make-adder".to_string()),
                Value::Symbol("n".to_string()),
            ]),
            Value::List(vec![
                Value::Symbol("lambda".to_string()),
                Value::List(vec![Value::Symbol("x".to_string())]),
                Value::List(vec![
                    Value::Symbol("+".to_string()),
                    Value::Symbol("x".to_string()),
                    Value::Symbol("n".to_string()),
                ]),
            ]),
        ]);
        eval(define_maker, env.clone()).unwrap();

        // (define add5 (make-adder 5))
        let define_add5 = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::Symbol("add5".to_string()),
            Value::List(vec![
                Value::Symbol("make-adder".to_string()),
                Value::Number(5.0),
            ]),
        ]);
        eval(define_add5, env.clone()).unwrap();

        // (add5 10) should be 15
        let call_add5 = Value::List(vec![Value::Symbol("add5".to_string()), Value::Number(10.0)]);
        let result = eval(call_add5, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 15.0),
            _ => panic!("Expected Number(15.0)"),
        }
    }

    #[test]
    fn test_builtin_function_call() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());

        // (+ 1 2 3)
        let expr = Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 6.0),
            _ => panic!("Expected Number(6.0)"),
        }
    }

    #[test]
    fn test_not_callable_error() {
        let env = Environment::new();

        // (42 1 2) - trying to call a number
        let expr = Value::List(vec![
            Value::Number(42.0),
            Value::Number(1.0),
            Value::Number(2.0),
        ]);

        let result = eval(expr, env);
        assert!(matches!(result, Err(EvalError::NotCallable)));
    }

    // ========================================================================
    // Control Flow Tests - if special form
    // ========================================================================

    #[test]
    fn test_if_true_condition() {
        let env = Environment::new();

        // (if #t 42 0)
        let expr = Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::Bool(true),
            Value::Number(42.0),
            Value::Number(0.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_if_false_condition() {
        let env = Environment::new();

        // (if #f 42 0)
        let expr = Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::Bool(false),
            Value::Number(42.0),
            Value::Number(0.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 0.0),
            _ => panic!("Expected Number(0.0)"),
        }
    }

    #[test]
    fn test_if_without_else() {
        let env = Environment::new();

        // (if #f 42) - should return nil
        let expr = Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::Bool(false),
            Value::Number(42.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Nil => (),
            _ => panic!("Expected Nil"),
        }
    }

    #[test]
    fn test_if_nil_is_falsy() {
        let env = Environment::new();

        // (if nil 42 0)
        let expr = Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::Nil,
            Value::Number(42.0),
            Value::Number(0.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 0.0),
            _ => panic!("Expected Number(0.0)"),
        }
    }

    #[test]
    fn test_if_numbers_are_truthy() {
        let env = Environment::new();

        // (if 0 42 0) - 0 is truthy in Lisp
        let expr = Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::Number(0.0),
            Value::Number(42.0),
            Value::Number(0.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_if_with_expression_condition() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());

        // (if (< 1 2) 42 0)
        let expr = Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::List(vec![
                Value::Symbol("<".to_string()),
                Value::Number(1.0),
                Value::Number(2.0),
            ]),
            Value::Number(42.0),
            Value::Number(0.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    // ========================================================================
    // Control Flow Tests - begin special form
    // ========================================================================

    #[test]
    fn test_begin_empty() {
        let env = Environment::new();

        // (begin)
        let expr = Value::List(vec![Value::Symbol("begin".to_string())]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Nil => (),
            _ => panic!("Expected Nil"),
        }
    }

    #[test]
    fn test_begin_single_expression() {
        let env = Environment::new();

        // (begin 42)
        let expr = Value::List(vec![
            Value::Symbol("begin".to_string()),
            Value::Number(42.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_begin_multiple_expressions() {
        let env = Environment::new();

        // (begin 1 2 3)
        let expr = Value::List(vec![
            Value::Symbol("begin".to_string()),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 3.0),
            _ => panic!("Expected Number(3.0)"),
        }
    }

    #[test]
    fn test_begin_with_side_effects() {
        let env = Environment::new();

        // (begin (define x 10) (define y 20) (+ x y))
        // This is just to verify all expressions execute
        let expr = Value::List(vec![
            Value::Symbol("begin".to_string()),
            Value::List(vec![
                Value::Symbol("define".to_string()),
                Value::Symbol("x".to_string()),
                Value::Number(10.0),
            ]),
            Value::List(vec![
                Value::Symbol("define".to_string()),
                Value::Symbol("y".to_string()),
                Value::Number(20.0),
            ]),
            Value::Symbol("y".to_string()),
        ]);

        let result = eval(expr, env.clone()).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 20.0),
            _ => panic!("Expected Number(20.0)"),
        }

        // Verify x was also defined
        match env.get("x") {
            Some(Value::Number(n)) => assert_eq!(n, 10.0),
            _ => panic!("Expected x to be defined as 10.0"),
        }
    }

    // ========================================================================
    // Control Flow Tests - let special form
    // ========================================================================

    #[test]
    fn test_let_simple_binding() {
        let env = Environment::new();

        // (let ((x 42)) x)
        let expr = Value::List(vec![
            Value::Symbol("let".to_string()),
            Value::List(vec![Value::List(vec![
                Value::Symbol("x".to_string()),
                Value::Number(42.0),
            ])]),
            Value::Symbol("x".to_string()),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_let_multiple_bindings() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());

        // (let ((x 10) (y 20)) (+ x y))
        let expr = Value::List(vec![
            Value::Symbol("let".to_string()),
            Value::List(vec![
                Value::List(vec![Value::Symbol("x".to_string()), Value::Number(10.0)]),
                Value::List(vec![Value::Symbol("y".to_string()), Value::Number(20.0)]),
            ]),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Symbol("x".to_string()),
                Value::Symbol("y".to_string()),
            ]),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 30.0),
            _ => panic!("Expected Number(30.0)"),
        }
    }

    #[test]
    fn test_let_shadowing() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());

        // Define x globally
        env.define("x".to_string(), Value::Number(100.0));

        // (let ((x 10)) x) - should shadow global x
        let expr = Value::List(vec![
            Value::Symbol("let".to_string()),
            Value::List(vec![Value::List(vec![
                Value::Symbol("x".to_string()),
                Value::Number(10.0),
            ])]),
            Value::Symbol("x".to_string()),
        ]);

        let result = eval(expr, env.clone()).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 10.0),
            _ => panic!("Expected Number(10.0)"),
        }

        // Global x should still be 100
        match env.get("x") {
            Some(Value::Number(n)) => assert_eq!(n, 100.0),
            _ => panic!("Expected global x to still be 100.0"),
        }
    }

    #[test]
    fn test_let_with_complex_expressions() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());

        // (let ((x (+ 1 2)) (y (* 3 4))) (+ x y))
        let expr = Value::List(vec![
            Value::Symbol("let".to_string()),
            Value::List(vec![
                Value::List(vec![
                    Value::Symbol("x".to_string()),
                    Value::List(vec![
                        Value::Symbol("+".to_string()),
                        Value::Number(1.0),
                        Value::Number(2.0),
                    ]),
                ]),
                Value::List(vec![
                    Value::Symbol("y".to_string()),
                    Value::List(vec![
                        Value::Symbol("*".to_string()),
                        Value::Number(3.0),
                        Value::Number(4.0),
                    ]),
                ]),
            ]),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Symbol("x".to_string()),
                Value::Symbol("y".to_string()),
            ]),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 15.0), // 3 + 12 = 15
            _ => panic!("Expected Number(15.0)"),
        }
    }

    #[test]
    fn test_let_empty_bindings() {
        let env = Environment::new();

        // (let () 42)
        let expr = Value::List(vec![
            Value::Symbol("let".to_string()),
            Value::List(vec![]),
            Value::Number(42.0),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }
    }

    #[test]
    fn test_let_multiple_body_expressions() {
        let env = Environment::new();

        // (let ((x 10)) 1 2 x)
        let expr = Value::List(vec![
            Value::Symbol("let".to_string()),
            Value::List(vec![Value::List(vec![
                Value::Symbol("x".to_string()),
                Value::Number(10.0),
            ])]),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Symbol("x".to_string()),
        ]);

        let result = eval(expr, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 10.0),
            _ => panic!("Expected Number(10.0)"),
        }
    }

    // ========================================================================
    // Tail Call Optimization Tests
    // ========================================================================

    #[test]
    fn test_tco_simple_recursion() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());

        // (define (sum n acc) (if (<= n 0) acc (sum (- n 1) (+ acc n))))
        let define_sum = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::List(vec![
                Value::Symbol("sum".to_string()),
                Value::Symbol("n".to_string()),
                Value::Symbol("acc".to_string()),
            ]),
            Value::List(vec![
                Value::Symbol("if".to_string()),
                Value::List(vec![
                    Value::Symbol("<=".to_string()),
                    Value::Symbol("n".to_string()),
                    Value::Number(0.0),
                ]),
                Value::Symbol("acc".to_string()),
                Value::List(vec![
                    Value::Symbol("sum".to_string()),
                    Value::List(vec![
                        Value::Symbol("-".to_string()),
                        Value::Symbol("n".to_string()),
                        Value::Number(1.0),
                    ]),
                    Value::List(vec![
                        Value::Symbol("+".to_string()),
                        Value::Symbol("acc".to_string()),
                        Value::Symbol("n".to_string()),
                    ]),
                ]),
            ]),
        ]);
        eval(define_sum, env.clone()).unwrap();

        // (sum 10 0) should be 55
        let call_sum = Value::List(vec![
            Value::Symbol("sum".to_string()),
            Value::Number(10.0),
            Value::Number(0.0),
        ]);
        let result = eval(call_sum, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 55.0),
            _ => panic!("Expected Number(55.0)"),
        }
    }

    #[test]
    fn test_tco_deep_recursion() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());

        // (define (sum n acc) (if (<= n 0) acc (sum (- n 1) (+ acc n))))
        let define_sum = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::List(vec![
                Value::Symbol("sum".to_string()),
                Value::Symbol("n".to_string()),
                Value::Symbol("acc".to_string()),
            ]),
            Value::List(vec![
                Value::Symbol("if".to_string()),
                Value::List(vec![
                    Value::Symbol("<=".to_string()),
                    Value::Symbol("n".to_string()),
                    Value::Number(0.0),
                ]),
                Value::Symbol("acc".to_string()),
                Value::List(vec![
                    Value::Symbol("sum".to_string()),
                    Value::List(vec![
                        Value::Symbol("-".to_string()),
                        Value::Symbol("n".to_string()),
                        Value::Number(1.0),
                    ]),
                    Value::List(vec![
                        Value::Symbol("+".to_string()),
                        Value::Symbol("acc".to_string()),
                        Value::Symbol("n".to_string()),
                    ]),
                ]),
            ]),
        ]);
        eval(define_sum, env.clone()).unwrap();

        // Test with 10000 - this would stack overflow without TCO
        let call_sum = Value::List(vec![
            Value::Symbol("sum".to_string()),
            Value::Number(10000.0),
            Value::Number(0.0),
        ]);
        let result = eval(call_sum, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 50005000.0), // sum of 1..10000
            _ => panic!("Expected Number(50005000.0)"),
        }
    }

    #[test]
    fn test_tco_with_begin() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());

        // (define (countdown n) (if (<= n 0) 0 (begin (countdown (- n 1)))))
        let define_countdown = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::List(vec![
                Value::Symbol("countdown".to_string()),
                Value::Symbol("n".to_string()),
            ]),
            Value::List(vec![
                Value::Symbol("if".to_string()),
                Value::List(vec![
                    Value::Symbol("<=".to_string()),
                    Value::Symbol("n".to_string()),
                    Value::Number(0.0),
                ]),
                Value::Number(0.0),
                Value::List(vec![
                    Value::Symbol("begin".to_string()),
                    Value::List(vec![
                        Value::Symbol("countdown".to_string()),
                        Value::List(vec![
                            Value::Symbol("-".to_string()),
                            Value::Symbol("n".to_string()),
                            Value::Number(1.0),
                        ]),
                    ]),
                ]),
            ]),
        ]);
        eval(define_countdown, env.clone()).unwrap();

        // Test with 5000 - should not stack overflow
        let call_countdown = Value::List(vec![
            Value::Symbol("countdown".to_string()),
            Value::Number(5000.0),
        ]);
        let result = eval(call_countdown, env).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 0.0),
            _ => panic!("Expected Number(0.0)"),
        }
    }

    // ========================================================================
    // Macro Tests
    // ========================================================================

    #[test]
    fn test_quasiquote_basic() {
        let env = Environment::new();
        let mut macro_reg = MacroRegistry::new();

        // `(1 2 3) should return (1 2 3)
        let expr = Value::List(vec![
            Value::Symbol("quasiquote".to_string()),
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
        ]);

        let result = eval_with_macros(expr, env, &mut macro_reg).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                match (&items[0], &items[1], &items[2]) {
                    (Value::Number(a), Value::Number(b), Value::Number(c)) => {
                        assert_eq!(*a, 1.0);
                        assert_eq!(*b, 2.0);
                        assert_eq!(*c, 3.0);
                    }
                    _ => panic!("Expected numbers"),
                }
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_quasiquote_with_unquote() {
        let env = Environment::new();
        let mut macro_reg = MacroRegistry::new();

        // Define x
        env.define("x".to_string(), Value::Number(42.0));

        // `(1 ,x 3) should return (1 42 3)
        let expr = Value::List(vec![
            Value::Symbol("quasiquote".to_string()),
            Value::List(vec![
                Value::Number(1.0),
                Value::List(vec![
                    Value::Symbol("unquote".to_string()),
                    Value::Symbol("x".to_string()),
                ]),
                Value::Number(3.0),
            ]),
        ]);

        let result = eval_with_macros(expr, env, &mut macro_reg).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                match (&items[0], &items[1], &items[2]) {
                    (Value::Number(a), Value::Number(b), Value::Number(c)) => {
                        assert_eq!(*a, 1.0);
                        assert_eq!(*b, 42.0);
                        assert_eq!(*c, 3.0);
                    }
                    _ => panic!("Expected numbers"),
                }
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_quasiquote_with_unquote_splicing() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());
        let mut macro_reg = MacroRegistry::new();

        // `(1 ,@(list 2 3) 4) should return (1 2 3 4)
        let expr = Value::List(vec![
            Value::Symbol("quasiquote".to_string()),
            Value::List(vec![
                Value::Number(1.0),
                Value::List(vec![
                    Value::Symbol("unquote-splicing".to_string()),
                    Value::List(vec![
                        Value::Symbol("list".to_string()),
                        Value::Number(2.0),
                        Value::Number(3.0),
                    ]),
                ]),
                Value::Number(4.0),
            ]),
        ]);

        let result = eval_with_macros(expr, env, &mut macro_reg).unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 4);
                match (&items[0], &items[1], &items[2], &items[3]) {
                    (Value::Number(a), Value::Number(b), Value::Number(c), Value::Number(d)) => {
                        assert_eq!(*a, 1.0);
                        assert_eq!(*b, 2.0);
                        assert_eq!(*c, 3.0);
                        assert_eq!(*d, 4.0);
                    }
                    _ => panic!("Expected numbers"),
                }
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_defmacro_simple() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());
        let mut macro_reg = MacroRegistry::new();

        // (defmacro when (test body) `(if ,test ,body nil))
        let defmacro_expr = Value::List(vec![
            Value::Symbol("defmacro".to_string()),
            Value::Symbol("when".to_string()),
            Value::List(vec![
                Value::Symbol("test".to_string()),
                Value::Symbol("body".to_string()),
            ]),
            Value::List(vec![
                Value::Symbol("quasiquote".to_string()),
                Value::List(vec![
                    Value::Symbol("if".to_string()),
                    Value::List(vec![
                        Value::Symbol("unquote".to_string()),
                        Value::Symbol("test".to_string()),
                    ]),
                    Value::List(vec![
                        Value::Symbol("unquote".to_string()),
                        Value::Symbol("body".to_string()),
                    ]),
                    Value::Nil,
                ]),
            ]),
        ]);

        let result = eval_with_macros(defmacro_expr, env.clone(), &mut macro_reg).unwrap();
        match result {
            Value::Symbol(s) => assert_eq!(s, "when"),
            _ => panic!("Expected Symbol(\"when\")"),
        }

        // Now use the macro: (when #t 42)
        let use_macro = Value::List(vec![
            Value::Symbol("when".to_string()),
            Value::Bool(true),
            Value::Number(42.0),
        ]);

        let result = eval_with_macros(use_macro, env.clone(), &mut macro_reg).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }

        // (when #f 42) should return nil
        let use_macro_false = Value::List(vec![
            Value::Symbol("when".to_string()),
            Value::Bool(false),
            Value::Number(42.0),
        ]);

        let result = eval_with_macros(use_macro_false, env, &mut macro_reg).unwrap();
        match result {
            Value::Nil => (),
            _ => panic!("Expected Nil"),
        }
    }

    #[test]
    fn test_defmacro_unless() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());
        let mut macro_reg = MacroRegistry::new();

        // (defmacro unless (test body) `(if ,test nil ,body))
        let defmacro_expr = Value::List(vec![
            Value::Symbol("defmacro".to_string()),
            Value::Symbol("unless".to_string()),
            Value::List(vec![
                Value::Symbol("test".to_string()),
                Value::Symbol("body".to_string()),
            ]),
            Value::List(vec![
                Value::Symbol("quasiquote".to_string()),
                Value::List(vec![
                    Value::Symbol("if".to_string()),
                    Value::List(vec![
                        Value::Symbol("unquote".to_string()),
                        Value::Symbol("test".to_string()),
                    ]),
                    Value::Nil,
                    Value::List(vec![
                        Value::Symbol("unquote".to_string()),
                        Value::Symbol("body".to_string()),
                    ]),
                ]),
            ]),
        ]);

        eval_with_macros(defmacro_expr, env.clone(), &mut macro_reg).unwrap();

        // (unless #f 42) should return 42
        let use_macro = Value::List(vec![
            Value::Symbol("unless".to_string()),
            Value::Bool(false),
            Value::Number(42.0),
        ]);

        let result = eval_with_macros(use_macro, env.clone(), &mut macro_reg).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number(42.0)"),
        }

        // (unless #t 42) should return nil
        let use_macro_true = Value::List(vec![
            Value::Symbol("unless".to_string()),
            Value::Bool(true),
            Value::Number(42.0),
        ]);

        let result = eval_with_macros(use_macro_true, env, &mut macro_reg).unwrap();
        match result {
            Value::Nil => (),
            _ => panic!("Expected Nil"),
        }
    }

    #[test]
    fn test_nested_quasiquote() {
        let env = Environment::new();
        let mut macro_reg = MacroRegistry::new();

        env.define("x".to_string(), Value::Number(42.0));

        // ``(1 ,x) should return `(1 ,x)
        let expr = Value::List(vec![
            Value::Symbol("quasiquote".to_string()),
            Value::List(vec![
                Value::Symbol("quasiquote".to_string()),
                Value::List(vec![
                    Value::Number(1.0),
                    Value::List(vec![
                        Value::Symbol("unquote".to_string()),
                        Value::Symbol("x".to_string()),
                    ]),
                ]),
            ]),
        ]);

        let result = eval_with_macros(expr, env, &mut macro_reg).unwrap();
        // Should return a list containing quasiquote symbol
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 2);
                match &items[0] {
                    Value::Symbol(s) => assert_eq!(s, "quasiquote"),
                    _ => panic!("Expected quasiquote symbol"),
                }
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_macro_with_computation() {
        let env = Environment::new();
        crate::builtins::register_builtins(env.clone());
        let mut macro_reg = MacroRegistry::new();

        // (defmacro square (x) `(* ,x ,x))
        let defmacro_expr = Value::List(vec![
            Value::Symbol("defmacro".to_string()),
            Value::Symbol("square".to_string()),
            Value::List(vec![Value::Symbol("x".to_string())]),
            Value::List(vec![
                Value::Symbol("quasiquote".to_string()),
                Value::List(vec![
                    Value::Symbol("*".to_string()),
                    Value::List(vec![
                        Value::Symbol("unquote".to_string()),
                        Value::Symbol("x".to_string()),
                    ]),
                    Value::List(vec![
                        Value::Symbol("unquote".to_string()),
                        Value::Symbol("x".to_string()),
                    ]),
                ]),
            ]),
        ]);

        eval_with_macros(defmacro_expr, env.clone(), &mut macro_reg).unwrap();

        // (square 5) should expand to (* 5 5) and evaluate to 25
        let use_macro = Value::List(vec![
            Value::Symbol("square".to_string()),
            Value::Number(5.0),
        ]);

        let result = eval_with_macros(use_macro, env, &mut macro_reg).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 25.0),
            _ => panic!("Expected Number(25.0)"),
        }
    }
}

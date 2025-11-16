// ABOUTME: Evaluator module for executing parsed Lisp expressions

use crate::env::Environment;
use crate::error::{EvalError, ARITY_ONE, ARITY_TWO_OR_THREE};
use crate::macros::MacroRegistry;
use crate::parser;
use crate::value::Value;
use std::sync::{Arc, RwLock};

// Thread-local storage for the global environment (used by define)
// This allows define to update the global environment while maintaining immutability
thread_local! {
    static GLOBAL_ENV: RwLock<Option<Arc<Environment>>> = RwLock::new(None);
}

/// Set the global environment for this thread (used by REPL/main)
pub fn set_global_env(env: Arc<Environment>) {
    GLOBAL_ENV.with(|global| {
        *global.write().unwrap() = Some(env);
    });
}

/// Get the current global environment
#[allow(dead_code)]
pub fn get_global_env() -> Option<Arc<Environment>> {
    GLOBAL_ENV.with(|global| global.read().unwrap().clone())
}

/// Update the global environment with a new binding
pub fn extend_global_env(name: String, value: Value) {
    GLOBAL_ENV.with(|global| {
        let mut guard = global.write().unwrap();
        if let Some(env) = guard.as_ref() {
            *guard = Some(env.extend(name, value));
        }
    });
}

/// Main evaluation function with tail call optimization
#[allow(dead_code)]
pub fn eval(expr: Value, env: Arc<Environment>) -> Result<Value, EvalError> {
    eval_with_macros(expr, env, &mut MacroRegistry::new())
}

/// Evaluation function with macro registry support
pub fn eval_with_macros(
    mut expr: Value,
    env: Arc<Environment>,
    macro_reg: &mut MacroRegistry,
) -> Result<Value, EvalError> {
    let mut current_env = env;
    loop {
        // First expand macros
        expr = expand_macros(expr.clone(), macro_reg, current_env.clone())?;

        match &expr {
            // Self-evaluating values
            Value::Number(_)
            | Value::Bool(_)
            | Value::String(_)
            | Value::Keyword(_)
            | Value::Channel { .. }
            | Value::Nil => {
                return Ok(expr.clone());
            }

            // Maps: evaluate all values
            Value::Map(map) => {
                use std::collections::HashMap;
                let mut evaluated_map = HashMap::new();
                for (key, value) in map {
                    let evaluated_value =
                        eval_with_macros(value.clone(), current_env.clone(), macro_reg)?;
                    evaluated_map.insert(key.clone(), evaluated_value);
                }
                return Ok(Value::Map(evaluated_map));
            }

            // Symbol lookup
            Value::Symbol(name) => {
                // Special case: 'nil' as a symbol evaluates to Nil value
                if name == "nil" {
                    return Ok(Value::Nil);
                }

                // Look in current environment first (for local bindings like let, lambda params)
                if let Some(val) = current_env.get(name) {
                    return Ok(val);
                }

                // Fallback to global environment if set (for top-level defines)
                if let Some(global_env) = get_global_env() {
                    if let Some(val) = global_env.get(name) {
                        return Ok(val);
                    }
                }

                // Not found anywhere
                return Err(EvalError::UndefinedSymbol(name.clone()));
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
                            return Err(EvalError::arity_error(
                                "quote",
                                ARITY_ONE,
                                items.len() - 1,
                            ));
                        }
                        return Ok(items[1].clone());
                    }
                    Value::Symbol(s) if s == "quasiquote" => {
                        if items.len() != 2 {
                            return Err(EvalError::arity_error(
                                "quasiquote",
                                ARITY_ONE,
                                items.len() - 1,
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
                            return Err(EvalError::arity_error(
                                "if",
                                ARITY_TWO_OR_THREE,
                                items.len() - 1,
                            ));
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
                                    // Get lambda name if available (from define)
                                    let name = match &items[0] {
                                        Value::Symbol(s) => s.as_str(),
                                        _ => "<lambda>",
                                    };
                                    return Err(EvalError::arity_error(
                                        name,
                                        params.len().to_string(),
                                        args.len(),
                                    ));
                                }

                                // Create new environment for lambda
                                let mut new_env = Environment::with_parent(lambda_env);
                                for (param, arg) in params.iter().zip(args.iter()) {
                                    new_env = new_env.extend(param.clone(), arg.clone());
                                }

                                // Tail call: set up for next iteration
                                expr = *body;
                                current_env = new_env;
                                // Continue loop
                            }
                            Value::BuiltIn(f) => {
                                // All builtins now include function context in errors
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
    env: Arc<Environment>,
    macro_reg: &mut MacroRegistry,
) -> Result<Value, EvalError> {
    if args.len() < 2 {
        return Err(EvalError::arity_error("define", "at least 2", args.len()));
    }

    // Initialize global environment if not set (for tests)
    GLOBAL_ENV.with(|global| {
        let guard = global.read().unwrap();
        if guard.is_none() {
            drop(guard);
            set_global_env(env.clone());
        }
    });

    match &args[0] {
        // Variable definition: (define x 42)
        Value::Symbol(name) => {
            let value = eval_with_macros(args[1].clone(), env.clone(), macro_reg)?;
            extend_global_env(name.clone(), value);
            Ok(Value::Symbol(name.clone()))
        }

        // Function definition: (define (f x y) body)
        Value::List(func_def) if !func_def.is_empty() => {
            // Extract function name
            let name = match &func_def[0] {
                Value::Symbol(n) => n.clone(),
                _ => {
                    return Err(EvalError::runtime_error(
                        "define",
                        "function name must be a symbol",
                    ));
                }
            };

            // Extract parameters
            let mut params = Vec::new();
            for param in &func_def[1..] {
                match param {
                    Value::Symbol(p) => params.push(p.clone()),
                    _ => {
                        return Err(EvalError::runtime_error(
                            "define",
                            "function parameters must be symbols",
                        ));
                    }
                }
            }

            // Extract docstring if present: (define (f x) "doc" body)
            let (inline_docstring, body) = match &args[1] {
                Value::String(s) if args.len() > 2 => (Some(s.clone()), Box::new(args[2].clone())),
                _ => (None, Box::new(args[1].clone())),
            };

            // Check for pending doc comments from ;;; and merge with inline docstring
            let pending_docs = parser::take_pending_docs();
            let docstring = if !pending_docs.is_empty() {
                // Prefer pending docs (;;; comments) over inline docstrings
                Some(pending_docs.join("\n"))
            } else {
                inline_docstring
            };

            // Register help entry if we have documentation (unless we're loading stdlib)
            if let Some(ref doc) = docstring {
                if !parser::should_skip_help_registration() {
                    let signature = format!("({} {})", name, params.join(" "));
                    crate::help::register_help(crate::help::HelpEntry {
                        name: name.clone(),
                        signature,
                        description: doc.clone(),
                        examples: vec![], // Could parse from doc later
                        related: vec![],
                        category: "User Defined".to_string(),
                    });
                }
            }

            // Create lambda
            let lambda = Value::Lambda {
                params,
                body,
                env: Arc::clone(&env),
                docstring,
            };

            // Define it in global env
            extend_global_env(name.clone(), lambda);
            Ok(Value::Symbol(name))
        }

        _ => Err(EvalError::runtime_error(
            "define",
            "requires a symbol or list as first argument",
        )),
    }
}

/// Evaluate a lambda expression
/// (lambda (x y z) body) or (lambda (x y z) "docstring" body)
fn eval_lambda(args: &[Value], env: Arc<Environment>) -> Result<Value, EvalError> {
    if args.len() < 2 {
        return Err(EvalError::arity_error("lambda", "at least 2", args.len()));
    }

    // Extract parameters from args[0]
    let params = match &args[0] {
        Value::List(param_list) => {
            let mut params = Vec::new();
            for param in param_list {
                match param {
                    Value::Symbol(name) => params.push(name.clone()),
                    _ => {
                        return Err(EvalError::runtime_error(
                            "lambda",
                            "parameters must be symbols",
                        ));
                    }
                }
            }
            params
        }
        Value::Nil => {
            // Empty parameter list () is parsed as Nil
            Vec::new()
        }
        _ => {
            return Err(EvalError::runtime_error(
                "lambda",
                "parameters must be a list",
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
    env: Arc<Environment>,
    macro_reg: &mut MacroRegistry,
) -> Result<Value, EvalError> {
    if args.is_empty() {
        return Err(EvalError::arity_error("let", "at least 1", 0));
    }

    let bindings = match &args[0] {
        Value::List(items) => items,
        _ => return Err(EvalError::runtime_error("let", "bindings must be a list")),
    };

    // Create new environment as child of current env
    let mut new_env = Environment::with_parent(env);

    // Evaluate bindings and add to new environment
    for binding in bindings {
        match binding {
            Value::List(pair) if pair.len() == 2 => {
                let name = match &pair[0] {
                    Value::Symbol(s) => s.clone(),
                    _ => {
                        return Err(EvalError::runtime_error(
                            "let",
                            "binding name must be symbol",
                        ))
                    }
                };
                let value = eval_with_macros(pair[1].clone(), new_env.clone(), macro_reg)?;
                new_env = new_env.extend(name, value);
            }
            _ => {
                return Err(EvalError::runtime_error(
                    "let",
                    "binding must be [symbol value]",
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
    env: Arc<Environment>,
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
                        return Err(EvalError::arity_error(
                            "unquote",
                            ARITY_ONE,
                            items.len() - 1,
                        ));
                    }
                    eval_with_macros(items[1].clone(), env, macro_reg)
                }

                // (quasiquote ...) → increase depth and recurse
                Value::Symbol(s) if s == "quasiquote" => {
                    if items.len() != 2 {
                        return Err(EvalError::arity_error(
                            "quasiquote",
                            ARITY_ONE,
                            items.len() - 1,
                        ));
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
                                        return Err(EvalError::arity_error(
                                            "unquote-splicing",
                                            "1",
                                            parts.len() - 1,
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
                                            return Err(EvalError::runtime_error(
                                                "unquote-splicing",
                                                "requires a list",
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
    _env: Arc<Environment>,
    macro_reg: &mut MacroRegistry,
) -> Result<Value, EvalError> {
    if args.len() < 3 {
        return Err(EvalError::arity_error("defmacro", "at least 3", args.len()));
    }

    let name = match &args[0] {
        Value::Symbol(n) => n.clone(),
        _ => {
            return Err(EvalError::runtime_error(
                "defmacro",
                "name must be a symbol",
            ))
        }
    };

    let params = match &args[1] {
        Value::List(p) => p
            .iter()
            .map(|v| match v {
                Value::Symbol(s) => Ok(s.clone()),
                _ => Err(EvalError::runtime_error(
                    "defmacro",
                    "parameter must be symbol",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => {
            return Err(EvalError::runtime_error(
                "defmacro",
                "params must be a list",
            ))
        }
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
    env: Arc<Environment>,
) -> Result<Value, EvalError> {
    match expr {
        Value::List(ref items) if !items.is_empty() => {
            match &items[0] {
                Value::Symbol(name) => {
                    if let Some((params, body)) = macro_reg.get(name) {
                        // Bind arguments to parameters
                        let args = &items[1..];

                        if params.len() != args.len() {
                            return Err(EvalError::arity_error(
                                name,
                                params.len().to_string(),
                                args.len(),
                            ));
                        }

                        let mut macro_env = Environment::with_parent(env.clone());
                        for (param, arg) in params.iter().zip(args.iter()) {
                            // Arguments to macros are NOT evaluated yet
                            macro_env = macro_env.extend(param.clone(), arg.clone());
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

/// Register help documentation for special forms (Part 1)
/// Documents: define, lambda, if, begin
pub fn register_special_forms_part1() {
    crate::help::register_help(crate::help::HelpEntry {
        name: "define".to_string(),
        signature: "(define name value) or (define (name params...) body)".to_string(),
        description: "Define a variable or function in the current scope.\n\nThe first form binds a value to a name. The second form is syntactic sugar for defining a function, equivalent to `(define name (lambda (params...) body))`.\n\nReturns the name of the defined symbol.".to_string(),
        examples: vec![
            "(define x 42) => x".to_string(),
            "(define (square x) (* x x)) => square".to_string(),
            "(define (add a b) (+ a b)) => add".to_string(),
            "(add 3 4) => 7".to_string(),
        ],
        related: vec!["lambda".to_string(), "let".to_string()],
        category: "Special forms".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "lambda".to_string(),
        signature: "(lambda (params...) [docstring] body)".to_string(),
        description: "Create an anonymous function.\n\nThe parameters are a list of symbols. The body is evaluated when the function is called with the parameters bound to the argument values. Optionally, a docstring can be provided as the first element of the body.\n\nThe created function captures the lexical environment at definition time, enabling closures.".to_string(),
        examples: vec![
            "((lambda (x) (+ x 1)) 5) => 6".to_string(),
            "(define add (lambda (a b) (+ a b))) => add".to_string(),
            "(define make-adder (lambda (n) (lambda (x) (+ x n)))) => make-adder".to_string(),
            "((make-adder 10) 5) => 15".to_string(),
        ],
        related: vec!["define".to_string(), "let".to_string(), "doc".to_string()],
        category: "Special forms".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "if".to_string(),
        signature: "(if condition then-expr [else-expr])".to_string(),
        description: "Conditional evaluation.\n\nIf condition evaluates to a truthy value (anything except false), then-expr is evaluated and returned. Otherwise, else-expr is evaluated (if provided) and returned. If no else-expr is provided and condition is false, returns nil.\n\nOnly the taken branch is evaluated (short-circuit evaluation).".to_string(),
        examples: vec![
            "(if (> 5 3) \"yes\" \"no\") => \"yes\"".to_string(),
            "(if false 42) => nil".to_string(),
            "(if true (+ 1 2) (/ 1 0)) => 3".to_string(),
            "(define (abs x) (if (< x 0) (- x) x)) => abs".to_string(),
        ],
        related: vec!["begin".to_string(), "and".to_string(), "or".to_string()],
        category: "Special forms".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "begin".to_string(),
        signature: "(begin expr1 expr2 ... exprN)".to_string(),
        description: "Sequence multiple expressions.\n\nEvaluates each expression in order and returns the value of the last expression. All expressions are evaluated for their side effects, but only the final value is returned.\n\nUseful for grouping expressions in contexts that expect a single expression, like in `if` branches or `lambda` bodies.".to_string(),
        examples: vec![
            "(begin (print \"step 1\") (print \"step 2\") 42) => 42".to_string(),
            "(define (side-effects) (begin (print \"first\") (print \"second\") \"result\")) => side-effects".to_string(),
            "(if condition (begin (print \"doing something\") (+ 1 2)) 0)".to_string(),
        ],
        related: vec!["if".to_string(), "define".to_string()],
        category: "Special forms".to_string(),
    });
}

/// Register help documentation for special forms (Part 2)
/// Documents: let, quote, quasiquote, defmacro
pub fn register_special_forms_part2() {
    crate::help::register_help(crate::help::HelpEntry {
        name: "let".to_string(),
        signature: "(let ((var1 expr1) (var2 expr2) ...) body)".to_string(),
        description: "Create local variable bindings.\n\nDefines temporary variables that are visible only within the body. Each variable is bound to the value of its corresponding expression. All binding expressions are evaluated in the outer scope before the body is evaluated.\n\nEquivalent to `((lambda (var1 var2 ...) body) expr1 expr2 ...)`.\n\nUseful for avoiding repeated calculations and improving code clarity.".to_string(),
        examples: vec![
            "(let ((x 10) (y 20)) (+ x y)) => 30".to_string(),
            "(let ((a (+ 1 2)) (b (* 3 4))) (+ a b)) => 15".to_string(),
            "(let ((x 5)) (let ((y 10)) (+ x y))) => 15".to_string(),
            "(define (quadratic a b c x) (let ((delta (- (* b b) (* 4 a c)))) (/ delta 2))) => quadratic".to_string(),
        ],
        related: vec!["lambda".to_string(), "define".to_string()],
        category: "Special forms".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "quote".to_string(),
        signature: "(quote expr) or 'expr".to_string(),
        description: "Return an expression unevaluated.\n\nPrevents evaluation of the expression. Returns the expression itself as data.\n\nOften used with symbols and lists to create data structures that would otherwise be evaluated as code. The shorthand syntax 'expr is equivalent to (quote expr).".to_string(),
        examples: vec![
            "'x => x (the symbol, not its value)".to_string(),
            "'(1 2 3) => (1 2 3) (the list, not evaluated as function call)".to_string(),
            "(quote (+ 1 2)) => (+ 1 2)".to_string(),
            "'() => () (empty list)".to_string(),
        ],
        related: vec!["quasiquote".to_string()],
        category: "Special forms".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "quasiquote".to_string(),
        signature: "(quasiquote template) or `template".to_string(),
        description: "Return a template with selective evaluation.\n\nLike quote, but allows selective evaluation of parts using unquote (,) and unquote-splicing (,@).\n\nUnquoted parts are evaluated; unquoted-spliced lists are spliced into the result. This is the foundation of the macro system.".to_string(),
        examples: vec![
            "`(+ 1 2) => (+ 1 2)".to_string(),
            "`(+ 1 ,(+ 2 3)) => (+ 1 5)".to_string(),
            "`(list ,@(list 1 2 3)) => (list 1 2 3)".to_string(),
            "(define x 10) => x".to_string(),
            "`(x is ,x) => (x is 10)".to_string(),
        ],
        related: vec!["quote".to_string(), "defmacro".to_string()],
        category: "Special forms".to_string(),
    });

    crate::help::register_help(crate::help::HelpEntry {
        name: "defmacro".to_string(),
        signature: "(defmacro (name params...) [docstring] body)".to_string(),
        description: "Define a compile-time transformation.\n\nMacros receive unevaluated arguments and return code to be evaluated. Unlike functions, macro arguments are not evaluated before the macro is called. The macro body should return a list representing the code to evaluate.\n\nMacros enable syntactic abstraction and domain-specific languages.".to_string(),
        examples: vec![
            "(defmacro (when condition body) `(if ,condition ,body))".to_string(),
            "(defmacro (repeat n body) `(let ((i 0)) (while (< i ,n) (begin ,body (set! i (+ i 1))))))".to_string(),
            "(defmacro (assert condition) `(if (not ,condition) (error \"Assertion failed\")))".to_string(),
        ],
        related: vec!["quote".to_string(), "quasiquote".to_string(), "lambda".to_string()],
        category: "Special forms".to_string(),
    });
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
        let env = env.extend("x".to_string(), Value::Number(42.0));

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

        // Check that x is now defined in the global environment
        let global_env = get_global_env().expect("Global env should be set");
        match global_env.get("x") {
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

        // Check that f is now defined as a lambda in global env
        let global_env = get_global_env().expect("Global env should be set");
        match global_env.get("f") {
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
    fn test_shadowing_with_let() {
        // Note: In our immutable architecture, 'define' creates global bindings,
        // while 'let' creates local bindings. This test shows local shadowing with 'let'.
        let env = Environment::new();
        let env = env.extend("x".to_string(), Value::Number(10.0));

        // Use let to shadow x locally
        let expr = Value::List(vec![
            Value::Symbol("let".to_string()),
            Value::List(vec![Value::List(vec![
                Value::Symbol("x".to_string()),
                Value::Number(20.0),
            ])]),
            Value::Symbol("x".to_string()),
        ]);

        let result = eval(expr, env.clone()).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 20.0),
            _ => panic!("Expected Number(20.0)"),
        }

        // Original x should still be 10
        match env.get("x") {
            Some(Value::Number(n)) => assert_eq!(n, 10.0),
            _ => panic!("Expected x to still be 10.0"),
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
        assert!(matches!(result, Err(EvalError::ArityError { .. })));
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

        // Verify x was also defined in global env
        let global_env = get_global_env().expect("Global env should be set");
        match global_env.get("x") {
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
        let env = env.extend("x".to_string(), Value::Number(100.0));

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

        // Global x should still be 100 (unchanged by let)
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
        let env = env.extend("x".to_string(), Value::Number(42.0));

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

        let env = env.extend("x".to_string(), Value::Number(42.0));

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

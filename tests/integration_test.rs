// ABOUTME: Comprehensive integration tests verifying all features work together

use lisp_llm_sandbox::*;
use std::rc::Rc;

/// Set up environment with builtins and stdlib loaded
fn setup() -> (Rc<env::Environment>, macros::MacroRegistry) {
    let env = env::Environment::new();
    let mut macro_reg = macros::MacroRegistry::new();
    builtins::register_builtins(env.clone());

    // Load stdlib
    let stdlib = include_str!("../src/stdlib.lisp");
    load_stdlib(stdlib, env.clone(), &mut macro_reg).expect("Failed to load stdlib");

    (env, macro_reg)
}

/// Load stdlib code into environment
fn load_stdlib(
    code: &str,
    env: Rc<env::Environment>,
    macro_reg: &mut macros::MacroRegistry,
) -> Result<(), String> {
    let mut remaining = code.trim();

    while !remaining.is_empty() {
        remaining = skip_whitespace_and_comments(remaining);
        if remaining.is_empty() {
            break;
        }

        match parse_one_expr(remaining) {
            Ok((expr, rest)) => match eval::eval_with_macros(expr, env.clone(), macro_reg) {
                Ok(_) => {
                    remaining = rest;
                }
                Err(e) => {
                    return Err(format!("Eval error: {:?}", e));
                }
            },
            Err(e) => {
                return Err(format!("Parse error: {}", e));
            }
        }
    }

    Ok(())
}

fn skip_whitespace_and_comments(input: &str) -> &str {
    let mut remaining = input;
    loop {
        remaining = remaining.trim_start();
        if remaining.starts_with(';') {
            if let Some(pos) = remaining.find('\n') {
                remaining = &remaining[pos + 1..];
            } else {
                remaining = "";
            }
        } else {
            break;
        }
    }
    remaining
}

fn parse_one_expr(input: &str) -> Result<(value::Value, &str), String> {
    let trimmed = skip_whitespace_and_comments(input);
    if trimmed.is_empty() {
        return Err("No expression to parse".to_string());
    }

    let end_pos = find_expr_end(trimmed)?;
    let expr_str = &trimmed[..end_pos];
    let rest = &trimmed[end_pos..];

    let expr = parser::parse(expr_str)?;
    Ok((expr, rest))
}

fn find_expr_end(input: &str) -> Result<usize, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }

    if i >= chars.len() {
        return Err("Empty input".to_string());
    }

    if chars[i] == '(' {
        let mut depth = 0;
        let mut in_string = false;

        while i < chars.len() {
            match chars[i] {
                '"' => in_string = !in_string,
                '(' if !in_string => depth += 1,
                ')' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(i + 1);
                    }
                }
                _ => {}
            }
            i += 1;
        }

        Err("Unclosed s-expression".to_string())
    } else {
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != ')' {
            i += 1;
        }
        Ok(i)
    }
}

fn eval_code(
    code: &str,
    env: Rc<env::Environment>,
    macro_reg: &mut macros::MacroRegistry,
) -> Result<value::Value, String> {
    let expr = parser::parse(code).map_err(|e| format!("Parse error: {}", e))?;
    eval::eval_with_macros(expr, env, macro_reg).map_err(|e| format!("Eval error: {:?}", e))
}

// ============================================================================
// Integration Tests: Complete Programs
// ============================================================================

#[test]
fn test_factorial_program() {
    let (env, mut macro_reg) = setup();

    // Define factorial using recursion
    let code = r#"
    (define (factorial n)
      (if (<= n 1)
          1
          (* n (factorial (- n 1)))))
    "#;
    eval_code(code, env.clone(), &mut macro_reg).unwrap();

    // Test factorial(5)
    let result = eval_code("(factorial 5)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 120.0),
        _ => panic!("Expected Number(120), got {:?}", result),
    }

    // Test factorial(0)
    let result = eval_code("(factorial 0)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 1.0),
        _ => panic!("Expected Number(1), got {:?}", result),
    }
}

#[test]
fn test_fibonacci_program() {
    let (env, mut macro_reg) = setup();

    // Define fibonacci using recursion
    let code = r#"
    (define (fib n)
      (if (< n 2)
          n
          (+ (fib (- n 1)) (fib (- n 2)))))
    "#;
    eval_code(code, env.clone(), &mut macro_reg).unwrap();

    // Test fib(10) = 55
    let result = eval_code("(fib 10)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 55.0),
        _ => panic!("Expected Number(55), got {:?}", result),
    }

    // Test fib(0) = 0
    let result = eval_code("(fib 0)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 0.0),
        _ => panic!("Expected Number(0), got {:?}", result),
    }
}

#[test]
fn test_higher_order_functions() {
    let (env, mut macro_reg) = setup();

    // Test map from stdlib
    let result = eval_code(
        "(map (lambda (x) (* x 2)) '(1 2 3))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();
    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], value::Value::Number(n) if n == 2.0));
            assert!(matches!(items[1], value::Value::Number(n) if n == 4.0));
            assert!(matches!(items[2], value::Value::Number(n) if n == 6.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }

    // Test filter from stdlib
    let result = eval_code(
        "(filter (lambda (x) (> x 2)) '(1 2 3 4 5))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();
    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], value::Value::Number(n) if n == 3.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }

    // Test reduce from stdlib
    let result = eval_code("(reduce + 0 '(1 2 3 4))", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 10.0),
        _ => panic!("Expected Number(10), got {:?}", result),
    }
}

#[test]
fn test_macro_expansion() {
    let (env, mut macro_reg) = setup();

    // Define a simple test macro
    eval_code(
        r#"
    (defmacro when (test expr)
      `(if ,test ,expr nil))
    "#,
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();

    // Test when macro (expands to if)
    let result = eval_code("(when #t 42)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected Number(42), got {:?}", result),
    }

    // Test when with false condition
    let result = eval_code("(when #f 42)", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Nil));

    // Define unless macro
    eval_code(
        r#"
    (defmacro unless (test expr)
      `(if ,test nil ,expr))
    "#,
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();

    // Test unless macro
    let result = eval_code("(unless #f 100)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 100.0),
        _ => panic!("Expected Number(100), got {:?}", result),
    }
}

#[test]
fn test_tco_deep_recursion() {
    let (env, mut macro_reg) = setup();

    // Define tail-recursive sum function
    let code = r#"
    (define (sum n acc)
      (if (<= n 0)
          acc
          (sum (- n 1) (+ acc n))))
    "#;
    eval_code(code, env.clone(), &mut macro_reg).unwrap();

    // Test with 10000 iterations - would stack overflow without TCO
    let result = eval_code("(sum 10000 0)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => {
            // Sum of 1 to 10000 = 10000 * 10001 / 2 = 50005000
            assert_eq!(n, 50005000.0);
        }
        _ => panic!("Expected Number, got {:?}", result),
    }
}

#[test]
fn test_closures() {
    let (env, mut macro_reg) = setup();

    // Define a function that returns a closure
    let code = r#"
    (define (make-adder n)
      (lambda (x) (+ n x)))
    "#;
    eval_code(code, env.clone(), &mut macro_reg).unwrap();

    // Create an adder that adds 5
    eval_code("(define add5 (make-adder 5))", env.clone(), &mut macro_reg).unwrap();

    // Test the closure
    let result = eval_code("(add5 10)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 15.0),
        _ => panic!("Expected Number(15), got {:?}", result),
    }

    // Create another adder with different value
    eval_code(
        "(define add100 (make-adder 100))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();
    let result = eval_code("(add100 23)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 123.0),
        _ => panic!("Expected Number(123), got {:?}", result),
    }
}

#[test]
fn test_list_operations() {
    let (env, mut macro_reg) = setup();

    // Test cons
    let result = eval_code("(cons 1 (list 2 3 4))", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 4);
            assert!(matches!(items[0], value::Value::Number(n) if n == 1.0));
            assert!(matches!(items[1], value::Value::Number(n) if n == 2.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }

    // Test car
    let result = eval_code("(car '(1 2 3))", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 1.0),
        _ => panic!("Expected Number(1), got {:?}", result),
    }

    // Test cdr
    let result = eval_code("(cdr '(1 2 3))", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(items[0], value::Value::Number(n) if n == 2.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }

    // Test append
    let result = eval_code("(append '(1 2) '(3 4))", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 4);
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_quoting() {
    let (env, mut macro_reg) = setup();

    // Test simple quote
    let result = eval_code("'(1 2 3)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(items) => assert_eq!(items.len(), 3),
        _ => panic!("Expected List, got {:?}", result),
    }

    // Test quasiquote with unquote
    let result = eval_code("`(1 ,(+ 2 3) 4)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], value::Value::Number(n) if n == 1.0));
            assert!(matches!(items[1], value::Value::Number(n) if n == 5.0));
            assert!(matches!(items[2], value::Value::Number(n) if n == 4.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }

    // Test quasiquote with unquote-splicing
    let result = eval_code("`(1 ,@(list 2 3) 4)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 4);
            assert!(matches!(items[0], value::Value::Number(n) if n == 1.0));
            assert!(matches!(items[1], value::Value::Number(n) if n == 2.0));
            assert!(matches!(items[2], value::Value::Number(n) if n == 3.0));
            assert!(matches!(items[3], value::Value::Number(n) if n == 4.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_let_bindings() {
    let (env, mut macro_reg) = setup();

    // Define outer x
    eval_code("(define x 10)", env.clone(), &mut macro_reg).unwrap();

    // Test let with shadowing
    let result = eval_code(
        r#"
        (let ((x 20))
          (+ x 5))
    "#,
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();

    match result {
        value::Value::Number(n) => assert_eq!(n, 25.0),
        _ => panic!("Expected Number(25), got {:?}", result),
    }

    // Verify outer x is still 10
    let result = eval_code("x", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 10.0),
        _ => panic!("Expected Number(10), got {:?}", result),
    }
}

#[test]
fn test_complex_nested_expressions() {
    let (env, mut macro_reg) = setup();

    // Complex expression combining multiple features
    let code = r#"
    (define (process-list lst)
      (let ((doubled (map (lambda (x) (* x 2)) lst)))
        (filter (lambda (x) (> x 5)) doubled)))
    "#;
    eval_code(code, env.clone(), &mut macro_reg).unwrap();

    let result = eval_code("(process-list '(1 2 3 4 5))", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::List(items) => {
            // Original: [1, 2, 3, 4, 5]
            // Doubled: [2, 4, 6, 8, 10]
            // Filtered (> 5): [6, 8, 10]
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], value::Value::Number(n) if n == 6.0));
            assert!(matches!(items[1], value::Value::Number(n) if n == 8.0));
            assert!(matches!(items[2], value::Value::Number(n) if n == 10.0));
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_curry_and_composition() {
    let (env, mut macro_reg) = setup();

    // Test function composition from stdlib
    eval_code(
        "(define double (lambda (x) (* x 2)))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();
    eval_code(
        "(define inc (lambda (x) (+ x 1)))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();

    let result = eval_code("((compose double inc) 5)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 12.0), // (5 + 1) * 2 = 12
        _ => panic!("Expected Number(12), got {:?}", result),
    }
}

#[test]
fn test_predicates_and_logic() {
    let (env, mut macro_reg) = setup();

    // Test type predicates from builtins
    let result = eval_code("(list? '(1 2 3))", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Bool(true)));

    let result = eval_code("(number? 42)", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Bool(true)));

    let result = eval_code("(string? \"hello\")", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Bool(true)));

    // Test logical operations
    let result = eval_code("(and #t #t)", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Bool(true)));

    let result = eval_code("(or #f #t)", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Bool(true)));

    let result = eval_code("(not #f)", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Bool(true)));
}

#[test]
fn test_arithmetic_operations() {
    let (env, mut macro_reg) = setup();

    // Test basic arithmetic
    let result = eval_code("(+ 1 2 3 4)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 10.0),
        _ => panic!("Expected Number(10), got {:?}", result),
    }

    let result = eval_code("(* 2 3 4)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 24.0),
        _ => panic!("Expected Number(24), got {:?}", result),
    }

    let result = eval_code("(- 10 3)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 7.0),
        _ => panic!("Expected Number(7), got {:?}", result),
    }

    let result = eval_code("(/ 20 4)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 5.0),
        _ => panic!("Expected Number(5), got {:?}", result),
    }
}

#[test]
fn test_quicksort_algorithm() {
    let (env, mut macro_reg) = setup();

    // Implement quicksort in Lisp - chain append calls since it takes only 2 args
    let code = r#"
    (define (quicksort lst)
      (if (empty? lst)
          '()
          (append
            (quicksort (filter (lambda (x) (< x (car lst))) (cdr lst)))
            (append
              (list (car lst))
              (quicksort (filter (lambda (x) (>= x (car lst))) (cdr lst)))))))
    "#;
    eval_code(code, env.clone(), &mut macro_reg).unwrap();

    // Test quicksort
    let result = eval_code(
        "(quicksort '(3 1 4 1 5 9 2 6))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();
    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 8);
            assert!(matches!(items[0], value::Value::Number(n) if n == 1.0));
            assert!(matches!(items[1], value::Value::Number(n) if n == 1.0));
            assert!(matches!(items[2], value::Value::Number(n) if n == 2.0));
            assert!(matches!(items[3], value::Value::Number(n) if n == 3.0));
            assert!(matches!(items[4], value::Value::Number(n) if n == 4.0));
            assert!(matches!(items[5], value::Value::Number(n) if n == 5.0));
            assert!(matches!(items[6], value::Value::Number(n) if n == 6.0));
            assert!(matches!(items[7], value::Value::Number(n) if n == 9.0));
        }
        _ => panic!("Expected sorted List, got {:?}", result),
    }
}

#[test]
fn test_error_conditions() {
    let (env, mut macro_reg) = setup();

    // Test undefined variable
    let result = eval_code("undefined-var", env.clone(), &mut macro_reg);
    assert!(result.is_err());

    // Test division by zero
    let result = eval_code("(/ 1 0)", env.clone(), &mut macro_reg);
    assert!(result.is_err());

    // Test invalid function application
    let result = eval_code("(42)", env.clone(), &mut macro_reg);
    assert!(result.is_err());
}

#[test]
fn test_multiple_definitions() {
    let (env, mut macro_reg) = setup();

    // Define multiple functions and use them together
    eval_code("(define (add a b) (+ a b))", env.clone(), &mut macro_reg).unwrap();
    eval_code("(define (mul a b) (* a b))", env.clone(), &mut macro_reg).unwrap();
    eval_code("(define (square x) (mul x x))", env.clone(), &mut macro_reg).unwrap();

    let result = eval_code("(add (square 3) (square 4))", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 25.0), // 9 + 16 = 25
        _ => panic!("Expected Number(25), got {:?}", result),
    }
}

#[test]
fn test_begin_sequencing() {
    let (env, mut macro_reg) = setup();

    // Test begin with side effects
    let result = eval_code(
        r#"
        (begin
          (define x 10)
          (define y 20)
          (+ x y))
    "#,
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();

    match result {
        value::Value::Number(n) => assert_eq!(n, 30.0),
        _ => panic!("Expected Number(30), got {:?}", result),
    }

    // Verify variables were defined
    let result = eval_code("x", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 10.0),
        _ => panic!("Expected Number(10), got {:?}", result),
    }
}

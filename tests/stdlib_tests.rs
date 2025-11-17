// ABOUTME: Tests for standard library functions

use lisp_llm_sandbox::*;
use std::rc::Rc;

fn setup() -> (Rc<env::Environment>, macros::MacroRegistry) {
    let env = env::Environment::new();
    let mut macro_reg = macros::MacroRegistry::new();
    builtins::register_builtins(env.clone());

    // Load stdlib modules (same order as main.rs)
    let modules = [
        include_str!("../src/stdlib/lisp/core.lisp"),
        include_str!("../src/stdlib/lisp/math.lisp"),
        include_str!("../src/stdlib/lisp/string.lisp"),
        include_str!("../src/stdlib/lisp/test.lisp"),
        include_str!("../src/stdlib/lisp/http.lisp"),
        include_str!("../src/stdlib/lisp/db.lisp"),
    ];

    for module in &modules {
        load_stdlib_test(module, env.clone(), &mut macro_reg).expect("Failed to load stdlib module");
    }

    (env, macro_reg)
}

fn load_stdlib_test(
    code: &str,
    env: Rc<env::Environment>,
    macro_reg: &mut macros::MacroRegistry,
) -> Result<(), String> {
    let mut remaining = code.trim();

    while !remaining.is_empty() {
        remaining = skip_whitespace_and_comments_test(remaining);
        if remaining.is_empty() {
            break;
        }

        match parse_one_expr_test(remaining) {
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

fn skip_whitespace_and_comments_test(input: &str) -> &str {
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

fn parse_one_expr_test(input: &str) -> Result<(value::Value, &str), String> {
    let trimmed = skip_whitespace_and_comments_test(input);
    if trimmed.is_empty() {
        return Err("No expression to parse".to_string());
    }

    let end_pos = find_expr_end_test(trimmed)?;
    let expr_str = &trimmed[..end_pos];
    let rest = &trimmed[end_pos..];

    let expr = parser::parse(expr_str)?;
    Ok((expr, rest))
}

fn find_expr_end_test(input: &str) -> Result<usize, String> {
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
// Higher-Order Functions Tests
// ============================================================================

#[test]
fn test_map() {
    let (env, mut macro_reg) = setup();

    let result = eval_code(
        "(map (lambda (x) (* x 2)) '(1 2 3))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();

    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 3);
            match &items[0] {
                value::Value::Number(n) => assert_eq!(*n, 2.0),
                _ => panic!("Expected Number"),
            }
            match &items[1] {
                value::Value::Number(n) => assert_eq!(*n, 4.0),
                _ => panic!("Expected Number"),
            }
            match &items[2] {
                value::Value::Number(n) => assert_eq!(*n, 6.0),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected List, got {:?}", result),
    }
}

#[test]
fn test_filter() {
    let (env, mut macro_reg) = setup();

    let result = eval_code(
        "(filter (lambda (x) (> x 2)) '(1 2 3 4 5))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();

    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 3);
            match &items[0] {
                value::Value::Number(n) => assert_eq!(*n, 3.0),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_reduce() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(reduce + 0 '(1 2 3 4))", env.clone(), &mut macro_reg).unwrap();

    match result {
        value::Value::Number(n) => assert_eq!(n, 10.0),
        _ => panic!("Expected Number(10)"),
    }
}

// ============================================================================
// List Utilities Tests
// ============================================================================

#[test]
fn test_reverse() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(reverse '(1 2 3))", env.clone(), &mut macro_reg).unwrap();

    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 3);
            match &items[0] {
                value::Value::Number(n) => assert_eq!(*n, 3.0),
                _ => panic!("Expected Number"),
            }
            match &items[2] {
                value::Value::Number(n) => assert_eq!(*n, 1.0),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_append() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(append '(1 2) '(3 4))", env.clone(), &mut macro_reg).unwrap();

    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 4);
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_member() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(member 2 '(1 2 3))", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Bool(true)));

    let result = eval_code("(member 5 '(1 2 3))", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Bool(false)));
}

#[test]
fn test_nth() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(nth 0 '(10 20 30))", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 10.0),
        _ => panic!("Expected Number(10)"),
    }

    let result = eval_code("(nth 2 '(10 20 30))", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 30.0),
        _ => panic!("Expected Number(30)"),
    }
}

#[test]
fn test_last() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(last '(1 2 3))", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 3.0),
        _ => panic!("Expected Number(3)"),
    }
}

#[test]
fn test_take() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(take 2 '(1 2 3 4))", env.clone(), &mut macro_reg).unwrap();

    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 2);
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_drop() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(drop 2 '(1 2 3 4))", env.clone(), &mut macro_reg).unwrap();

    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 2);
            match &items[0] {
                value::Value::Number(n) => assert_eq!(*n, 3.0),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected List"),
    }
}

// ============================================================================
// Predicate Functions Tests
// ============================================================================

#[test]
fn test_all() {
    let (env, mut macro_reg) = setup();

    let result = eval_code(
        "(all (lambda (x) (> x 0)) '(1 2 3))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();
    assert!(matches!(result, value::Value::Bool(true)));

    let result = eval_code(
        "(all (lambda (x) (> x 2)) '(1 2 3))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();
    assert!(matches!(result, value::Value::Bool(false)));
}

#[test]
fn test_any() {
    let (env, mut macro_reg) = setup();

    let result = eval_code(
        "(any (lambda (x) (> x 2)) '(1 2 3))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();
    assert!(matches!(result, value::Value::Bool(true)));

    let result = eval_code(
        "(any (lambda (x) (> x 5)) '(1 2 3))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();
    assert!(matches!(result, value::Value::Bool(false)));
}

#[test]
fn test_count() {
    let (env, mut macro_reg) = setup();

    let result = eval_code(
        "(count (lambda (x) (> x 2)) '(1 2 3 4 5))",
        env.clone(),
        &mut macro_reg,
    )
    .unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 3.0),
        _ => panic!("Expected Number(3)"),
    }
}

// ============================================================================
// Range Tests
// ============================================================================

#[test]
fn test_range() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(range 0 5)", env.clone(), &mut macro_reg).unwrap();

    match result {
        value::Value::List(items) => {
            assert_eq!(items.len(), 5);
            match &items[0] {
                value::Value::Number(n) => assert_eq!(*n, 0.0),
                _ => panic!("Expected Number"),
            }
            match &items[4] {
                value::Value::Number(n) => assert_eq!(*n, 4.0),
                _ => panic!("Expected Number"),
            }
        }
        _ => panic!("Expected List"),
    }
}

// ============================================================================
// Math Utilities Tests
// ============================================================================

#[test]
fn test_abs() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(abs -5)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 5.0),
        _ => panic!("Expected Number(5)"),
    }

    let result = eval_code("(abs 5)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 5.0),
        _ => panic!("Expected Number(5)"),
    }
}

#[test]
fn test_min_max() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(min 3 5)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 3.0),
        _ => panic!("Expected Number(3)"),
    }

    let result = eval_code("(max 3 5)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 5.0),
        _ => panic!("Expected Number(5)"),
    }
}

#[test]
fn test_square_cube() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(square 5)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 25.0),
        _ => panic!("Expected Number(25)"),
    }

    let result = eval_code("(cube 3)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 27.0),
        _ => panic!("Expected Number(27)"),
    }
}

#[test]
fn test_even_odd() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(even? 4)", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Bool(true)));

    let result = eval_code("(odd? 3)", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Bool(true)));

    let result = eval_code("(even? 3)", env.clone(), &mut macro_reg).unwrap();
    assert!(matches!(result, value::Value::Bool(false)));
}

#[test]
fn test_sum_product() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(sum '(1 2 3 4))", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 10.0),
        _ => panic!("Expected Number(10)"),
    }

    let result = eval_code("(product '(1 2 3 4))", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 24.0),
        _ => panic!("Expected Number(24)"),
    }
}

#[test]
fn test_factorial() {
    let (env, mut macro_reg) = setup();

    let result = eval_code("(factorial 5)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 120.0, "Got {} instead of 120", n),
        _ => panic!("Expected Number(120), got {:?}", result),
    }

    let result = eval_code("(factorial 0)", env.clone(), &mut macro_reg).unwrap();
    match result {
        value::Value::Number(n) => assert_eq!(n, 1.0, "Got {} instead of 1", n),
        _ => panic!("Expected Number(1), got {:?}", result),
    }
}

// ============================================================================
// Higher-Order Function Composition Tests
// ============================================================================

#[test]
fn test_compose() {
    let (env, mut macro_reg) = setup();

    // Set up functions and test composition
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
        _ => panic!("Expected Number(12)"),
    }
}

// ABOUTME: Parser module for parsing Lisp expressions using nom combinators

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while, take_while1},
    character::complete::{char, digit1, multispace1, none_of, one_of},
    combinator::{not, opt, peek, recognize, value},
    multi::many0,
    IResult, Parser,
};

use crate::value::Value;
use std::cell::RefCell;

// ============================================================================
// Thread-Local Doc Comment Storage
// ============================================================================

thread_local! {
    /// Holds doc comments (;;;) that precede a top-level expression
    static PENDING_DOCS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Store doc comments to be attached to the next defined function
pub fn set_pending_docs(docs: Vec<String>) {
    PENDING_DOCS.with(|d| *d.borrow_mut() = docs);
}

/// Retrieve and clear pending doc comments
pub fn take_pending_docs() -> Vec<String> {
    PENDING_DOCS.with(|d| std::mem::take(&mut *d.borrow_mut()))
}

// ============================================================================
// Comment Parsers
// ============================================================================

/// Parse a documentation comment (line starting with ;;;)
fn parse_doc_comment(input: &str) -> IResult<&str, String> {
    let (input, _) = tag(";;;")(input)?;
    let (input, text) = take_while(|c| c != '\n')(input)?;
    Ok((input, text.trim().to_string()))
}

/// Parse a regular comment (line starting with ;, but not ;; or ;;;)
fn parse_regular_comment(input: &str) -> IResult<&str, ()> {
    let (input, _) = char(';')(input)?;
    // Make sure it's not ;; (look ahead without consuming)
    let (input, _) = not(peek(char(';'))).parse(input)?;
    let (input, _) = take_while(|c| c != '\n')(input)?;
    Ok((input, ()))
}

/// Parse a double semicolon comment (;;, but not ;;;)
fn parse_double_comment(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag(";;")(input)?;
    // Make sure it's not ;;; (look ahead without consuming)
    let (input, _) = not(peek(char(';'))).parse(input)?;
    let (input, _) = take_while(|c| c != '\n')(input)?;
    Ok((input, ()))
}

/// Skip whitespace and comments
fn ws_and_comments(input: &str) -> IResult<&str, ()> {
    many0(alt((
        value((), multispace1),
        parse_double_comment,
        parse_regular_comment,
        value((), parse_doc_comment.map(|_| ())), // Doc comments are skipped here
    )))
    .map(|_| ())
    .parse(input)
}

/// Skip whitespace and regular comments, but collect doc comments
fn ws_and_collect_docs(input: &str) -> IResult<&str, Vec<String>> {
    let mut docs = Vec::new();
    let mut input = input;

    loop {
        let start = input;

        // Try whitespace
        if let Ok((rest, _)) = multispace1::<_, nom::error::Error<_>>(input) {
            input = rest;
            continue;
        }

        // Try doc comment (;;;)
        if let Ok((rest, doc)) = parse_doc_comment(input) {
            docs.push(doc);
            input = rest;
            // Skip trailing newline after doc comment
            if let Ok((rest, _)) = char::<_, nom::error::Error<_>>('\n')(input) {
                input = rest;
            }
            continue;
        }

        // Try double semicolon comment (;;) - discard
        if let Ok((rest, _)) = parse_double_comment(input) {
            input = rest;
            if let Ok((rest, _)) = char::<_, nom::error::Error<_>>('\n')(input) {
                input = rest;
            }
            continue;
        }

        // Try regular comment (;) - discard
        if let Ok((rest, _)) = parse_regular_comment(input) {
            input = rest;
            if let Ok((rest, _)) = char::<_, nom::error::Error<_>>('\n')(input) {
                input = rest;
            }
            continue;
        }

        // No more whitespace or comments
        if start == input {
            break;
        }
    }

    Ok((input, docs))
}

/// Parse a number (integer or floating point)
/// Handles: 42, -42, 3.14, -3.14, .5, -.5
fn parse_number(input: &str) -> IResult<&str, Value> {
    recognize((
        opt(char('-')),
        alt((
            // Handle numbers starting with digit: 123, 123.456
            recognize((digit1, opt((char('.'), opt(digit1))))),
            // Handle numbers starting with decimal: .5, .123
            recognize((char('.'), digit1)),
        )),
    ))
    .map(|num_str: &str| {
        let num: f64 = num_str.parse().expect("Failed to parse number");
        Value::Number(num)
    })
    .parse(input)
}

/// Parse a boolean (#t or #f)
fn parse_bool(input: &str) -> IResult<&str, Value> {
    alt((
        value(Value::Bool(true), tag("#t")),
        value(Value::Bool(false), tag("#f")),
    ))
    .parse(input)
}

/// Parse a symbol
/// Starts with letter or special chars: +-*/%<>=!?
/// Followed by alphanumeric, -, or _
fn parse_symbol(input: &str) -> IResult<&str, Value> {
    let (input, first) =
        one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ+-*/%<>=!?")(input)?;
    let (input, rest) = take_while1::<_, _, nom::error::Error<_>>(|c: char| {
        c.is_alphanumeric()
            || c == '-'
            || c == '_'
            || c == '?'
            || c == '!'
            || c == '<'
            || c == '>'
            || c == '='
            || c == '+'
            || c == '*'
            || c == '/'
            || c == '%'
    })(input)
    .unwrap_or((input, ""));

    let mut symbol = String::new();
    symbol.push(first);
    symbol.push_str(rest);

    Ok((input, Value::Symbol(symbol)))
}

/// Parse a string with escape sequences
/// Handles: "hello world", with escapes: \", \\, \n, \t
fn parse_string(input: &str) -> IResult<&str, Value> {
    let (input, _) = char('"')(input)?;

    // Handle empty strings
    if let Ok((input, _)) = char::<_, nom::error::Error<_>>('"')(input) {
        return Ok((input, Value::String(String::new())));
    }

    let (input, content) = escaped(none_of("\"\\"), '\\', one_of("\"\\nt"))(input)?;
    let (input, _) = char('"')(input)?;

    // Process escape sequences
    let mut result = String::new();
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    _ => {
                        result.push('\\');
                        result.push(next);
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    Ok((input, Value::String(result)))
}

/// Parse a quoted expression: 'expr -> (quote expr)
fn parse_quote(input: &str) -> IResult<&str, Value> {
    let (input, _) = char('\'')(input)?;
    let (input, expr) = parse_expr(input)?;
    Ok((
        input,
        Value::List(vec![Value::Symbol("quote".to_string()), expr]),
    ))
}

/// Parse a quasiquoted expression: `expr -> (quasiquote expr)
fn parse_quasiquote(input: &str) -> IResult<&str, Value> {
    let (input, _) = char('`')(input)?;
    let (input, expr) = parse_expr(input)?;
    Ok((
        input,
        Value::List(vec![Value::Symbol("quasiquote".to_string()), expr]),
    ))
}

/// Parse an unquote expression: ,expr -> (unquote expr)
/// or unquote-splicing: ,@expr -> (unquote-splicing expr)
fn parse_unquote(input: &str) -> IResult<&str, Value> {
    let (input, _) = char(',')(input)?;

    // Check for ,@ (unquote-splicing)
    if let Ok((input, _)) = char::<_, nom::error::Error<_>>('@')(input) {
        let (input, expr) = parse_expr(input)?;
        Ok((
            input,
            Value::List(vec![Value::Symbol("unquote-splicing".to_string()), expr]),
        ))
    } else {
        // Just , (unquote)
        let (input, expr) = parse_expr(input)?;
        Ok((
            input,
            Value::List(vec![Value::Symbol("unquote".to_string()), expr]),
        ))
    }
}

/// Parse a list: (expr1 expr2 ...)
/// Empty list () becomes Value::Nil
fn parse_list(input: &str) -> IResult<&str, Value> {
    let (input, _) = char('(')(input)?;
    let (input, _) = ws_and_comments(input)?;

    let mut items = Vec::new();
    let mut remaining = input;

    loop {
        // Try to parse closing paren
        if let Ok((rest, _)) = char::<_, nom::error::Error<_>>(')')(remaining) {
            // Empty list is nil
            if items.is_empty() {
                return Ok((rest, Value::Nil));
            }
            return Ok((rest, Value::List(items)));
        }

        // Parse an expression
        let (rest, expr) = parse_expr(remaining)?;
        items.push(expr);

        // Skip whitespace and comments
        let (rest, _) = ws_and_comments(rest)?;
        remaining = rest;
    }
}

/// Main expression parser - tries all alternatives
fn parse_expr(input: &str) -> IResult<&str, Value> {
    let (input, _) = ws_and_comments(input)?;
    alt((
        parse_quote,
        parse_quasiquote,
        parse_unquote,
        parse_list,
        parse_bool,
        parse_number,
        parse_string,
        parse_symbol,
    ))
    .parse(input)
}

/// Public entry point for parsing
///
/// Collects any leading doc comments (;;;) and stores them in thread-local storage
/// so they can be attached to the next `define` expression.
pub fn parse(input: &str) -> Result<Value, String> {
    // First, collect any leading doc comments
    let (input_after_docs, docs) = ws_and_collect_docs(input).unwrap_or((input, Vec::new()));

    // Store doc comments for the next define
    if !docs.is_empty() {
        set_pending_docs(docs);
    }

    // Check if input is only whitespace/comments (nothing to parse)
    if input_after_docs.trim().is_empty() {
        // Return nil for comment-only input
        return Ok(Value::Nil);
    }

    // Parse the expression
    match parse_expr(input_after_docs) {
        Ok((rest, value)) => {
            // Check if there's unconsumed input (after skipping trailing whitespace)
            let (rest, _) = ws_and_comments(rest).unwrap_or((rest, ()));
            if !rest.is_empty() {
                Err(format!(
                    "Parse error: unexpected trailing input: '{}'",
                    rest
                ))
            } else {
                Ok(value)
            }
        }
        Err(e) => Err(format!("Parse error: {:?}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        // Integers
        assert!(matches!(parse("42"), Ok(Value::Number(n)) if n == 42.0));
        assert!(matches!(parse("-42"), Ok(Value::Number(n)) if n == -42.0));
        assert!(matches!(parse("0"), Ok(Value::Number(n)) if n == 0.0));

        // Floats
        assert!(matches!(parse("2.5"), Ok(Value::Number(n)) if (n - 2.5).abs() < 0.001));
        assert!(matches!(parse("-2.5"), Ok(Value::Number(n)) if (n + 2.5).abs() < 0.001));

        // Leading decimal point
        assert!(matches!(parse(".5"), Ok(Value::Number(n)) if n == 0.5));
        assert!(matches!(parse("-.5"), Ok(Value::Number(n)) if n == -0.5));

        // Trailing decimal point
        assert!(matches!(parse("42."), Ok(Value::Number(n)) if n == 42.0));
    }

    #[test]
    fn test_parse_bool() {
        assert!(matches!(parse("#t"), Ok(Value::Bool(true))));
        assert!(matches!(parse("#f"), Ok(Value::Bool(false))));
    }

    #[test]
    fn test_parse_symbol() {
        assert!(matches!(parse("x"), Ok(Value::Symbol(s)) if s == "x"));
        assert!(matches!(parse("foo"), Ok(Value::Symbol(s)) if s == "foo"));
        assert!(matches!(parse("foo-bar"), Ok(Value::Symbol(s)) if s == "foo-bar"));
        assert!(matches!(parse("foo_bar"), Ok(Value::Symbol(s)) if s == "foo_bar"));
        assert!(matches!(parse("foo?"), Ok(Value::Symbol(s)) if s == "foo?"));
        assert!(matches!(parse("foo!"), Ok(Value::Symbol(s)) if s == "foo!"));

        // Operators
        assert!(matches!(parse("+"), Ok(Value::Symbol(s)) if s == "+"));
        assert!(matches!(parse("-"), Ok(Value::Symbol(s)) if s == "-"));
        assert!(matches!(parse("*"), Ok(Value::Symbol(s)) if s == "*"));
        assert!(matches!(parse("/"), Ok(Value::Symbol(s)) if s == "/"));
        assert!(matches!(parse("<"), Ok(Value::Symbol(s)) if s == "<"));
        assert!(matches!(parse(">"), Ok(Value::Symbol(s)) if s == ">"));
        assert!(matches!(parse("="), Ok(Value::Symbol(s)) if s == "="));
        assert!(matches!(parse(">="), Ok(Value::Symbol(s)) if s == ">="));
    }

    #[test]
    fn test_parse_string() {
        assert!(matches!(parse(r#""hello""#), Ok(Value::String(s)) if s == "hello"));
        assert!(matches!(parse(r#""hello world""#), Ok(Value::String(s)) if s == "hello world"));
        assert!(matches!(parse(r#""""#), Ok(Value::String(s)) if s.is_empty()));

        // Escape sequences
        assert!(matches!(parse(r#""hello\nworld""#), Ok(Value::String(s)) if s == "hello\nworld"));
        assert!(matches!(parse(r#""hello\tworld""#), Ok(Value::String(s)) if s == "hello\tworld"));
        assert!(matches!(parse(r#""say \"hi\"""#), Ok(Value::String(s)) if s == r#"say "hi""#));
        assert!(matches!(parse(r#""back\\slash""#), Ok(Value::String(s)) if s == r"back\slash"));
    }

    #[test]
    fn test_parse_empty_list() {
        assert!(matches!(parse("()"), Ok(Value::Nil)));
        assert!(matches!(parse("(  )"), Ok(Value::Nil)));
        assert!(matches!(parse("(\n)"), Ok(Value::Nil)));
    }

    #[test]
    fn test_parse_simple_list() {
        match parse("(1 2 3)") {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 3);
                assert!(matches!(items[0], Value::Number(n) if n == 1.0));
                assert!(matches!(items[1], Value::Number(n) if n == 2.0));
                assert!(matches!(items[2], Value::Number(n) if n == 3.0));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_nested_list() {
        match parse("(1 (2 3) 4)") {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 3);
                assert!(matches!(items[0], Value::Number(n) if n == 1.0));

                match &items[1] {
                    Value::List(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert!(matches!(inner[0], Value::Number(n) if n == 2.0));
                        assert!(matches!(inner[1], Value::Number(n) if n == 3.0));
                    }
                    _ => panic!("Expected nested list"),
                }

                assert!(matches!(items[2], Value::Number(n) if n == 4.0));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_quoted() {
        // 'x -> (quote x)
        match parse("'x") {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], Value::Symbol(s) if s == "quote"));
                assert!(matches!(&items[1], Value::Symbol(s) if s == "x"));
            }
            _ => panic!("Expected quoted expression"),
        }

        // '(1 2) -> (quote (1 2))
        match parse("'(1 2)") {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], Value::Symbol(s) if s == "quote"));
                match &items[1] {
                    Value::List(inner) => {
                        assert_eq!(inner.len(), 2);
                    }
                    _ => panic!("Expected list inside quote"),
                }
            }
            _ => panic!("Expected quoted list"),
        }
    }

    #[test]
    fn test_parse_quasiquote() {
        // `x -> (quasiquote x)
        match parse("`x") {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], Value::Symbol(s) if s == "quasiquote"));
                assert!(matches!(&items[1], Value::Symbol(s) if s == "x"));
            }
            _ => panic!("Expected quasiquoted expression"),
        }
    }

    #[test]
    fn test_parse_unquote() {
        // ,x -> (unquote x)
        match parse(",x") {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], Value::Symbol(s) if s == "unquote"));
                assert!(matches!(&items[1], Value::Symbol(s) if s == "x"));
            }
            _ => panic!("Expected unquoted expression"),
        }

        // ,@x -> (unquote-splicing x)
        match parse(",@x") {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], Value::Symbol(s) if s == "unquote-splicing"));
                assert!(matches!(&items[1], Value::Symbol(s) if s == "x"));
            }
            _ => panic!("Expected unquote-splicing expression"),
        }
    }

    #[test]
    fn test_parse_comments() {
        // Comment on its own
        assert!(matches!(parse("; this is a comment\n42"), Ok(Value::Number(n)) if n == 42.0));

        // Comment after expression
        match parse("(1 2 ; comment\n 3)") {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected list with comments"),
        }
    }

    #[test]
    fn test_parse_complex_expr() {
        // (define (square x) (* x x))
        match parse("(define (square x) (* x x))") {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 3);
                assert!(matches!(&items[0], Value::Symbol(s) if s == "define"));

                // (square x)
                match &items[1] {
                    Value::List(func_def) => {
                        assert_eq!(func_def.len(), 2);
                        assert!(matches!(&func_def[0], Value::Symbol(s) if s == "square"));
                        assert!(matches!(&func_def[1], Value::Symbol(s) if s == "x"));
                    }
                    _ => panic!("Expected function definition"),
                }

                // (* x x)
                match &items[2] {
                    Value::List(body) => {
                        assert_eq!(body.len(), 3);
                        assert!(matches!(&body[0], Value::Symbol(s) if s == "*"));
                        assert!(matches!(&body[1], Value::Symbol(s) if s == "x"));
                        assert!(matches!(&body[2], Value::Symbol(s) if s == "x"));
                    }
                    _ => panic!("Expected function body"),
                }
            }
            _ => panic!("Expected define expression"),
        }
    }

    #[test]
    fn test_parse_whitespace_handling() {
        // Leading/trailing whitespace
        assert!(matches!(parse("  42  "), Ok(Value::Number(n)) if n == 42.0));
        assert!(matches!(parse("\n42\n"), Ok(Value::Number(n)) if n == 42.0));
        assert!(matches!(parse("\t42\t"), Ok(Value::Number(n)) if n == 42.0));

        // Whitespace in lists
        match parse("(  1   2   3  )") {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_mixed_types() {
        // (+ 1 2.5 (* 3 4))
        match parse("(+ 1 2.5 (* 3 4))") {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 4);
                assert!(matches!(&items[0], Value::Symbol(s) if s == "+"));
                assert!(matches!(&items[1], Value::Number(n) if *n == 1.0));
                assert!(matches!(&items[2], Value::Number(n) if (*n - 2.5).abs() < 0.001));

                match &items[3] {
                    Value::List(inner) => {
                        assert_eq!(inner.len(), 3);
                        assert!(matches!(&inner[0], Value::Symbol(s) if s == "*"));
                    }
                    _ => panic!("Expected nested list"),
                }
            }
            _ => panic!("Expected expression"),
        }
    }

    #[test]
    fn test_parse_error_unclosed_list() {
        assert!(parse("(1 2").is_err());
    }

    #[test]
    fn test_parse_error_unexpected_closing() {
        assert!(parse(")").is_err());
    }

    #[test]
    fn test_parse_multiple_top_level() {
        // Should error on multiple top-level expressions
        assert!(parse("1 2").is_err());
    }
}

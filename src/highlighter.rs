// ABOUTME: Syntax highlighter for REPL with color support
// Implements rustyline's Highlighter trait to provide ANSI color codes
// for Lisp syntax elements while preserving display width
// Also provides output highlighting for pretty-printed values

use crate::value::Value;
use rustyline::completion::Completer;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Helper;
use std::borrow::Cow;
use std::collections::HashSet;

// ANSI color codes (using 3-bit/4-bit colors for maximum terminal compatibility)
const COLOR_RESET: &str = "\x1b[0m";
const COLOR_PARENS: &str = "\x1b[1;34m"; // Bold blue
const COLOR_SPECIAL_FORM: &str = "\x1b[1;35m"; // Bold magenta
const COLOR_BUILTIN: &str = "\x1b[36m"; // Cyan
const COLOR_NUMBER: &str = "\x1b[33m"; // Yellow
const COLOR_STRING: &str = "\x1b[32m"; // Green
const COLOR_BOOLEAN: &str = "\x1b[33m"; // Yellow
const COLOR_COMMENT: &str = "\x1b[90m"; // Bright black (gray)
const COLOR_QUOTE: &str = "\x1b[1;33m"; // Bold yellow

/// Main highlighter helper for Lisp REPL
/// Provides syntax-aware color highlighting for Lisp syntax
pub struct LispHelper;

impl LispHelper {
    pub fn new() -> Self {
        LispHelper
    }
}

impl Default for LispHelper {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the required rustyline traits
impl Helper for LispHelper {}

impl Completer for LispHelper {
    type Candidate = String;
}

impl Hinter for LispHelper {
    type Hint = String;
}

impl Validator for LispHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        // Check for obvious syntax errors (e.g., stray closing parens) - fail fast
        if has_syntax_error(input) {
            return Ok(ValidationResult::Invalid(Some(
                "Syntax error: unmatched closing parenthesis".into(),
            )));
        }

        // Check if input is incomplete using the same logic as main.rs
        if is_input_incomplete(input) {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

impl Highlighter for LispHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Build the special forms and built-in sets
        let special_forms = get_special_forms();
        let builtins = get_builtins();
        let stdlib_funcs = get_stdlib_functions();

        // Tokenize and colorize
        let highlighted = highlight_line(line, &special_forms, &builtins, &stdlib_funcs);

        if highlighted == line {
            Cow::Borrowed(line)
        } else {
            Cow::Owned(highlighted)
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        true // Always trigger re-highlighting on character input or cursor movement
    }
}

impl LispHelper {
    /// Highlight a Lisp value for output display
    /// Applies color codes to make values more readable in the REPL
    pub fn highlight_output(value: &Value) -> String {
        highlight_value(value)
    }
}

/// Tokenize a line and apply syntax highlighting
fn highlight_line(
    line: &str,
    special_forms: &HashSet<&'static str>,
    builtins: &HashSet<&'static str>,
    stdlib_funcs: &HashSet<&'static str>,
) -> String {
    let mut result = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            // Comments: everything from ; to end of line
            ';' => {
                result.push_str(COLOR_COMMENT);
                while i < chars.len() && chars[i] != '\n' {
                    result.push(chars[i]);
                    i += 1;
                }
                result.push_str(COLOR_RESET);
            }

            // Strings: preserve exact content but colorize
            '"' => {
                result.push_str(COLOR_STRING);
                result.push('"');
                i += 1;

                // Read string content with escape handling
                let mut found_close = false;
                while i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        result.push(chars[i]);
                        result.push(chars[i + 1]);
                        i += 2;
                    } else if chars[i] == '"' {
                        result.push('"');
                        i += 1;
                        found_close = true;
                        break;
                    } else {
                        result.push(chars[i]);
                        i += 1;
                    }
                }

                result.push_str(COLOR_RESET);
                if !found_close && i > 0 {
                    // Unclosed string - let it still be colored to end of line
                    while i < chars.len() && chars[i] != '\n' {
                        result.push(chars[i]);
                        i += 1;
                    }
                }
            }

            // Numbers: handle all numeric formats
            '0'..='9' | '.' => {
                let old_i = i;
                if chars[i] == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                    // .5 style number
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                } else if chars[i].is_ascii_digit() {
                    // Regular number
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    if i < chars.len()
                        && chars[i] == '.'
                        && i + 1 < chars.len()
                        && chars[i + 1].is_ascii_digit()
                    {
                        i += 1;
                        while i < chars.len() && chars[i].is_ascii_digit() {
                            i += 1;
                        }
                    }
                } else {
                    // Just a dot, which might be part of a symbol
                    result.push(chars[i]);
                    i += 1;
                    continue;
                }

                let num_str: String = chars[old_i..i].iter().collect();
                result.push_str(COLOR_NUMBER);
                result.push_str(&num_str);
                result.push_str(COLOR_RESET);
            }

            // Signed numbers or symbols starting with +/-
            '+' | '-' => {
                // Only treat as number start if immediately followed by digit or dot+digit
                if i + 1 < chars.len()
                    && (chars[i + 1].is_ascii_digit()
                        || (chars[i + 1] == '.'
                            && i + 2 < chars.len()
                            && chars[i + 2].is_ascii_digit()))
                {
                    let old_i = i;
                    i += 1;

                    if chars[old_i + 1] == '.' {
                        // -.5 or +.5
                        i += 1;
                        while i < chars.len() && chars[i].is_ascii_digit() {
                            i += 1;
                        }
                    } else {
                        // -123 or +456
                        while i < chars.len() && chars[i].is_ascii_digit() {
                            i += 1;
                        }
                        if i < chars.len()
                            && chars[i] == '.'
                            && i + 1 < chars.len()
                            && chars[i + 1].is_ascii_digit()
                        {
                            i += 1;
                            while i < chars.len() && chars[i].is_ascii_digit() {
                                i += 1;
                            }
                        }
                    }

                    let num_str: String = chars[old_i..i].iter().collect();
                    result.push_str(COLOR_NUMBER);
                    result.push_str(&num_str);
                    result.push_str(COLOR_RESET);
                } else {
                    // Just a symbol (+, -, or symbol starting with them)
                    let start = i;
                    while i < chars.len()
                        && !chars[i].is_whitespace()
                        && chars[i] != '('
                        && chars[i] != ')'
                        && chars[i] != '['
                        && chars[i] != ']'
                        && chars[i] != '{'
                        && chars[i] != '}'
                        && chars[i] != '"'
                        && chars[i] != ';'
                        && chars[i] != '\''
                        && chars[i] != '`'
                        && chars[i] != ','
                    {
                        i += 1;
                    }

                    let symbol: String = chars[start..i].iter().collect();
                    let builtins = get_builtins();
                    let stdlib_funcs = get_stdlib_functions();

                    if builtins.contains(symbol.as_str()) || stdlib_funcs.contains(symbol.as_str())
                    {
                        result.push_str(COLOR_BUILTIN);
                        result.push_str(&symbol);
                        result.push_str(COLOR_RESET);
                    } else {
                        result.push_str(&symbol);
                    }
                }
            }

            // Booleans and special values
            '#' => {
                if i + 1 < chars.len() && (chars[i + 1] == 't' || chars[i + 1] == 'f') {
                    if i + 2 < chars.len()
                        && (chars[i + 2].is_alphanumeric()
                            || chars[i + 2] == '_'
                            || chars[i + 2] == '-')
                    {
                        // Not a boolean, it's a symbol that starts with #
                        result.push_str(COLOR_BUILTIN);
                        result.push(chars[i]);
                        result.push(chars[i + 1]);
                        i += 2;
                        result.push_str(COLOR_RESET);
                    } else {
                        // It's a boolean
                        result.push_str(COLOR_BOOLEAN);
                        result.push(chars[i]);
                        result.push(chars[i + 1]);
                        i += 2;
                        result.push_str(COLOR_RESET);
                    }
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }

            // Quote-like special characters
            '\'' | '`' => {
                result.push_str(COLOR_QUOTE);
                result.push(chars[i]);
                i += 1;
                result.push_str(COLOR_RESET);
            }

            // Unquote
            ',' => {
                if i + 1 < chars.len() && chars[i + 1] == '@' {
                    result.push_str(COLOR_QUOTE);
                    result.push(',');
                    result.push('@');
                    i += 2;
                    result.push_str(COLOR_RESET);
                } else {
                    result.push_str(COLOR_QUOTE);
                    result.push(',');
                    i += 1;
                    result.push_str(COLOR_RESET);
                }
            }

            // Parentheses and brackets
            '(' | ')' | '[' | ']' | '{' | '}' => {
                result.push_str(COLOR_PARENS);
                result.push(chars[i]);
                i += 1;
                result.push_str(COLOR_RESET);
            }

            // Whitespace
            ' ' | '\t' | '\n' | '\r' => {
                result.push(chars[i]);
                i += 1;
            }

            // Symbols (variables, function names, etc.)
            _ => {
                let start = i;
                while i < chars.len()
                    && !chars[i].is_whitespace()
                    && chars[i] != '('
                    && chars[i] != ')'
                    && chars[i] != '['
                    && chars[i] != ']'
                    && chars[i] != '{'
                    && chars[i] != '}'
                    && chars[i] != '"'
                    && chars[i] != ';'
                    && chars[i] != '\''
                    && chars[i] != '`'
                    && chars[i] != ','
                {
                    i += 1;
                }

                let symbol: String = chars[start..i].iter().collect();

                // Classify the symbol
                if special_forms.contains(symbol.as_str()) {
                    result.push_str(COLOR_SPECIAL_FORM);
                    result.push_str(&symbol);
                    result.push_str(COLOR_RESET);
                } else if builtins.contains(symbol.as_str())
                    || stdlib_funcs.contains(symbol.as_str())
                {
                    result.push_str(COLOR_BUILTIN);
                    result.push_str(&symbol);
                    result.push_str(COLOR_RESET);
                } else {
                    // Regular symbol
                    result.push_str(&symbol);
                }
            }
        }
    }

    result
}

/// Check if input is incomplete and needs more lines
fn is_input_incomplete(input: &str) -> bool {
    let trimmed = input.trim();

    // If input starts with ;;; (doc comment), check for following expression
    if trimmed.starts_with(";;;") {
        let mut has_expression = false;
        for line in input.lines() {
            let line_trimmed = line.trim();
            if !line_trimmed.is_empty()
                && !line_trimmed.starts_with(";")
                && !line_trimmed.chars().all(char::is_whitespace)
            {
                has_expression = true;
                break;
            }
        }
        if !has_expression {
            return true;
        }
    }

    // Check for balanced parentheses and quotes
    let mut paren_depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for ch in trimmed.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '(' if !in_string => paren_depth += 1,
            ')' if !in_string => paren_depth -= 1,
            _ => {}
        }
    }

    paren_depth > 0 || in_string
}

/// Check if input has obvious syntax errors (e.g., stray closing parens)
/// Returns true if expression is malformed and should fail immediately
pub fn has_syntax_error(input: &str) -> bool {
    let trimmed = input.trim();

    // Skip comment-only input
    if trimmed.is_empty() || trimmed.starts_with(";") {
        return false;
    }

    // Count parentheses balance
    let mut paren_depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for ch in trimmed.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '(' if !in_string => paren_depth += 1,
            ')' if !in_string => {
                paren_depth -= 1;
                // More closing parens than opening = syntax error
                if paren_depth < 0 {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

/// Get all special forms (keywords that have special evaluation semantics)
fn get_special_forms() -> HashSet<&'static str> {
    [
        "define",
        "lambda",
        "if",
        "begin",
        "let",
        "letrec",
        "quote",
        "quasiquote",
        "unquote",
        "unquote-splicing",
        "defmacro",
    ]
    .iter()
    .copied()
    .collect()
}

/// Get all built-in functions
fn get_builtins() -> HashSet<&'static str> {
    [
        // Arithmetic
        "+",
        "-",
        "*",
        "/",
        "%",
        // Comparison
        "=",
        "<",
        ">",
        "<=",
        ">=",
        // Logic
        "and",
        "or",
        "not",
        // List operations
        "cons",
        "car",
        "cdr",
        "list",
        "length",
        "empty?",
        // Type predicates
        "bool?",
        "channel?",
        "keyword?",
        "list?",
        "map?",
        "nil?",
        "number?",
        "string?",
        "symbol?",
        // Map operations
        "map-new",
        // Console I/O
        "print",
        "println",
        // Filesystem
        "file-exists?",
        "file-size",
        "file-stat",
        "list-files",
        "read-file",
        "write-file",
        // Network
        "http-request",
        // Concurrency
        "channel-close",
        "channel-recv",
        "channel-send",
        "make-channel",
        "spawn",
        "spawn-link",
        // Error handling
        "error",
        "error?",
        "error-msg",
        // Help
        "help",
        "doc",
    ]
    .iter()
    .copied()
    .collect()
}

/// Get all stdlib functions that should be highlighted
fn get_stdlib_functions() -> HashSet<&'static str> {
    [
        // Higher-order functions
        "map",
        "filter",
        "reduce",
        "compose",
        "partial",
        // List utilities
        "reverse",
        "append",
        "member",
        "nth",
        "last",
        "take",
        "drop",
        "zip",
        // Predicates
        "all",
        "any",
        "count",
        "even?",
        "odd?",
        // Math
        "abs",
        "min",
        "max",
        "square",
        "cube",
        "sum",
        "product",
        "factorial",
        // Other
        "range",
    ]
    .iter()
    .copied()
    .collect()
}

/// Recursively highlight a Value for output display
fn highlight_value(value: &Value) -> String {
    match value {
        Value::Number(n) => {
            let num_str = if n.fract() == 0.0 && n.is_finite() {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            };
            format!("{}{}{}", COLOR_NUMBER, num_str, COLOR_RESET)
        }
        Value::Bool(b) => {
            let bool_str = if *b { "#t" } else { "#f" };
            format!("{}{}{}", COLOR_BOOLEAN, bool_str, COLOR_RESET)
        }
        Value::String(s) => {
            format!("{}\"{}\"{}", COLOR_STRING, s, COLOR_RESET)
        }
        Value::Symbol(s) => {
            // Symbols are normally displayed uncolored unless they're special
            s.clone()
        }
        Value::Keyword(k) => {
            // Keywords displayed with : prefix
            format!("{}:{}{}", COLOR_STRING, k, COLOR_RESET)
        }
        Value::List(items) => {
            let mut result = format!("{}({}", COLOR_PARENS, COLOR_RESET);
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    result.push(' ');
                }
                result.push_str(&highlight_value(item));
            }
            result.push_str(&format!("{}){}", COLOR_PARENS, COLOR_RESET));
            result
        }
        Value::Map(map) => {
            let mut result = format!("{}{{{}", COLOR_PARENS, COLOR_RESET);
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(k, _)| *k);
            for (i, (key, value)) in entries.iter().enumerate() {
                if i > 0 {
                    result.push(' ');
                }
                result.push_str(&format!("{}:{}{} ", COLOR_STRING, key, COLOR_RESET));
                result.push_str(&highlight_value(value));
            }
            result.push_str(&format!("{}}}{}", COLOR_PARENS, COLOR_RESET));
            result
        }
        Value::Lambda { .. } => {
            format!("{}#<lambda>{}", COLOR_BUILTIN, COLOR_RESET)
        }
        Value::Macro { .. } => {
            format!("{}#<macro>{}", COLOR_BUILTIN, COLOR_RESET)
        }
        Value::BuiltIn(_) => {
            format!("{}#<builtin>{}", COLOR_BUILTIN, COLOR_RESET)
        }
        Value::Channel { .. } => {
            format!("{}#<channel>{}", COLOR_BUILTIN, COLOR_RESET)
        }
        Value::Error(msg) => {
            format!("{}#<error: {}>{}", COLOR_SPECIAL_FORM, msg, COLOR_RESET)
        }
        Value::Nil => {
            format!("{}nil{}", COLOR_BUILTIN, COLOR_RESET)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_highlighting() {
        let special_forms = get_special_forms();
        let builtins = get_builtins();
        let stdlib = get_stdlib_functions();

        let highlighted = highlight_line("42", &special_forms, &builtins, &stdlib);
        assert!(highlighted.contains(COLOR_NUMBER));
    }

    #[test]
    fn test_string_highlighting() {
        let special_forms = get_special_forms();
        let builtins = get_builtins();
        let stdlib = get_stdlib_functions();

        let highlighted = highlight_line("\"hello\"", &special_forms, &builtins, &stdlib);
        assert!(highlighted.contains(COLOR_STRING));
    }

    #[test]
    fn test_comment_highlighting() {
        let special_forms = get_special_forms();
        let builtins = get_builtins();
        let stdlib = get_stdlib_functions();

        let highlighted = highlight_line("; this is a comment", &special_forms, &builtins, &stdlib);
        assert!(highlighted.contains(COLOR_COMMENT));
    }

    #[test]
    fn test_special_form_highlighting() {
        let special_forms = get_special_forms();
        let builtins = get_builtins();
        let stdlib = get_stdlib_functions();

        let highlighted = highlight_line("(define x 5)", &special_forms, &builtins, &stdlib);
        assert!(highlighted.contains(COLOR_SPECIAL_FORM));
        assert!(highlighted.contains(COLOR_PARENS));
    }

    #[test]
    fn test_builtin_function_highlighting() {
        let special_forms = get_special_forms();
        let builtins = get_builtins();
        let stdlib = get_stdlib_functions();

        let highlighted = highlight_line("(+ 1 2)", &special_forms, &builtins, &stdlib);
        assert!(highlighted.contains(COLOR_BUILTIN));
        assert!(highlighted.contains(COLOR_PARENS));
    }

    #[test]
    fn test_boolean_highlighting() {
        let special_forms = get_special_forms();
        let builtins = get_builtins();
        let stdlib = get_stdlib_functions();

        let highlighted = highlight_line("#t #f", &special_forms, &builtins, &stdlib);
        assert!(highlighted.contains(COLOR_BOOLEAN));
    }

    #[test]
    fn test_quote_highlighting() {
        let special_forms = get_special_forms();
        let builtins = get_builtins();
        let stdlib = get_stdlib_functions();

        let highlighted = highlight_line("'(1 2 3)", &special_forms, &builtins, &stdlib);
        assert!(highlighted.contains(COLOR_QUOTE));
    }

    #[test]
    fn test_stdlib_function_highlighting() {
        let special_forms = get_special_forms();
        let builtins = get_builtins();
        let stdlib = get_stdlib_functions();

        let highlighted = highlight_line("(map inc lst)", &special_forms, &builtins, &stdlib);
        assert!(highlighted.contains(COLOR_BUILTIN)); // 'map' is in stdlib
    }

    #[test]
    fn test_output_number_highlighting() {
        let value = Value::Number(42.0);
        let highlighted = LispHelper::highlight_output(&value);
        assert!(highlighted.contains(COLOR_NUMBER));
        assert!(highlighted.contains("42"));
    }

    #[test]
    fn test_output_bool_highlighting() {
        let value_true = Value::Bool(true);
        let highlighted_true = LispHelper::highlight_output(&value_true);
        assert!(highlighted_true.contains(COLOR_BOOLEAN));
        assert!(highlighted_true.contains("#t"));

        let value_false = Value::Bool(false);
        let highlighted_false = LispHelper::highlight_output(&value_false);
        assert!(highlighted_false.contains(COLOR_BOOLEAN));
        assert!(highlighted_false.contains("#f"));
    }

    #[test]
    fn test_output_string_highlighting() {
        let value = Value::String("hello".to_string());
        let highlighted = LispHelper::highlight_output(&value);
        assert!(highlighted.contains(COLOR_STRING));
        assert!(highlighted.contains("\"hello\""));
    }

    #[test]
    fn test_output_list_highlighting() {
        let value = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let highlighted = LispHelper::highlight_output(&value);
        assert!(highlighted.contains(COLOR_PARENS));
        assert!(highlighted.contains(COLOR_NUMBER));
    }

    #[test]
    fn test_output_nil_highlighting() {
        let value = Value::Nil;
        let highlighted = LispHelper::highlight_output(&value);
        assert!(highlighted.contains("nil"));
    }

    #[test]
    fn test_output_symbol_highlighting() {
        let value = Value::Symbol("my-var".to_string());
        let highlighted = LispHelper::highlight_output(&value);
        assert!(highlighted.contains("my-var"));
    }

    #[test]
    fn test_syntax_error_stray_closing_paren() {
        assert!(has_syntax_error(")"));
        assert!(has_syntax_error("(+ 1 2))"));
        assert!(has_syntax_error("(* 3 4)))"));
    }

    #[test]
    fn test_syntax_error_balanced_parens() {
        assert!(!has_syntax_error("(+ 1 2)"));
        assert!(!has_syntax_error("(define x 5)"));
        assert!(!has_syntax_error("((lambda (x) x) 5)"));
    }

    #[test]
    fn test_syntax_error_incomplete_expression() {
        // Incomplete is not an error, just incomplete
        assert!(!has_syntax_error("(+ 1"));
        assert!(!has_syntax_error("(define (f x)"));
    }

    #[test]
    fn test_syntax_error_comment_only() {
        // Comments should not trigger error
        assert!(!has_syntax_error("; comment"));
        assert!(!has_syntax_error(";;; doc comment"));
    }

    #[test]
    fn test_syntax_error_string_with_parens() {
        // Parens inside strings should not affect balance check
        assert!(!has_syntax_error("(string-append \"(\" \")\")"));
        // Unclosed string is incomplete, not a syntax error
        assert!(!has_syntax_error("\"unclosed string with )"));
    }

    #[test]
    fn test_incomplete_input_detection() {
        // Incomplete expressions should return true
        assert!(is_input_incomplete("(+ 1 2"));
        assert!(is_input_incomplete("(define x"));
        assert!(is_input_incomplete("\"unclosed string"));
    }

    #[test]
    fn test_incomplete_input_complete_expression() {
        // Complete expressions should return false
        assert!(!is_input_incomplete("(+ 1 2)"));
        assert!(!is_input_incomplete("42"));
        assert!(!is_input_incomplete("\"hello\""));
    }

    #[test]
    fn test_incomplete_input_doc_comment() {
        // Doc comment without expression
        assert!(is_input_incomplete(";;; This is a doc comment"));
        // Doc comment with expression
        assert!(!is_input_incomplete(";;; Doc\n(define x 5)"));
    }
}

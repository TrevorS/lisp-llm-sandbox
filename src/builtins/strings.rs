//! String manipulation operations
//!
//! Comprehensive string manipulation functions including:
//! - Splitting and joining: string-split, string-join, string-append
//! - Extraction: substring, string-trim
//! - Transformation: string-upper, string-lower, string-replace
//! - Predicates: string-contains?, string-starts-with?, string-ends-with?, string-empty?
//! - Conversion: string->number, number->string, string->list, list->string
//! - Measurement: string-length

use crate::error::EvalError;
use crate::value::Value;
use lisp_macros::builtin;

#[builtin(name = "string-split", category = "String manipulation", related(string-join, substring))]
/// Split a string by delimiter into a list of strings.
///
/// # Examples
///
/// ```lisp
/// (string-split "a,b,c" ",") => ("a" "b" "c")
/// ```
///
/// # See Also
///
/// string-join, substring
pub fn builtin_string_split(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let delimiter = match &args[1] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let parts: Vec<Value> = string
        .split(delimiter.as_str())
        .map(|s| Value::String(s.to_string()))
        .collect();

    Ok(Value::List(parts))
}

#[builtin(name = "string-join", category = "String manipulation", related(string-split, string-append))]
/// Join a list of strings with delimiter.
///
/// # Examples
///
/// ```lisp
/// (string-join '("a" "b" "c") ",") => "a,b,c"
/// ```
///
/// # See Also
///
/// string-split, string-append
pub fn builtin_string_join(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let list = match &args[0] {
        Value::List(l) => l,
        _ => return Err(EvalError::TypeError),
    };

    let delimiter = match &args[1] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let strings: Result<Vec<String>, EvalError> = list
        .iter()
        .map(|v| match v {
            Value::String(s) => Ok(s.clone()),
            _ => Err(EvalError::TypeError),
        })
        .collect();

    Ok(Value::String(strings?.join(delimiter)))
}

#[builtin(name = "substring", category = "String manipulation", related(string-split, string-trim))]
/// Extract substring from start index (inclusive) to end index (exclusive).
///
/// # Examples
///
/// ```lisp
/// (substring "hello" 0 3) => "hel"
/// ```
///
/// # See Also
///
/// string-split, string-trim
pub fn builtin_substring(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 3 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let start = match &args[1] {
        Value::Number(n) if *n >= 0.0 && n.fract() == 0.0 => *n as usize,
        _ => return Err(EvalError::TypeError),
    };

    let end = match &args[2] {
        Value::Number(n) if *n >= 0.0 && n.fract() == 0.0 => *n as usize,
        _ => return Err(EvalError::TypeError),
    };

    let chars: Vec<char> = string.chars().collect();

    if start > chars.len() || end > chars.len() || start > end {
        return Err(EvalError::Custom(format!(
            "Invalid substring indices: start={}, end={}, length={}",
            start, end, chars.len()
        )));
    }

    let result: String = chars[start..end].iter().collect();
    Ok(Value::String(result))
}

#[builtin(name = "string-trim", category = "String manipulation", related(substring))]
/// Trim whitespace from both ends of string.
///
/// # Examples
///
/// ```lisp
/// (string-trim "  hello  ") => "hello"
/// ```
///
/// # See Also
///
/// substring
pub fn builtin_string_trim(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::String(string.trim().to_string()))
}

#[builtin(name = "string-upper", category = "String manipulation", related(string-lower))]
/// Convert string to uppercase.
///
/// # Examples
///
/// ```lisp
/// (string-upper "hello") => "HELLO"
/// ```
///
/// # See Also
///
/// string-lower
pub fn builtin_string_upper(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::String(string.to_uppercase()))
}

#[builtin(name = "string-lower", category = "String manipulation", related(string-upper))]
/// Convert string to lowercase.
///
/// # Examples
///
/// ```lisp
/// (string-lower "WORLD") => "world"
/// ```
///
/// # See Also
///
/// string-upper
pub fn builtin_string_lower(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::String(string.to_lowercase()))
}

#[builtin(name = "string-replace", category = "String manipulation", related(string-contains?))]
/// Replace all occurrences of pattern with replacement in string.
///
/// # Examples
///
/// ```lisp
/// (string-replace "hello" "l" "L") => "heLLo"
/// ```
///
/// # See Also
///
/// string-contains?
pub fn builtin_string_replace(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 3 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let pattern = match &args[1] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let replacement = match &args[2] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::String(string.replace(pattern, replacement)))
}

#[builtin(name = "string-contains?", category = "String manipulation", related(string-starts-with?, string-ends-with?))]
/// Check if string contains substring.
///
/// # Examples
///
/// ```lisp
/// (string-contains? "hello world" "world") => #t
/// ```
///
/// # See Also
///
/// string-starts-with?, string-ends-with?
pub fn builtin_string_contains(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let substring = match &args[1] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Bool(string.contains(substring.as_str())))
}

#[builtin(name = "string-starts-with?", category = "String manipulation", related(string-ends-with?, string-contains?))]
/// Check if string starts with prefix.
///
/// # Examples
///
/// ```lisp
/// (string-starts-with? "hello" "he") => #t
/// ```
///
/// # See Also
///
/// string-ends-with?, string-contains?
pub fn builtin_string_starts_with(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let prefix = match &args[1] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Bool(string.starts_with(prefix.as_str())))
}

#[builtin(name = "string-ends-with?", category = "String manipulation", related(string-starts-with?, string-contains?))]
/// Check if string ends with suffix.
///
/// # Examples
///
/// ```lisp
/// (string-ends-with? "hello" "lo") => #t
/// ```
///
/// # See Also
///
/// string-starts-with?, string-contains?
pub fn builtin_string_ends_with(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let suffix = match &args[1] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Bool(string.ends_with(suffix.as_str())))
}

#[builtin(name = "string-empty?", category = "String manipulation", related(string-length))]
/// Check if string is empty.
///
/// # Examples
///
/// ```lisp
/// (string-empty? "") => #t
/// ```
///
/// # See Also
///
/// string-length
pub fn builtin_string_empty(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Bool(string.is_empty()))
}

#[builtin(name = "string-length", category = "String manipulation", related(string-empty?))]
/// Get the length of a string (in characters, not bytes).
///
/// # Examples
///
/// ```lisp
/// (string-length "hello") => 5
/// ```
///
/// # See Also
///
/// string-empty?
pub fn builtin_string_length(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    Ok(Value::Number(string.chars().count() as f64))
}

#[builtin(name = "string->number", category = "String manipulation", related(number->string))]
/// Convert string to number.
///
/// # Examples
///
/// ```lisp
/// (string->number "42") => 42
/// ```
///
/// # See Also
///
/// number->string
pub fn builtin_string_to_number(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    match string.trim().parse::<f64>() {
        Ok(n) => Ok(Value::Number(n)),
        Err(_) => Ok(Value::Error(format!("Cannot parse '{}' as number", string))),
    }
}

#[builtin(name = "number->string", category = "String manipulation", related(string->number))]
/// Convert number to string.
///
/// # Examples
///
/// ```lisp
/// (number->string 42) => "42"
/// ```
///
/// # See Also
///
/// string->number
pub fn builtin_number_to_string(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let number = match &args[0] {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError),
    };

    // Format nicely: if it's a whole number, don't show decimal point
    let result = if number.fract() == 0.0 {
        format!("{:.0}", number)
    } else {
        format!("{}", number)
    };

    Ok(Value::String(result))
}

#[builtin(name = "string->list", category = "String manipulation", related(list->string))]
/// Convert string to list of characters.
///
/// # Examples
///
/// ```lisp
/// (string->list "abc") => ("a" "b" "c")
/// ```
///
/// # See Also
///
/// list->string
pub fn builtin_string_to_list(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => return Err(EvalError::TypeError),
    };

    let chars: Vec<Value> = string
        .chars()
        .map(|c| Value::String(c.to_string()))
        .collect();

    Ok(Value::List(chars))
}

#[builtin(name = "list->string", category = "String manipulation", related(string->list))]
/// Convert list of strings to single string.
///
/// # Examples
///
/// ```lisp
/// (list->string '("h" "e" "l" "l" "o")) => "hello"
/// ```
///
/// # See Also
///
/// string->list
pub fn builtin_list_to_string(args: &[Value]) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch);
    }

    let list = match &args[0] {
        Value::List(l) => l,
        _ => return Err(EvalError::TypeError),
    };

    let mut result = String::new();
    for item in list {
        match item {
            Value::String(s) => result.push_str(s),
            _ => return Err(EvalError::TypeError),
        }
    }

    Ok(Value::String(result))
}

#[builtin(name = "string-append", category = "String manipulation", related(string-join, list->string))]
/// Concatenate multiple strings into one.
///
/// Accepts variable number of arguments (0 or more strings).
///
/// # Examples
///
/// ```lisp
/// (string-append "hello" " " "world") => "hello world"
/// (string-append) => ""
/// ```
///
/// # See Also
///
/// string-join, list->string
pub fn builtin_string_append(args: &[Value]) -> Result<Value, EvalError> {
    let mut result = String::new();
    for arg in args {
        match arg {
            Value::String(s) => result.push_str(s),
            _ => return Err(EvalError::TypeError),
        }
    }
    Ok(Value::String(result))
}

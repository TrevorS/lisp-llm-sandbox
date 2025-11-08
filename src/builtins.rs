// ABOUTME: Built-in functions module providing core Lisp primitives

use crate::env::Environment;
use crate::error::EvalError;
use crate::sandbox::Sandbox;
use crate::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// Sandbox Storage for I/O Built-in Functions
// ============================================================================

thread_local! {
    static SANDBOX: RefCell<Option<Sandbox>> = const { RefCell::new(None) };
}

/// Initialize the sandbox for I/O built-in functions
pub fn set_sandbox_storage(sandbox: Sandbox) {
    SANDBOX.with(|s| {
        *s.borrow_mut() = Some(sandbox);
    });
}

// ============================================================================
// Builtin Definition Macro
// ============================================================================
//
// Unified macro for defining builtins with integrated help documentation.
// Converts identifiers to Lisp names and generates registration code.
//
// Usage:
//   define_builtin! {
//       identifier,
//       "Category",
//       "Description...",
//       examples: ["example1", "example2"],
//       related: [related_id1, related_id2],
//       |args| { /* implementation */ }
//   }
//
// Smart features:
// - identifier → "identifier" Lisp name + builtin_identifier Rust function
// - Related identifiers converted to strings automatically
// - Signature auto-generated as "(identifier ...)"
// - Optional overrides: name: "custom", signature: "custom sig"

macro_rules! define_builtin {
    // Main pattern: identifier, category, description, examples, related, implementation
    {
        $fn_ident:ident,
        $category:literal,
        $description:literal,
        examples: [$($example:literal),* $(,)?],
        related: [$($related_ident:ident),* $(,)?],
        $impl:expr
    } => {
        define_builtin! {
            @inner
            $fn_ident,
            stringify!($fn_ident),
            $category,
            concat!("(", stringify!($fn_ident), " ...)"),
            $description,
            [$($example),*],
            [$($related_ident),*],
            $impl
        }
    };

    // Pattern with name override
    {
        $fn_ident:ident,
        name: $lisp_name:literal,
        $category:literal,
        $description:literal,
        examples: [$($example:literal),* $(,)?],
        related: [$($related_ident:ident),* $(,)?],
        $impl:expr
    } => {
        define_builtin! {
            @inner
            $fn_ident,
            $lisp_name,
            $category,
            concat!("(", $lisp_name, " ...)"),
            $description,
            [$($example),*],
            [$($related_ident),*],
            $impl
        }
    };

    // Pattern with name and signature override
    {
        $fn_ident:ident,
        name: $lisp_name:literal,
        $category:literal,
        signature: $signature:literal,
        $description:literal,
        examples: [$($example:literal),* $(,)?],
        related: [$($related_ident:ident),* $(,)?],
        $impl:expr
    } => {
        define_builtin! {
            @inner
            $fn_ident,
            $lisp_name,
            $category,
            $signature,
            $description,
            [$($example),*],
            [$($related_ident),*],
            $impl
        }
    };

    // Inner implementation that generates everything
    {
        @inner
        $fn_ident:ident,
        $lisp_name:expr,
        $category:literal,
        $signature:expr,
        $description:literal,
        [$($example:literal),*],
        [$($related_ident:ident),*],
        $impl:expr
    } => {
        pub fn $fn_ident(args: &[Value]) -> Result<Value, EvalError> {
            $impl(args)
        }

        // Helper function to register this builtin
        paste::paste! {
            pub fn [<register_ $fn_ident>](
                env: std::rc::Rc<crate::env::Environment>
            ) {
                // Register the function
                env.define($lisp_name.to_string(), Value::BuiltIn($fn_ident));

                // Register help
                crate::help::register_help(crate::help::HelpEntry {
                    name: $lisp_name.to_string(),
                    category: $category.to_string(),
                    signature: $signature.to_string(),
                    description: $description.trim().to_string(),
                    examples: vec![$($example.to_string()),*],
                    related: vec![$(stringify!($related_ident).to_string()),*],
                });
            }
        }
    };
}

// ============================================================================
// Arithmetic Operations
// ============================================================================

define_builtin! {
    builtin_add,
    name: "+",
    "Arithmetic",
    "Returns the sum of all arguments.",
    examples: [
        "(+ 1 2 3) => 6",
        "(+ 10) => 10",
        "(+) => 0"
    ],
    related: [builtin_sub, builtin_mul, builtin_div],
    |args: &[Value]| {
        let mut sum = 0.0;
        for arg in args {
            match arg {
                Value::Number(n) => sum += n,
                _ => return Err(EvalError::TypeError),
            }
        }
        Ok(Value::Number(sum))
    }
}

define_builtin! {
    builtin_sub,
    name: "-",
    "Arithmetic",
    "Subtracts subsequent arguments from the first.\nWith one argument, returns its negation.",
    examples: [
        "(- 10 3 2) => 5",
        "(- 5) => -5"
    ],
    related: [builtin_add, builtin_mul, builtin_div],
    |args: &[Value]| {
        if args.is_empty() {
            return Err(EvalError::ArityMismatch);
        }

        let first = match args[0] {
            Value::Number(n) => n,
            _ => return Err(EvalError::TypeError),
        };

        if args.len() == 1 {
            return Ok(Value::Number(-first));
        }

        let mut result = first;
        for arg in &args[1..] {
            match arg {
                Value::Number(n) => result -= n,
                _ => return Err(EvalError::TypeError),
            }
        }
        Ok(Value::Number(result))
    }
}

define_builtin! {
    builtin_mul,
    name: "*",
    "Arithmetic",
    "Returns the product of all arguments.",
    examples: [
        "(* 2 3 4) => 24",
        "(* 5) => 5",
        "(*) => 1"
    ],
    related: [builtin_add, builtin_sub, builtin_div],
    |args: &[Value]| {
        let mut product = 1.0;
        for arg in args {
            match arg {
                Value::Number(n) => product *= n,
                _ => return Err(EvalError::TypeError),
            }
        }
        Ok(Value::Number(product))
    }
}

define_builtin! {
    builtin_div,
    name: "/",
    "Arithmetic",
    "Divides the first argument by subsequent arguments.\nInteger division in Lisp.",
    examples: [
        "(/ 20 4) => 5",
        "(/ 100 2 5) => 10"
    ],
    related: [builtin_add, builtin_sub, builtin_mul, builtin_mod],
    |args: &[Value]| {
        if args.is_empty() {
            return Err(EvalError::ArityMismatch);
        }

        let first = match args[0] {
            Value::Number(n) => n,
            _ => return Err(EvalError::TypeError),
        };

        if args.len() == 1 {
            if first == 0.0 {
                return Err(EvalError::Custom("Division by zero".to_string()));
            }
            return Ok(Value::Number(1.0 / first));
        }

        let mut result = first;
        for arg in &args[1..] {
            match arg {
                Value::Number(n) => {
                    if *n == 0.0 {
                        return Err(EvalError::Custom("Division by zero".to_string()));
                    }
                    result /= n;
                }
                _ => return Err(EvalError::TypeError),
            }
        }
        Ok(Value::Number(result))
    }
}

define_builtin! {
    builtin_mod,
    name: "%",
    "Arithmetic",
    "Returns the remainder when num1 is divided by num2.",
    examples: [
        "(% 17 5) => 2",
        "(% 10 3) => 1"
    ],
    related: [builtin_div],
    |args: &[Value]| {
        if args.len() != 2 {
            return Err(EvalError::ArityMismatch);
        }

        let a = match args[0] {
            Value::Number(n) => n,
            _ => return Err(EvalError::TypeError),
        };

        let b = match args[1] {
            Value::Number(n) => {
                if n == 0.0 {
                    return Err(EvalError::Custom("Division by zero".to_string()));
                }
                n
            }
            _ => return Err(EvalError::TypeError),
        };

        Ok(Value::Number(a % b))
    }
}

// ============================================================================
// Comparison Operations
// ============================================================================

define_builtin! {
    builtin_eq,
    name: "=",
    "Comparison",
    "Tests if all arguments are equal. Works with numbers, strings, symbols.",
    examples: [
        "(= 5 5) => #t",
        "(= 5 5 5) => #t",
        "(= 5 6) => #f",
        "(= \"hello\" \"hello\") => #t"
    ],
    related: [builtin_lt, builtin_gt, builtin_le, builtin_ge],
    |args: &[Value]| {
        if args.len() != 2 {
            return Err(EvalError::ArityMismatch);
        }

        let result = match (&args[0], &args[1]) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        };

        Ok(Value::Bool(result))
    }
}

define_builtin! {
    builtin_lt,
    name: "<",
    "Comparison",
    "Tests if each argument is strictly less than the next.",
    examples: [
        "(< 1 2 3) => #t",
        "(< 1 1) => #f",
        "(< 5 3) => #f"
    ],
    related: [builtin_gt, builtin_le, builtin_ge, builtin_eq],
    |args: &[Value]| {
        if args.len() != 2 {
            return Err(EvalError::ArityMismatch);
        }

        let a = match args[0] {
            Value::Number(n) => n,
            _ => return Err(EvalError::TypeError),
        };

        let b = match args[1] {
            Value::Number(n) => n,
            _ => return Err(EvalError::TypeError),
        };

        Ok(Value::Bool(a < b))
    }
}

define_builtin! {
    builtin_gt,
    name: ">",
    "Comparison",
    "Tests if each argument is strictly greater than the next.",
    examples: [
        "(> 3 2 1) => #t",
        "(> 3 3) => #f"
    ],
    related: [builtin_lt, builtin_le, builtin_ge, builtin_eq],
    |args: &[Value]| {
        if args.len() != 2 {
            return Err(EvalError::ArityMismatch);
        }

        let a = match args[0] {
            Value::Number(n) => n,
            _ => return Err(EvalError::TypeError),
        };

        let b = match args[1] {
            Value::Number(n) => n,
            _ => return Err(EvalError::TypeError),
        };

        Ok(Value::Bool(a > b))
    }
}

define_builtin! {
    builtin_le,
    name: "<=",
    "Comparison",
    "Tests if each argument is less than or equal to the next.",
    examples: [
        "(<= 1 2 2 3) => #t",
        "(<= 5 5) => #t"
    ],
    related: [builtin_lt, builtin_gt, builtin_ge, builtin_eq],
    |args: &[Value]| {
        if args.len() != 2 {
            return Err(EvalError::ArityMismatch);
        }

        let a = match args[0] {
            Value::Number(n) => n,
            _ => return Err(EvalError::TypeError),
        };

        let b = match args[1] {
            Value::Number(n) => n,
            _ => return Err(EvalError::TypeError),
        };

        Ok(Value::Bool(a <= b))
    }
}

define_builtin! {
    builtin_ge,
    name: ">=",
    "Comparison",
    "Tests if each argument is greater than or equal to the next.",
    examples: [
        "(>= 3 2 2 1) => #t",
        "(>= 5 5) => #t"
    ],
    related: [builtin_lt, builtin_gt, builtin_le, builtin_eq],
    |args: &[Value]| {
        if args.len() != 2 {
            return Err(EvalError::ArityMismatch);
        }

        let a = match args[0] {
            Value::Number(n) => n,
            _ => return Err(EvalError::TypeError),
        };

        let b = match args[1] {
            Value::Number(n) => n,
            _ => return Err(EvalError::TypeError),
        };

        Ok(Value::Bool(a >= b))
    }
}

// ============================================================================
// Logic Operations
// ============================================================================

define_builtin! {
    builtin_and,
    name: "and",
    "Logic",
    "Logical AND. Returns #f if any argument is falsy, otherwise returns the last argument.\nShort-circuits: stops evaluating after first falsy value.",
    examples: [
        "(and #t #t #t) => #t",
        "(and #t #f #t) => #f",
        "(and 1 2 3) => 3"
    ],
    related: [builtin_or, builtin_not],
    |args: &[Value]| {
        for arg in args {
            match arg {
                Value::Bool(false) => return Ok(Value::Bool(false)),
                Value::Bool(true) => continue,
                _ => return Err(EvalError::TypeError),
            }
        }
        Ok(Value::Bool(true))
    }
}

define_builtin! {
    builtin_or,
    name: "or",
    "Logic",
    "Logical OR. Returns the first truthy value or #f if all are falsy.\nShort-circuits: stops evaluating after first truthy value.",
    examples: [
        "(or #f #f #t) => #t",
        "(or #f #f) => #f",
        "(or nil 2) => 2"
    ],
    related: [builtin_and, builtin_not],
    |args: &[Value]| {
        for arg in args {
            match arg {
                Value::Bool(true) => return Ok(Value::Bool(true)),
                Value::Bool(false) => continue,
                _ => return Err(EvalError::TypeError),
            }
        }
        Ok(Value::Bool(false))
    }
}

define_builtin! {
    builtin_not,
    name: "not",
    "Logic",
    "Logical NOT. Returns #t if val is falsy (#f or nil), otherwise #f.",
    examples: [
        "(not #f) => #t",
        "(not #t) => #f",
        "(not nil) => #t",
        "(not 5) => #f"
    ],
    related: [builtin_and, builtin_or],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        match args[0] {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(EvalError::TypeError),
        }
    }
}

// ============================================================================
// List Operations
// ============================================================================

define_builtin! {
    builtin_cons,
    name: "cons",
    "List operations",
    "Constructs a new list by prepending elem to list.\nReturns a new list; original is not modified.",
    examples: [
        "(cons 1 '(2 3)) => (1 2 3)",
        "(cons 'a '(b c)) => (a b c)",
        "(cons 1 nil) => (1)"
    ],
    related: [builtin_car, builtin_cdr, builtin_list],
    |args: &[Value]| {
        if args.len() != 2 {
            return Err(EvalError::ArityMismatch);
        }

        let mut result = vec![args[0].clone()];

        match &args[1] {
            Value::List(items) => result.extend(items.clone()),
            Value::Nil => (),
            _ => return Err(EvalError::TypeError),
        }

        Ok(Value::List(result))
    }
}

define_builtin! {
    builtin_car,
    name: "car",
    "List operations",
    "Returns the first element of a list. Also called 'head'.\nThrows error on empty list or non-list.",
    examples: [
        "(car '(1 2 3)) => 1",
        "(car '(a)) => a"
    ],
    related: [builtin_cdr, builtin_cons],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        match &args[0] {
            Value::List(items) if !items.is_empty() => Ok(items[0].clone()),
            Value::List(_) => Err(EvalError::Custom("car of empty list".to_string())),
            _ => Err(EvalError::TypeError),
        }
    }
}

define_builtin! {
    builtin_cdr,
    name: "cdr",
    "List operations",
    "Returns all elements except the first. Also called 'tail'.\nReturns nil for single-element list.",
    examples: [
        "(cdr '(1 2 3)) => (2 3)",
        "(cdr '(a b)) => (b)",
        "(cdr '(1)) => nil"
    ],
    related: [builtin_car, builtin_cons],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        match &args[0] {
            Value::List(items) if !items.is_empty() => {
                if items.len() == 1 {
                    Ok(Value::Nil)
                } else {
                    Ok(Value::List(items[1..].to_vec()))
                }
            }
            Value::List(_) => Err(EvalError::Custom("cdr of empty list".to_string())),
            _ => Err(EvalError::TypeError),
        }
    }
}

define_builtin! {
    builtin_list,
    name: "list",
    "List operations",
    "Creates a new list containing the given elements in order.",
    examples: [
        "(list 1 2 3) => (1 2 3)",
        "(list 'a 'b) => (a b)",
        "(list) => nil"
    ],
    related: [builtin_cons, builtin_car, builtin_cdr],
    |args: &[Value]| {
        Ok(Value::List(args.to_vec()))
    }
}

define_builtin! {
    builtin_length,
    name: "length",
    "List operations",
    "Returns the number of elements in a list.",
    examples: [
        "(length '(1 2 3)) => 3",
        "(length '()) => 0",
        "(length '(a)) => 1"
    ],
    related: [builtin_empty_q, builtin_list],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        match &args[0] {
            Value::List(items) => Ok(Value::Number(items.len() as f64)),
            Value::Nil => Ok(Value::Number(0.0)),
            _ => Err(EvalError::TypeError),
        }
    }
}

define_builtin! {
    builtin_empty_q,
    name: "empty?",
    "List operations",
    signature: "(empty? list)",
    "Tests if a list is empty (nil or ()).\nReturns #t for empty lists, #f otherwise.",
    examples: [
        "(empty? nil) => #t",
        "(empty? '()) => #t",
        "(empty? '(1)) => #f"
    ],
    related: [builtin_length, builtin_nil_p],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        match &args[0] {
            Value::List(items) => Ok(Value::Bool(items.is_empty())),
            Value::Nil => Ok(Value::Bool(true)),
            _ => Err(EvalError::TypeError),
        }
    }
}

// ============================================================================
// Type Predicates
// ============================================================================

define_builtin! {
    builtin_number_p,
    name: "number?",
    "Type predicates",
    signature: "(number? val)",
    "Tests if val is a number (integer or float).",
    examples: [
        "(number? 42) => #t",
        "(number? 3.14) => #t",
        "(number? \"42\") => #f"
    ],
    related: [builtin_string_p, builtin_symbol_p, builtin_list_p],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        Ok(Value::Bool(matches!(args[0], Value::Number(_))))
    }
}

define_builtin! {
    builtin_string_p,
    name: "string?",
    "Type predicates",
    signature: "(string? val)",
    "Tests if val is a string.",
    examples: [
        "(string? \"hello\") => #t",
        "(string? 42) => #f",
        "(string? 'hello) => #f"
    ],
    related: [builtin_number_p, builtin_symbol_p],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        Ok(Value::Bool(matches!(args[0], Value::String(_))))
    }
}

define_builtin! {
    builtin_list_p,
    name: "list?",
    "Type predicates",
    signature: "(list? val)",
    "Tests if val is a list (including nil).",
    examples: [
        "(list? '(1 2 3)) => #t",
        "(list? nil) => #t",
        "(list? 42) => #f"
    ],
    related: [builtin_number_p, builtin_string_p, builtin_nil_p],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        Ok(Value::Bool(matches!(args[0], Value::List(_))))
    }
}

define_builtin! {
    builtin_nil_p,
    name: "nil?",
    "Type predicates",
    signature: "(nil? val)",
    "Tests if val is nil (empty list).",
    examples: [
        "(nil? nil) => #t",
        "(nil? '()) => #t",
        "(nil? 0) => #f"
    ],
    related: [builtin_empty_q, builtin_list_p],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        Ok(Value::Bool(matches!(args[0], Value::Nil)))
    }
}

define_builtin! {
    builtin_symbol_p,
    name: "symbol?",
    "Type predicates",
    signature: "(symbol? val)",
    "Tests if val is a symbol (e.g., from 'hello or var names).",
    examples: [
        "(symbol? 'hello) => #t",
        "(symbol? \"hello\") => #f",
        "(symbol? hello) => error (undefined variable)"
    ],
    related: [builtin_string_p, builtin_number_p],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        Ok(Value::Bool(matches!(args[0], Value::Symbol(_))))
    }
}

define_builtin! {
    builtin_bool_p,
    name: "bool?",
    "Type predicates",
    signature: "(bool? val)",
    "Tests if val is a boolean (#t or #f).",
    examples: [
        "(bool? #t) => #t",
        "(bool? #f) => #t",
        "(bool? 1) => #f"
    ],
    related: [builtin_number_p, builtin_string_p],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
    }
}

// ============================================================================
// Console I/O
// ============================================================================

define_builtin! {
    builtin_print,
    name: "print",
    "Console I/O",
    "Prints values to stdout without newline. Returns nil.",
    examples: [
        "(print \"hello\") => outputs: hello",
        "(print 1 2 3) => outputs: 1 2 3"
    ],
    related: [builtin_println],
    |args: &[Value]| {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            match arg {
                Value::String(s) => print!("{}", s),
                other => print!("{}", other),
            }
        }
        Ok(Value::Nil)
    }
}

define_builtin! {
    builtin_println,
    name: "println",
    "Console I/O",
    "Prints values to stdout with newline at end. Returns nil.",
    examples: [
        "(println \"hello\") => outputs: hello",
        "(println \"a\" \"b\") => outputs: a b"
    ],
    related: [builtin_print],
    |args: &[Value]| {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            match arg {
                Value::String(s) => print!("{}", s),
                other => print!("{}", other),
            }
        }
        println!();
        Ok(Value::Nil)
    }
}

// ============================================================================
// Error Handling
// ============================================================================

define_builtin! {
    error,
    "Error handling",
    "Raises an error with the given message. Always throws.",
    examples: ["(error \"invalid input\") => Error: invalid input"],
    related: [error_p, error_msg],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        let msg = match &args[0] {
            Value::String(s) => s.clone(),
            other => format!("{}", other),
        };

        Ok(Value::Error(msg))
    }
}

define_builtin! {
    error_p,
    name: "error?",
    "Error handling",
    "Tests if val is an error value.",
    examples: ["(error? (error \"test\")) => would throw before testing"],
    related: [error, error_msg],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        Ok(Value::Bool(matches!(args[0], Value::Error(_))))
    }
}

define_builtin! {
    error_msg,
    name: "error-msg",
    "Error handling",
    "Extracts the message from an error value.",
    examples: ["(error-msg (error \"test\")) => would throw before extracting"],
    related: [error, error_p],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        match &args[0] {
            Value::Error(msg) => Ok(Value::String(msg.clone())),
            _ => Err(EvalError::TypeError),
        }
    }
}

// ============================================================================
// Help System
// ============================================================================

define_builtin! {
    help,
    name: "help",
    "Help system",
    signature: "(help) or (help 'function-name)",
    "Show help information. With no arguments, displays quick reference.\nWith a function name, shows detailed documentation for that function.",
    examples: [
        "(help) => shows quick reference",
        "(help 'cons) => detailed help for cons",
        "(help 'map) => help for user or stdlib function"
    ],
    related: [doc],
    |args: &[Value]| {
        use crate::help;

        match args.len() {
            0 => {
                // Show quick reference
                let output = help::format_quick_reference();
                println!("{}", output);
                Ok(Value::Nil)
            }
            1 => {
                // Get help for specific function
                match &args[0] {
                    Value::Symbol(name) => {
                        // First try built-in help
                        if let Some(entry) = help::get_help(name) {
                            let output = help::format_help_entry(&entry);
                            println!("{}", output);
                            return Ok(Value::Nil);
                        }

                        // If not found in help registry, it might be a user function
                        // User functions would need to be looked up in environment
                        // For now, just report not found
                        Err(EvalError::Custom(format!("No help found for '{}'", name)))
                    }
                    _ => Err(EvalError::TypeError),
                }
            }
            _ => Err(EvalError::ArityMismatch),
        }
    }
}

define_builtin! {
    doc,
    "Help system",
    "Returns the docstring of a function as a string.\nWorks with user-defined functions that have docstrings.",
    examples: ["(doc factorial) => \"Computes factorial\""],
    related: [help],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        match &args[0] {
            Value::Lambda { docstring, .. } => match docstring {
                Some(doc) => Ok(Value::String(doc.clone())),
                None => Ok(Value::Nil),
            },
            _ => Err(EvalError::TypeError),
        }
    }
}

// ============================================================================
// Filesystem I/O
// ============================================================================

define_builtin! {
    read_file,
    name: "read-file",
    "Filesystem I/O",
    signature: "(read-file path)",
    "Reads and returns the contents of a file as a string.\nPath is relative to allowed sandbox directories.",
    examples: ["(read-file \"data/input.txt\") => \"file contents\""],
    related: [write_file, file_exists_q],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        let path = match &args[0] {
            Value::String(s) => s,
            _ => return Err(EvalError::TypeError),
        };

        SANDBOX.with(|s| {
            let sandbox_ref = s.borrow();
            let sandbox = sandbox_ref
                .as_ref()
                .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

            sandbox
                .read_file(path)
                .map(Value::String)
                .map_err(|e| EvalError::IoError(e.to_string()))
        })
    }
}

define_builtin! {
    write_file,
    name: "write-file",
    "Filesystem I/O",
    signature: "(write-file path contents)",
    "Writes contents to a file, creating it if it doesn't exist.\nReturns #t on success. Path is relative to sandbox.",
    examples: ["(write-file \"data/output.txt\" \"hello\") => #t"],
    related: [read_file, file_exists_q],
    |args: &[Value]| {
        if args.len() != 2 {
            return Err(EvalError::ArityMismatch);
        }

        let path = match &args[0] {
            Value::String(s) => s,
            _ => return Err(EvalError::TypeError),
        };

        let contents = match &args[1] {
            Value::String(s) => s,
            _ => return Err(EvalError::TypeError),
        };

        SANDBOX.with(|s| {
            let sandbox_ref = s.borrow();
            let sandbox = sandbox_ref
                .as_ref()
                .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

            sandbox
                .write_file(path, contents)
                .map(|_| Value::Bool(true))
                .map_err(|e| EvalError::IoError(e.to_string()))
        })
    }
}

define_builtin! {
    file_exists_q,
    name: "file-exists?",
    "Filesystem I/O",
    signature: "(file-exists? path)",
    "Tests if a file exists and is accessible in sandbox.\nReturns #t or #f.",
    examples: [
        "(file-exists? \"data/file.txt\") => #t",
        "(file-exists? \"nonexistent.txt\") => #f"
    ],
    related: [file_size, read_file],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        let path = match &args[0] {
            Value::String(s) => s,
            _ => return Err(EvalError::TypeError),
        };

        SANDBOX.with(|s| {
            let sandbox_ref = s.borrow();
            let sandbox = sandbox_ref
                .as_ref()
                .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

            sandbox
                .file_exists(path)
                .map(Value::Bool)
                .map_err(|e| EvalError::IoError(e.to_string()))
        })
    }
}

define_builtin! {
    file_size,
    name: "file-size",
    "Filesystem I/O",
    signature: "(file-size path)",
    "Returns the size of a file in bytes.\nThrows error if file doesn't exist.",
    examples: ["(file-size \"data/file.txt\") => 1024"],
    related: [file_exists_q, read_file],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        let path = match &args[0] {
            Value::String(s) => s,
            _ => return Err(EvalError::TypeError),
        };

        SANDBOX.with(|s| {
            let sandbox_ref = s.borrow();
            let sandbox = sandbox_ref
                .as_ref()
                .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

            sandbox
                .file_size(path)
                .map(|size| Value::Number(size as f64))
                .map_err(|e| EvalError::IoError(e.to_string()))
        })
    }
}

define_builtin! {
    list_files,
    name: "list-files",
    "Filesystem I/O",
    signature: "(list-files directory)",
    "Returns a list of filenames in a directory.\nDoes not include . or .., returns only names not full paths.",
    examples: ["(list-files \"data\") => (\"file1.txt\" \"file2.txt\")"],
    related: [file_exists_q],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        let dir = match &args[0] {
            Value::String(s) => s,
            _ => return Err(EvalError::TypeError),
        };

        SANDBOX.with(|s| {
            let sandbox_ref = s.borrow();
            let sandbox = sandbox_ref
                .as_ref()
                .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

            sandbox
                .list_files(dir)
                .map(|files| Value::List(files.into_iter().map(Value::String).collect::<Vec<_>>()))
                .map_err(|e| EvalError::IoError(e.to_string()))
        })
    }
}

// ============================================================================
// Network I/O
// ============================================================================

define_builtin! {
    http_get,
    name: "http-get",
    "Network I/O",
    signature: "(http-get url)",
    "Performs an HTTP GET request and returns the response body as a string.\nURL must be in allowed addresses list. Has 30 second timeout.\nWARNING: DNS lookup cannot be interrupted, may hang if DNS is slow.",
    examples: ["(http-get \"https://example.com\") => \"<html>...\""],
    related: [http_post],
    |args: &[Value]| {
        if args.len() != 1 {
            return Err(EvalError::ArityMismatch);
        }

        let url = match &args[0] {
            Value::String(s) => s,
            _ => return Err(EvalError::TypeError),
        };

        SANDBOX.with(|s| {
            let sandbox_ref = s.borrow();
            let sandbox = sandbox_ref
                .as_ref()
                .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

            sandbox
                .http_get(url)
                .map(Value::String)
                .map_err(|e| EvalError::IoError(e.to_string()))
        })
    }
}

define_builtin! {
    http_post,
    name: "http-post",
    "Network I/O",
    signature: "(http-post url body)",
    "Performs an HTTP POST request and returns the response body as a string.\nURL must be in allowed addresses. Sends body as plain text. 30 second timeout.\nWARNING: DNS lookup cannot be interrupted, may hang if DNS is slow.",
    examples: ["(http-post \"https://api.example.com\" \"data\") => \"response\""],
    related: [http_get],
    |args: &[Value]| {
        if args.len() != 2 {
            return Err(EvalError::ArityMismatch);
        }

        let url = match &args[0] {
            Value::String(s) => s,
            _ => return Err(EvalError::TypeError),
        };

        let body = match &args[1] {
            Value::String(s) => s,
            _ => return Err(EvalError::TypeError),
        };

        SANDBOX.with(|s| {
            let sandbox_ref = s.borrow();
            let sandbox = sandbox_ref
                .as_ref()
                .ok_or_else(|| EvalError::IoError("Sandbox not initialized".to_string()))?;

            sandbox
                .http_post(url, body)
                .map(Value::String)
                .map_err(|e| EvalError::IoError(e.to_string()))
        })
    }
}

// ============================================================================
// Registration
// ============================================================================

/// Register all built-in functions in the global environment
pub fn register_builtins(env: Rc<Environment>) {
    // Arithmetic
    register_builtin_add(env.clone());
    register_builtin_sub(env.clone());
    register_builtin_mul(env.clone());
    register_builtin_div(env.clone());
    register_builtin_mod(env.clone());

    // Comparison
    register_builtin_eq(env.clone());
    register_builtin_lt(env.clone());
    register_builtin_gt(env.clone());
    register_builtin_le(env.clone());
    register_builtin_ge(env.clone());

    // Logic
    register_builtin_and(env.clone());
    register_builtin_or(env.clone());
    register_builtin_not(env.clone());

    // List operations
    register_builtin_cons(env.clone());
    register_builtin_car(env.clone());
    register_builtin_cdr(env.clone());
    register_builtin_list(env.clone());
    register_builtin_length(env.clone());
    register_builtin_empty_q(env.clone());

    // Type predicates
    register_builtin_number_p(env.clone());
    register_builtin_string_p(env.clone());
    register_builtin_list_p(env.clone());
    register_builtin_nil_p(env.clone());
    register_builtin_symbol_p(env.clone());
    register_builtin_bool_p(env.clone());

    // I/O - Console
    register_builtin_print(env.clone());
    register_builtin_println(env.clone());

    // I/O - Filesystem
    register_read_file(env.clone());
    register_write_file(env.clone());
    register_file_exists_q(env.clone());
    register_file_size(env.clone());
    register_list_files(env.clone());

    // I/O - Network
    register_http_get(env.clone());
    register_http_post(env.clone());

    // Error handling
    register_error(env.clone());
    register_error_p(env.clone());
    register_error_msg(env.clone());

    // Help system
    register_help(env.clone());
    register_doc(env.clone());
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Arithmetic Tests
    // ========================================================================

    #[test]
    fn test_add() {
        let result =
            builtin_add(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 6.0),
            _ => panic!("Expected Number(6.0)"),
        }

        // Zero arguments
        let result = builtin_add(&[]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 0.0),
            _ => panic!("Expected Number(0.0)"),
        }
    }

    #[test]
    fn test_sub() {
        let result = builtin_sub(&[Value::Number(10.0), Value::Number(3.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 7.0),
            _ => panic!("Expected Number(7.0)"),
        }

        // Single argument negation
        let result = builtin_sub(&[Value::Number(5.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, -5.0),
            _ => panic!("Expected Number(-5.0)"),
        }

        // Multiple arguments
        let result =
            builtin_sub(&[Value::Number(10.0), Value::Number(5.0), Value::Number(2.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 3.0),
            _ => panic!("Expected Number(3.0)"),
        }
    }

    #[test]
    fn test_mul() {
        let result =
            builtin_mul(&[Value::Number(2.0), Value::Number(3.0), Value::Number(4.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 24.0),
            _ => panic!("Expected Number(24.0)"),
        }

        // Zero arguments
        let result = builtin_mul(&[]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 1.0),
            _ => panic!("Expected Number(1.0)"),
        }
    }

    #[test]
    fn test_div() {
        let result = builtin_div(&[Value::Number(10.0), Value::Number(2.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 5.0),
            _ => panic!("Expected Number(5.0)"),
        }

        // Division by zero
        let result = builtin_div(&[Value::Number(10.0), Value::Number(0.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_mod() {
        let result = builtin_mod(&[Value::Number(10.0), Value::Number(3.0)]).unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 1.0),
            _ => panic!("Expected Number(1.0)"),
        }
    }

    // ========================================================================
    // Comparison Tests
    // ========================================================================

    #[test]
    fn test_eq() {
        let result = builtin_eq(&[Value::Number(1.0), Value::Number(1.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_eq(&[Value::Number(1.0), Value::Number(2.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_lt() {
        let result = builtin_lt(&[Value::Number(1.0), Value::Number(2.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_lt(&[Value::Number(2.0), Value::Number(1.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_gt() {
        let result = builtin_gt(&[Value::Number(2.0), Value::Number(1.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    fn test_le() {
        let result = builtin_le(&[Value::Number(1.0), Value::Number(1.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    fn test_ge() {
        let result = builtin_ge(&[Value::Number(2.0), Value::Number(1.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }
    }

    // ========================================================================
    // Logic Tests
    // ========================================================================

    #[test]
    fn test_and() {
        let result = builtin_and(&[Value::Bool(true), Value::Bool(false)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }

        let result = builtin_and(&[Value::Bool(true), Value::Bool(true)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }
    }

    #[test]
    fn test_or() {
        let result = builtin_or(&[Value::Bool(true), Value::Bool(false)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_or(&[Value::Bool(false), Value::Bool(false)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_not() {
        let result = builtin_not(&[Value::Bool(false)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_not(&[Value::Bool(true)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    // ========================================================================
    // List Operation Tests
    // ========================================================================

    #[test]
    fn test_cons() {
        let result = builtin_cons(&[
            Value::Number(1.0),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ])
        .unwrap();
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                match &items[0] {
                    Value::Number(n) => assert_eq!(*n, 1.0),
                    _ => panic!("Expected Number(1.0)"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_car() {
        let result = builtin_car(&[Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])])
        .unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 1.0),
            _ => panic!("Expected Number(1.0)"),
        }
    }

    #[test]
    fn test_cdr() {
        let result = builtin_cdr(&[Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])])
        .unwrap();
        match result {
            Value::List(items) => assert_eq!(items.len(), 2),
            _ => panic!("Expected List"),
        }

        // Single element
        let result = builtin_cdr(&[Value::List(vec![Value::Number(1.0)])]).unwrap();
        match result {
            Value::Nil => (),
            _ => panic!("Expected Nil"),
        }
    }

    #[test]
    fn test_list() {
        let result =
            builtin_list(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]).unwrap();
        match result {
            Value::List(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_length() {
        let result = builtin_length(&[Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])])
        .unwrap();
        match result {
            Value::Number(n) => assert_eq!(n, 3.0),
            _ => panic!("Expected Number(3.0)"),
        }
    }

    #[test]
    fn test_empty_q() {
        let result = builtin_empty_q(&[Value::List(vec![])]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_empty_q(&[Value::List(vec![Value::Number(1.0)])]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    // ========================================================================
    // Type Predicate Tests
    // ========================================================================

    #[test]
    fn test_number_p() {
        let result = builtin_number_p(&[Value::Number(42.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_number_p(&[Value::String("hello".to_string())]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_string_p() {
        let result = builtin_string_p(&[Value::String("hello".to_string())]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_string_p(&[Value::Number(42.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_list_p() {
        let result = builtin_list_p(&[Value::List(vec![Value::Number(1.0)])]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_list_p(&[Value::Number(42.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_nil_p() {
        let result = builtin_nil_p(&[Value::Nil]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_nil_p(&[Value::Number(42.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_symbol_p() {
        let result = builtin_symbol_p(&[Value::Symbol("x".to_string())]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_symbol_p(&[Value::Number(42.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }

    #[test]
    fn test_bool_p() {
        let result = builtin_bool_p(&[Value::Bool(true)]).unwrap();
        match result {
            Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool(true)"),
        }

        let result = builtin_bool_p(&[Value::Number(42.0)]).unwrap();
        match result {
            Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool(false)"),
        }
    }
}

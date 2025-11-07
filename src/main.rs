mod builtins;
mod config;
mod env;
mod error;
mod eval;
mod macros;
mod parser;
mod tools;
mod value;

use builtins::register_builtins;
use config::{WELCOME_MESSAGE, WELCOME_SUBTITLE};
use env::Environment;
use eval::eval_with_macros;
use macros::MacroRegistry;
use parser::parse;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

fn main() -> rustyline::Result<()> {
    // Initialize environment and macros
    let env = Environment::new();
    let mut macro_reg = MacroRegistry::new();
    register_builtins(env.clone());

    // Load standard library
    let stdlib = include_str!("stdlib.lisp");
    match load_stdlib(stdlib, env.clone(), &mut macro_reg) {
        Ok(_) => {} // Silently succeed
        Err(e) => eprintln!("Warning: Failed to load stdlib: {}", e),
    }

    // Create REPL with history support
    let mut rl = DefaultEditor::new()?;

    // Try to load history from previous sessions
    let history_file = ".lisp_history";
    let _ = rl.load_history(history_file);

    // Print welcome message
    println!("{}", WELCOME_MESSAGE);
    println!("{}", WELCOME_SUBTITLE);
    println!("Type (quit) or (exit) to exit, (help) for commands");
    println!("Type (builtins) to see all available functions\n");

    // REPL loop
    loop {
        let readline = rl.readline("lisp> ");

        match readline {
            Ok(line) => {
                // Skip empty lines
                if line.trim().is_empty() {
                    continue;
                }

                // Handle special commands
                match line.trim() {
                    "(quit)" | "(exit)" => {
                        println!("Goodbye!");
                        break;
                    }
                    "(clear)" => {
                        print!("\x1B[2J\x1B[H"); // ANSI clear screen
                        continue;
                    }
                    "(help)" => {
                        print_help();
                        continue;
                    }
                    "(builtins)" => {
                        print_builtins();
                        continue;
                    }
                    _ => {}
                }

                // Parse and evaluate the expression
                match parse(&line) {
                    Ok(expr) => match eval_with_macros(expr, env.clone(), &mut macro_reg) {
                        Ok(result) => {
                            println!("=> {}", result);
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Parse error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Handle Ctrl-C
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Handle Ctrl-D
                println!("\nGoodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }
    }

    // Save history on exit
    let _ = rl.save_history(history_file);

    Ok(())
}

fn print_help() {
    println!("Available commands:");
    println!("  (quit), (exit) - Exit the REPL");
    println!("  (clear)        - Clear the screen");
    println!("  (help)         - Show this message");
    println!("  (builtins)     - List built-in functions");
    println!();
    println!("Special forms:");
    println!("  define, lambda, if, begin, let");
    println!("  quasiquote (`), unquote (,), unquote-splicing (,@)");
    println!("  defmacro");
    println!();
    println!("Examples:");
    println!("  (define x 42)");
    println!("  (define (square x) (* x x))");
    println!("  (square 5)");
    println!("  (if (< 10 20) \"yes\" \"no\")");
    println!("  (let ((x 10) (y 20)) (+ x y))");
}

fn print_builtins() {
    println!("Built-in functions:");
    println!();
    println!("Arithmetic:");
    println!("  +, -, *, /, %");
    println!();
    println!("Comparison:");
    println!("  =, <, >, <=, >=");
    println!();
    println!("Logic:");
    println!("  and, or, not");
    println!();
    println!("List operations:");
    println!("  cons, car, cdr, list, length, empty?");
    println!();
    println!("Type predicates:");
    println!("  number?, string?, list?, nil?, symbol?, bool?");
    println!();
    println!("I/O:");
    println!("  print, println");
    println!();
    println!("Error handling:");
    println!("  error, error?, error-msg");
    println!();
    println!("Standard library functions loaded from stdlib.lisp:");
    println!("  map, filter, reduce, reverse, append, member, nth, last");
    println!("  take, drop, zip, all, any, count, range, compose, partial");
    println!("  abs, min, max, square, cube, even?, odd?, sum, product, factorial");
    println!("  Macros: when, unless");
}

/// Load and evaluate the standard library
fn load_stdlib(
    code: &str,
    env: std::rc::Rc<Environment>,
    macro_reg: &mut MacroRegistry,
) -> Result<(), String> {
    // Parse each expression in the stdlib
    // We need to handle multiple top-level forms
    let mut remaining = code.trim();

    while !remaining.is_empty() {
        // Skip whitespace and comments
        remaining = skip_whitespace_and_comments(remaining);
        if remaining.is_empty() {
            break;
        }

        // Parse one expression
        match parse_one_expr(remaining) {
            Ok((expr, rest)) => {
                // Evaluate the expression
                match eval_with_macros(expr, env.clone(), macro_reg) {
                    Ok(_) => {
                        remaining = rest;
                    }
                    Err(e) => {
                        return Err(format!("Eval error: {:?}", e));
                    }
                }
            }
            Err(e) => {
                return Err(format!("Parse error: {}", e));
            }
        }
    }

    Ok(())
}

/// Skip whitespace and comments in the input string
fn skip_whitespace_and_comments(input: &str) -> &str {
    let mut remaining = input;
    loop {
        remaining = remaining.trim_start();
        if remaining.starts_with(';') {
            // Skip until end of line
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

/// Parse one expression and return it along with the remaining input
fn parse_one_expr(input: &str) -> Result<(value::Value, &str), String> {
    let trimmed = skip_whitespace_and_comments(input);
    if trimmed.is_empty() {
        return Err("No expression to parse".to_string());
    }

    // Find the end of the first complete s-expression
    let end_pos = find_expr_end(trimmed)?;
    let expr_str = &trimmed[..end_pos];
    let rest = &trimmed[end_pos..];

    // Parse the expression
    let expr = parse(expr_str)?;
    Ok((expr, rest))
}

/// Find the end position of the first complete s-expression
fn find_expr_end(input: &str) -> Result<usize, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    // Skip initial whitespace
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }

    if i >= chars.len() {
        return Err("Empty input".to_string());
    }

    // Check what kind of expression this is
    if chars[i] == '(' {
        // S-expression - find matching closing paren
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
        // Atom - find end of token
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != ')' {
            i += 1;
        }
        Ok(i)
    }
}

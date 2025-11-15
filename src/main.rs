mod builtins;
mod config;
mod env;
mod error;
mod eval;
mod help;
mod highlighter;
mod macros;
mod parser;
mod sandbox;
mod stdlib;
mod stdlib_registry;
mod tools;
mod value;

use builtins::{register_builtins, set_sandbox_storage};
use clap::Parser;
use config::{FsConfig, NetConfig, WELCOME_FOOTER, WELCOME_MESSAGE, WELCOME_SUBTITLE};
use env::Environment;
use eval::eval_with_macros;
use highlighter::LispHelper;
use macros::MacroRegistry;
use parser::parse;
use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};
use sandbox::Sandbox;
use std::path::PathBuf;
use std::rc::Rc;
use stdlib::register_stdlib;
use stdlib_registry::register_stdlib_functions;

/// Lisp interpreter with sandboxed I/O capabilities
#[derive(Parser, Debug)]
#[command(name = "lisp-llm-sandbox")]
#[command(version = config::VERSION)]
#[command(about = "A production-ready Scheme-flavored Lisp interpreter")]
#[command(long_about = "An interpreter with capability-based I/O sandboxing")]
struct CliArgs {
    /// Script file to execute (optional - if not provided, starts REPL)
    #[arg(value_name = "FILE")]
    script: Option<PathBuf>,

    /// Add allowed filesystem path (can be repeated)
    #[arg(long = "fs-sandbox", value_name = "PATH", action = clap::ArgAction::Append)]
    fs_paths: Vec<PathBuf>,

    /// Maximum file size in bytes
    #[arg(
        long = "max-file-size",
        value_name = "BYTES",
        default_value = "10485760"
    )]
    max_file_size: usize,

    /// Enable network I/O
    #[arg(long = "allow-network")]
    allow_network: bool,

    /// Add allowed network address (can be repeated)
    #[arg(long = "net-allow", value_name = "ADDR", action = clap::ArgAction::Append)]
    net_addresses: Vec<String>,

    /// Skip loading standard library
    #[arg(long = "no-stdlib")]
    no_stdlib: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments
    let args = CliArgs::parse();

    // Build sandbox configuration from CLI args
    let fs_config = build_fs_config(&args);
    let net_config = build_net_config(&args);

    // Initialize sandbox with configuration
    let sandbox = Sandbox::new(fs_config, net_config)?;
    set_sandbox_storage(sandbox);

    // Initialize environment and macros
    let env = Environment::new();
    let mut macro_reg = MacroRegistry::new();
    register_builtins(env.clone());
    register_stdlib(env.clone());

    // Register special forms documentation
    eval::register_special_forms_part1();
    eval::register_special_forms_part2();

    // Register stdlib function documentation with proper categorization
    register_stdlib_functions();

    // Set environment for help system to enable lookup of user-defined functions
    help::set_current_env(Some(env.clone()));

    // Conditionally load standard library modules
    if !args.no_stdlib {
        // Load stdlib modules in order: core, math, string, test, http
        let modules = [
            ("core", include_str!("stdlib/lisp/core.lisp")),
            ("math", include_str!("stdlib/lisp/math.lisp")),
            ("string", include_str!("stdlib/lisp/string.lisp")),
            ("test", include_str!("stdlib/lisp/test.lisp")),
            ("http", include_str!("stdlib/lisp/http.lisp")),
        ];

        // Skip automatic help registration during stdlib loading
        // Stdlib functions will be registered with proper categorization by stdlib_registry
        parser::set_skip_help_registration(true);

        for (module_name, module_code) in &modules {
            match load_stdlib(module_code, env.clone(), &mut macro_reg) {
                Ok(_) => {} // Silently succeed
                Err(e) => eprintln!(
                    "Warning: Failed to load stdlib module {}: {}",
                    module_name, e
                ),
            }
        }

        // Re-enable help registration for user code
        parser::set_skip_help_registration(false);
    }

    // Check if we're running a script file or REPL
    if let Some(script_path) = args.script {
        // Script mode: execute file and exit
        run_script(&script_path, env, &mut macro_reg)?;
        return Ok(());
    }

    // REPL mode: interactive loop
    // Create REPL with history and syntax highlighting support
    let config = Config::builder().auto_add_history(true).build();
    let mut rl =
        Editor::with_config(config).map_err(|e| format!("Failed to initialize REPL: {}", e))?;

    // Set the helper with syntax highlighting
    let helper = LispHelper::new();
    rl.set_helper(Some(helper));

    // Try to load history from previous sessions
    let history_file = ".lisp_history";
    let _ = rl.load_history(history_file);

    // Print welcome message
    println!("{}", WELCOME_MESSAGE);
    println!("{}", WELCOME_SUBTITLE);
    println!("{}", WELCOME_FOOTER);

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
                    _ => {}
                }

                // Parse and evaluate the expression
                match parse(&line) {
                    Ok(expr) => {
                        // Set environment for help system lookup
                        crate::help::set_current_env(Some(env.clone()));
                        match eval_with_macros(expr, env.clone(), &mut macro_reg) {
                            Ok(result) => {
                                println!("=> {}", LispHelper::highlight_output(&result));
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                        }
                    }
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

/// Build filesystem configuration from CLI arguments
fn build_fs_config(args: &CliArgs) -> FsConfig {
    let allowed_paths = if args.fs_paths.is_empty() {
        // Use default paths if none specified
        vec![
            PathBuf::from("./data"),
            PathBuf::from("./examples"),
            PathBuf::from("./scripts"),
        ]
    } else {
        args.fs_paths.clone()
    };

    FsConfig {
        allowed_paths,
        max_file_size: args.max_file_size,
    }
}

/// Build network configuration from CLI arguments
fn build_net_config(args: &CliArgs) -> NetConfig {
    NetConfig {
        enabled: args.allow_network,
        allowed_addresses: args.net_addresses.clone(),
    }
}

/// Execute a Lisp script file
fn run_script(
    path: &PathBuf,
    env: Rc<Environment>,
    macro_reg: &mut MacroRegistry,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read script file (script files are trusted input, not sandboxed)
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read script file {}: {}", path.display(), e))?;

    // Parse and evaluate all expressions in the script
    let mut remaining = contents.trim();

    while !remaining.is_empty() {
        // Skip whitespace and regular comments (preserves ;;; doc comments)
        remaining = skip_whitespace_and_regular_comments(remaining);
        if remaining.is_empty() {
            break;
        }

        // Parse one expression
        match parse_one_expr(remaining) {
            Ok((expr, rest)) => {
                // Set environment for help system lookup
                crate::help::set_current_env(Some(env.clone()));
                // Evaluate the expression
                match eval_with_macros(expr, env.clone(), macro_reg) {
                    Ok(_result) => {
                        // Scripts typically don't print results unless explicitly printed
                        remaining = rest;
                    }
                    Err(e) => {
                        return Err(format!("Evaluation error: {}", e).into());
                    }
                }
            }
            Err(e) => {
                return Err(format!("Parse error: {}", e).into());
            }
        }
    }

    Ok(())
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
        // Skip whitespace and regular comments (preserves ;;; doc comments)
        remaining = skip_whitespace_and_regular_comments(remaining);
        if remaining.is_empty() {
            break;
        }

        // Parse one expression
        match parse_one_expr(remaining) {
            Ok((expr, rest)) => {
                // Set environment for help system lookup
                crate::help::set_current_env(Some(env.clone()));
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

/// Skip whitespace and NON-DOC comments in the input string
/// Preserves ;;; doc comments so they can be captured by parse()
fn skip_whitespace_and_regular_comments(input: &str) -> &str {
    let mut remaining = input;
    loop {
        remaining = remaining.trim_start();
        // Skip only ; and ;; comments, NOT ;;; doc comments
        if remaining.starts_with(";;;") {
            // Don't skip doc comments!
            break;
        } else if remaining.starts_with(";;") || remaining.starts_with(';') {
            // Skip regular comments
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
///
/// This function preserves ;;; doc comments and includes them in the parsed string
/// so that parse() can capture them via the thread-local PENDING_DOCS.
fn parse_one_expr(input: &str) -> Result<(value::Value, &str), String> {
    // Skip only non-doc comments and whitespace, preserve ;;; comments
    let start = skip_whitespace_and_regular_comments(input);
    if start.is_empty() {
        return Err("No expression to parse".to_string());
    }

    // Collect any preceding ;;; doc comments
    let mut doc_start = start;
    while doc_start.starts_with(";;;") {
        // Find the end of this doc comment line
        if let Some(pos) = doc_start.find('\n') {
            doc_start = &doc_start[pos + 1..];
            doc_start = skip_whitespace_and_regular_comments(doc_start);
        } else {
            break;
        }
    }

    // Find the end of the first complete s-expression (after the doc comments)
    let end_pos = find_expr_end(doc_start)?;

    // Include everything from the start of doc comments to the end of the expression
    let bytes_from_start = start.len() - doc_start.len();
    let total_expr_len = bytes_from_start + end_pos;
    let expr_str = &start[..total_expr_len];
    let rest = &start[total_expr_len..];

    // Parse the expression (this will capture ;;; comments via thread-local)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_fs_config_with_defaults() {
        let args = CliArgs {
            script: None,
            fs_paths: vec![],
            max_file_size: 10485760,
            allow_network: false,
            net_addresses: vec![],
            no_stdlib: false,
        };
        let config = build_fs_config(&args);
        assert_eq!(config.allowed_paths.len(), 3);
        assert_eq!(config.max_file_size, 10485760);
        assert_eq!(config.allowed_paths[0], PathBuf::from("./data"));
        assert_eq!(config.allowed_paths[1], PathBuf::from("./examples"));
        assert_eq!(config.allowed_paths[2], PathBuf::from("./scripts"));
    }

    #[test]
    fn test_build_fs_config_with_custom_paths() {
        let args = CliArgs {
            script: None,
            fs_paths: vec![PathBuf::from("/tmp/safe")],
            max_file_size: 5242880,
            allow_network: false,
            net_addresses: vec![],
            no_stdlib: false,
        };
        let config = build_fs_config(&args);
        assert_eq!(config.allowed_paths.len(), 1);
        assert_eq!(config.allowed_paths[0], PathBuf::from("/tmp/safe"));
        assert_eq!(config.max_file_size, 5242880);
    }

    #[test]
    fn test_build_fs_config_with_multiple_paths() {
        let args = CliArgs {
            script: None,
            fs_paths: vec![
                PathBuf::from("./data"),
                PathBuf::from("./uploads"),
                PathBuf::from("/tmp"),
            ],
            max_file_size: 1048576,
            allow_network: false,
            net_addresses: vec![],
            no_stdlib: false,
        };
        let config = build_fs_config(&args);
        assert_eq!(config.allowed_paths.len(), 3);
        assert_eq!(config.max_file_size, 1048576);
    }

    #[test]
    fn test_build_net_config_disabled_by_default() {
        let args = CliArgs {
            script: None,
            fs_paths: vec![],
            max_file_size: 10485760,
            allow_network: false,
            net_addresses: vec![],
            no_stdlib: false,
        };
        let config = build_net_config(&args);
        assert!(!config.enabled);
        assert_eq!(config.allowed_addresses.len(), 0);
    }

    #[test]
    fn test_build_net_config_enabled() {
        let args = CliArgs {
            script: None,
            fs_paths: vec![],
            max_file_size: 10485760,
            allow_network: true,
            net_addresses: vec![],
            no_stdlib: false,
        };
        let config = build_net_config(&args);
        assert!(config.enabled);
        assert_eq!(config.allowed_addresses.len(), 0);
    }

    #[test]
    fn test_build_net_config_with_allowlist() {
        let args = CliArgs {
            script: None,
            fs_paths: vec![],
            max_file_size: 10485760,
            allow_network: true,
            net_addresses: vec!["example.com".to_string(), "api.local:8080".to_string()],
            no_stdlib: false,
        };
        let config = build_net_config(&args);
        assert!(config.enabled);
        assert_eq!(config.allowed_addresses.len(), 2);
        assert_eq!(config.allowed_addresses[0], "example.com");
        assert_eq!(config.allowed_addresses[1], "api.local:8080");
    }

    #[test]
    fn test_cli_args_script_argument() {
        let args = CliArgs {
            script: Some(PathBuf::from("test.lisp")),
            fs_paths: vec![],
            max_file_size: 10485760,
            allow_network: false,
            net_addresses: vec![],
            no_stdlib: false,
        };
        assert!(args.script.is_some());
        assert_eq!(args.script.as_ref().unwrap(), &PathBuf::from("test.lisp"));
    }

    #[test]
    fn test_cli_args_no_stdlib_flag() {
        let args = CliArgs {
            script: None,
            fs_paths: vec![],
            max_file_size: 10485760,
            allow_network: false,
            net_addresses: vec![],
            no_stdlib: true,
        };
        assert!(args.no_stdlib);
    }
}

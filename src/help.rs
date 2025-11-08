// ABOUTME: Help and documentation system for the Lisp interpreter
// Provides first-class documentation for built-in and user-defined functions

use std::cell::RefCell;
use std::collections::HashMap;

/// A help entry for a function
#[derive(Debug, Clone)]
pub struct HelpEntry {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub examples: Vec<String>,
    pub related: Vec<String>,
    pub category: String,
}

/// Macro for defining help entries with less boilerplate
/// Usage: help_entry!("name", "category", "signature", "description", ["ex1", "ex2"], ["related1"])
macro_rules! help_entry {
    (
        $name:literal,
        $category:literal,
        $signature:literal,
        $description:literal,
        [$($example:literal),* $(,)?],
        [$($related:literal),* $(,)?]
    ) => {
        register_help(HelpEntry {
            name: $name.to_string(),
            category: $category.to_string(),
            signature: $signature.to_string(),
            description: $description.trim().to_string(),
            examples: vec![$($example.to_string()),*],
            related: vec![$($related.to_string()),*],
        });
    };
}

/// Registry for all function documentation
pub struct HelpRegistry {
    entries: HashMap<String, HelpEntry>,
}

impl HelpRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Register a help entry
    pub fn register(&mut self, entry: HelpEntry) {
        self.entries.insert(entry.name.clone(), entry);
    }

    /// Get a help entry by name
    pub fn get(&self, name: &str) -> Option<HelpEntry> {
        self.entries.get(name).cloned()
    }

    /// Get all entries organized by category
    pub fn by_category(&self) -> HashMap<String, Vec<HelpEntry>> {
        let mut by_cat: HashMap<String, Vec<HelpEntry>> = HashMap::new();
        for entry in self.entries.values() {
            by_cat
                .entry(entry.category.clone())
                .or_default()
                .push(entry.clone());
        }
        // Sort each category
        for entries in by_cat.values_mut() {
            entries.sort_by(|a, b| a.name.cmp(&b.name));
        }
        by_cat
    }

    /// Get all function names
    #[allow(dead_code)]
    pub fn all_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.entries.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for HelpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-local help registry
thread_local! {
    static HELP_REGISTRY: RefCell<HelpRegistry> = RefCell::new(HelpRegistry::new());
}

/// Register a help entry in the global registry
pub fn register_help(entry: HelpEntry) {
    HELP_REGISTRY.with(|reg| {
        reg.borrow_mut().register(entry);
    });
}

/// Get a help entry by name
pub fn get_help(name: &str) -> Option<HelpEntry> {
    HELP_REGISTRY.with(|reg| reg.borrow().get(name))
}

/// Get all entries organized by category
pub fn all_by_category() -> HashMap<String, Vec<HelpEntry>> {
    HELP_REGISTRY.with(|reg| reg.borrow().by_category())
}

/// Get all function names
#[allow(dead_code)]
pub fn all_names() -> Vec<String> {
    HELP_REGISTRY.with(|reg| reg.borrow().all_names())
}

/// Format a single help entry for display with syntax highlighting
pub fn format_help_entry(entry: &HelpEntry) -> String {
    let mut output = String::new();

    // Header with name and category
    output.push_str(&format!("{} - {}\n", entry.name, entry.category));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Signature - split multi-line signatures nicely
    output.push_str("Signature:\n");
    for sig_line in entry.signature.lines() {
        output.push_str(&format!("  {}\n", sig_line));
    }
    output.push('\n');

    // Description
    output.push_str("Description:\n");
    for line in entry.description.lines() {
        output.push_str(&format!("  {}\n", line));
    }
    output.push('\n');

    // Examples with better formatting
    if !entry.examples.is_empty() {
        output.push_str("Examples:\n");
        for example in &entry.examples {
            output.push_str("  ");
            output.push_str(example);
            output.push('\n');
        }
        output.push('\n');
    }

    // Related functions
    if !entry.related.is_empty() {
        output.push_str("Related:\n");
        output.push_str(&format!("  {}\n", entry.related.join(", ")));
        output.push('\n');
    }

    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    output
}

/// Format quick reference showing all functions
pub fn format_quick_reference() -> String {
    let mut output = String::new();

    let by_cat = all_by_category();
    let total = by_cat.values().map(|v| v.len()).sum::<usize>();

    output.push_str(&format!("Available Functions ({} total)\n", total));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    // Define category display order
    let categories = vec![
        "Special Forms",
        "Arithmetic",
        "Comparison",
        "Logic",
        "List operations",
        "Type predicates",
        "Console I/O",
        "Filesystem I/O",
        "Network I/O",
        "Error handling",
        "Help system",
    ];

    for category in categories {
        if let Some(entries) = by_cat.get(category) {
            let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
            output.push_str(&format!("{} ({})\n", category, names.len()));
            output.push_str(&format!("  {}\n\n", names.join(", ")));
        }
    }

    output.push_str("Type (help 'function-name) for detailed help.\n");
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    output
}

/// Populate the registry with all built-in function documentation
pub fn populate_builtin_help() {
    // Arithmetic
    help_entry!("+", "Arithmetic", "(+ num1 num2 ...)", "Returns the sum of all arguments.",
        ["(+ 1 2 3) => 6", "(+ 10) => 10", "(+) => 0"], ["-", "*", "/"]);
    help_entry!("-", "Arithmetic", "(- num1 num2 ...)",
        "Subtracts subsequent arguments from the first.\nWith one argument, returns its negation.",
        ["(- 10 3 2) => 5", "(- 5) => -5"], ["+", "*", "/"]);
    help_entry!("*", "Arithmetic", "(* num1 num2 ...)", "Returns the product of all arguments.",
        ["(* 2 3 4) => 24", "(* 5) => 5", "(*) => 1"], ["+", "-", "/"]);
    help_entry!("/", "Arithmetic", "(/ num1 num2 ...)",
        "Divides the first argument by subsequent arguments.\nInteger division in Lisp.",
        ["(/ 20 4) => 5", "(/ 100 2 5) => 10"], ["+", "-", "*", "%"]);
    help_entry!("%", "Arithmetic", "(% num1 num2)", "Returns the remainder when num1 is divided by num2.",
        ["(% 17 5) => 2", "(% 10 3) => 1"], ["/"]);

    // Comparison
    help_entry!("=", "Comparison", "(= val1 val2 ...)",
        "Tests if all arguments are equal. Works with numbers, strings, symbols.",
        ["(= 5 5) => #t", "(= 5 5 5) => #t", "(= 5 6) => #f", "(= \"hello\" \"hello\") => #t"],
        ["<", ">", "<=", ">="]);
    help_entry!("<", "Comparison", "(< num1 num2 ...)", "Tests if each argument is strictly less than the next.",
        ["(< 1 2 3) => #t", "(< 1 1) => #f", "(< 5 3) => #f"], [">", "<=", ">=", "="]);
    help_entry!(">", "Comparison", "(> num1 num2 ...)", "Tests if each argument is strictly greater than the next.",
        ["(> 3 2 1) => #t", "(> 3 3) => #f"], ["<", "<=", ">=", "="]);
    help_entry!("<=", "Comparison", "(<= num1 num2 ...)", "Tests if each argument is less than or equal to the next.",
        ["(<= 1 2 2 3) => #t", "(<= 5 5) => #t"], ["<", ">", ">=", "="]);
    help_entry!(">=", "Comparison", "(>= num1 num2 ...)", "Tests if each argument is greater than or equal to the next.",
        ["(>= 3 2 2 1) => #t", "(>= 5 5) => #t"], ["<", ">", "<=", "="]);

    // Logic
    help_entry!("and", "Logic", "(and val1 val2 ...)",
        "Logical AND. Returns #f if any argument is falsy, otherwise returns the last argument.\nShort-circuits: stops evaluating after first falsy value.",
        ["(and #t #t #t) => #t", "(and #t #f #t) => #f", "(and 1 2 3) => 3"], ["or", "not"]);
    help_entry!("or", "Logic", "(or val1 val2 ...)",
        "Logical OR. Returns the first truthy value or #f if all are falsy.\nShort-circuits: stops evaluating after first truthy value.",
        ["(or #f #f #t) => #t", "(or #f #f) => #f", "(or nil 2) => 2"], ["and", "not"]);
    help_entry!("not", "Logic", "(not val)", "Logical NOT. Returns #t if val is falsy (#f or nil), otherwise #f.",
        ["(not #f) => #t", "(not #t) => #f", "(not nil) => #t", "(not 5) => #f"], ["and", "or"]);

    // List operations
    help_entry!("cons", "List operations", "(cons elem list)",
        "Constructs a new list by prepending elem to list.\nReturns a new list; original is not modified.",
        ["(cons 1 '(2 3)) => (1 2 3)", "(cons 'a '(b c)) => (a b c)", "(cons 1 nil) => (1)"],
        ["car", "cdr", "list"]);
    help_entry!("car", "List operations", "(car list)",
        "Returns the first element of a list. Also called 'head'.\nThrows error on empty list or non-list.",
        ["(car '(1 2 3)) => 1", "(car '(a)) => a"], ["cdr", "cons", "nth"]);
    help_entry!("cdr", "List operations", "(cdr list)",
        "Returns all elements except the first. Also called 'tail'.\nReturns nil for single-element list.",
        ["(cdr '(1 2 3)) => (2 3)", "(cdr '(a b)) => (b)", "(cdr '(1)) => nil"], ["car", "cons"]);
    help_entry!("list", "List operations", "(list elem1 elem2 ...)",
        "Creates a new list containing the given elements in order.",
        ["(list 1 2 3) => (1 2 3)", "(list 'a 'b) => (a b)", "(list) => nil"],
        ["cons", "car", "cdr"]);
    help_entry!("length", "List operations", "(length list)", "Returns the number of elements in a list.",
        ["(length '(1 2 3)) => 3", "(length '()) => 0", "(length '(a)) => 1"], ["empty?", "list"]);
    help_entry!("empty?", "List operations", "(empty? list)",
        "Tests if a list is empty (nil or ()).\nReturns #t for empty lists, #f otherwise.",
        ["(empty? nil) => #t", "(empty? '()) => #t", "(empty? '(1)) => #f"], ["length", "nil?"]);

    // Type predicates
    help_entry!("number?", "Type predicates", "(number? val)", "Tests if val is a number (integer or float).",
        ["(number? 42) => #t", "(number? 3.14) => #t", "(number? \"42\") => #f"],
        ["string?", "symbol?", "list?"]);
    help_entry!("string?", "Type predicates", "(string? val)", "Tests if val is a string.",
        ["(string? \"hello\") => #t", "(string? 42) => #f", "(string? 'hello) => #f"],
        ["number?", "symbol?"]);
    help_entry!("list?", "Type predicates", "(list? val)", "Tests if val is a list (including nil).",
        ["(list? '(1 2 3)) => #t", "(list? nil) => #t", "(list? 42) => #f"],
        ["number?", "string?", "nil?"]);
    help_entry!("nil?", "Type predicates", "(nil? val)", "Tests if val is nil (empty list).",
        ["(nil? nil) => #t", "(nil? '()) => #t", "(nil? 0) => #f"], ["empty?", "list?"]);
    help_entry!("symbol?", "Type predicates", "(symbol? val)", "Tests if val is a symbol (e.g., from 'hello or var names).",
        ["(symbol? 'hello) => #t", "(symbol? \"hello\") => #f", "(symbol? hello) => error (undefined variable)"],
        ["string?", "number?"]);
    help_entry!("bool?", "Type predicates", "(bool? val)", "Tests if val is a boolean (#t or #f).",
        ["(bool? #t) => #t", "(bool? #f) => #t", "(bool? 1) => #f"], ["number?", "string?"]);

    // Console I/O
    help_entry!("print", "Console I/O", "(print val1 val2 ...)", "Prints values to stdout without newline. Returns nil.",
        ["(print \"hello\") => outputs: hello", "(print 1 2 3) => outputs: 1 2 3"], ["println"]);
    help_entry!("println", "Console I/O", "(println val1 val2 ...)", "Prints values to stdout with newline at end. Returns nil.",
        ["(println \"hello\") => outputs: hello", "(println \"a\" \"b\") => outputs: a b"], ["print"]);

    // Filesystem I/O
    help_entry!("read-file", "Filesystem I/O", "(read-file path)",
        "Reads and returns the contents of a file as a string.\nPath is relative to allowed sandbox directories.",
        ["(read-file \"data/input.txt\") => \"file contents\""], ["write-file", "file-exists?"]);
    help_entry!("write-file", "Filesystem I/O", "(write-file path contents)",
        "Writes contents to a file, creating it if it doesn't exist.\nReturns #t on success. Path is relative to sandbox.",
        ["(write-file \"data/output.txt\" \"hello\") => #t"], ["read-file", "file-exists?"]);
    help_entry!("file-exists?", "Filesystem I/O", "(file-exists? path)",
        "Tests if a file exists and is accessible in sandbox.\nReturns #t or #f.",
        ["(file-exists? \"data/file.txt\") => #t", "(file-exists? \"nonexistent.txt\") => #f"],
        ["file-size", "read-file"]);
    help_entry!("file-size", "Filesystem I/O", "(file-size path)",
        "Returns the size of a file in bytes.\nThrows error if file doesn't exist.",
        ["(file-size \"data/file.txt\") => 1024"], ["file-exists?", "read-file"]);
    help_entry!("list-files", "Filesystem I/O", "(list-files directory)",
        "Returns a list of filenames in a directory.\nDoes not include . or .., returns only names not full paths.",
        ["(list-files \"data\") => (\"file1.txt\" \"file2.txt\")"], ["file-exists?"]);

    // Network I/O
    help_entry!("http-get", "Network I/O", "(http-get url)",
        "Performs an HTTP GET request and returns the response body as a string.\nURL must be in allowed addresses list. Has 30 second timeout.\nWARNING: DNS lookup cannot be interrupted, may hang if DNS is slow.",
        ["(http-get \"https://example.com\") => \"<html>...\""], ["http-post"]);
    help_entry!("http-post", "Network I/O", "(http-post url body)",
        "Performs an HTTP POST request and returns the response body as a string.\nURL must be in allowed addresses. Sends body as plain text. 30 second timeout.\nWARNING: DNS lookup cannot be interrupted, may hang if DNS is slow.",
        ["(http-post \"https://api.example.com\" \"data\") => \"response\""], ["http-get"]);

    // Error handling
    help_entry!("error", "Error handling", "(error message)", "Raises an error with the given message. Always throws.",
        ["(error \"invalid input\") => Error: invalid input"], ["error?", "error-msg"]);
    help_entry!("error?", "Error handling", "(error? val)", "Tests if val is an error value.",
        ["(error? (error \"test\")) => would throw before testing"], ["error", "error-msg"]);
    help_entry!("error-msg", "Error handling", "(error-msg error)", "Extracts the message from an error value.",
        ["(error-msg (error \"test\")) => would throw before extracting"], ["error", "error?"]);

    // Help system
    help_entry!("help", "Help system", "(help) or (help 'function-name)",
        "Show help information. With no arguments, displays quick reference.\nWith a function name, shows detailed documentation for that function.",
        ["(help) => shows quick reference", "(help 'cons) => detailed help for cons", "(help 'map) => help for user or stdlib function"],
        ["doc"]);
    help_entry!("doc", "Help system", "(doc function)",
        "Returns the docstring of a function as a string.\nWorks with user-defined functions that have docstrings.",
        ["(doc factorial) => \"Computes factorial\""], ["help"]);

    // Special Forms
    help_entry!("define", "Special Forms", "(define name value)\n(define (name param1 param2 ...) body)",
        "Binds a name to a value in the current scope.\nCan define variables or functions with the shorthand syntax.",
        ["(define x 42)", "(define (square x) (* x x))", "(square 5) => 25"], ["let", "lambda"]);
    help_entry!("lambda", "Special Forms", "(lambda (param1 param2 ...) body)\n(lambda (param1 param2 ...) \"docstring\" body)",
        "Creates an anonymous function that captures the current lexical environment.\nOptional docstring available via help system.",
        ["((lambda (x) (* x x)) 5) => 25", "(define square (lambda (x) (* x x)))", "(map (lambda (x) (+ x 1)) '(1 2 3)) => (2 3 4)"],
        ["define", "let"]);
    help_entry!("if", "Special Forms", "(if condition then-expr else-expr)",
        "Conditional evaluation. Evaluates then-expr if condition is truthy, else-expr otherwise.\n#f and nil are falsy; everything else is truthy.",
        ["(if #t 1 2) => 1", "(if (= 5 5) \"yes\" \"no\") => \"yes\"", "(if #f 1 2) => 2"],
        ["begin", "and", "or"]);
    help_entry!("begin", "Special Forms", "(begin expr1 expr2 ...)",
        "Evaluates expressions in sequence and returns the value of the last one.\nUsed to group multiple expressions where only one is expected.",
        ["(begin (define x 1) (+ x 1)) => 2", "(begin 1 2 3) => 3"], ["if", "let"]);
    help_entry!("let", "Special Forms", "(let ((var1 val1) (var2 val2) ...) body)",
        "Creates local bindings for expressions.\nVariables are bound simultaneously (all right-side expressions use outer scope).\nBody has access to all bindings.",
        ["(let ((x 1) (y 2)) (+ x y)) => 3", "(let ((x 5)) (* x x)) => 25"], ["define", "lambda"]);
    help_entry!("quote", "Special Forms", "'expr or (quote expr)",
        "Returns the expression unevaluated as a data structure.\nUseful for working with code as data and creating lists.",
        ["'(1 2 3) => (1 2 3)", "'hello => hello", "(quote (+ 1 2)) => (+ 1 2)"], ["quasiquote"]);
    help_entry!("quasiquote", "Special Forms", "`expr or (quasiquote expr)",
        "Like quote, but allows selective evaluation using unquote (,) and unquote-splicing (,@).\nPrimarily used for writing macros.",
        ["`(1 2 3) => (1 2 3)", "`(1 ,(+ 1 1) 3) => (1 2 3)"], ["quote", "unquote", "defmacro"]);
    help_entry!("unquote", "Special Forms", ",expr (inside a quasiquote)",
        "Evaluates an expression inside a quasiquote context.\nOnly meaningful within a quasiquote.",
        ["`(1 ,(+ 1 1) 3) => (1 2 3)"], ["quasiquote", "unquote-splicing"]);
    help_entry!("unquote-splicing", "Special Forms", ",@expr (inside a quasiquote)",
        "Evaluates an expression and splices its elements into the surrounding list.\nOnly meaningful within a quasiquote context.",
        ["`(1 ,@'(2 3) 4) => (1 2 3 4)"], ["quasiquote", "unquote"]);
    help_entry!("defmacro", "Special Forms", "(defmacro (name param1 param2 ...) body)",
        "Defines a macro that transforms code at compile-time.\nMacros receive unevaluated arguments and return code to be evaluated.\nUse with quasiquote and unquote for code generation.",
        ["(defmacro (unless cond body) `(if (not ,cond) ,body))"], ["lambda", "quasiquote"]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_registry_register_and_get() {
        let mut registry = HelpRegistry::new();
        let entry = HelpEntry {
            name: "test-fn".to_string(),
            signature: "(test-fn x)".to_string(),
            description: "Test function".to_string(),
            examples: vec![],
            related: vec![],
            category: "Test".to_string(),
        };

        registry.register(entry.clone());
        assert_eq!(registry.get("test-fn").unwrap().name, "test-fn");
    }

    #[test]
    fn test_help_registry_by_category() {
        let mut registry = HelpRegistry::new();
        registry.register(HelpEntry {
            name: "fn1".to_string(),
            signature: "".to_string(),
            description: "".to_string(),
            examples: vec![],
            related: vec![],
            category: "Arithmetic".to_string(),
        });
        registry.register(HelpEntry {
            name: "fn2".to_string(),
            signature: "".to_string(),
            description: "".to_string(),
            examples: vec![],
            related: vec![],
            category: "Arithmetic".to_string(),
        });

        let by_cat = registry.by_category();
        assert_eq!(by_cat["Arithmetic"].len(), 2);
    }

    #[test]
    fn test_format_help_entry() {
        let entry = HelpEntry {
            name: "test".to_string(),
            signature: "(test x)".to_string(),
            description: "A test function".to_string(),
            examples: vec!["(test 1)".to_string()],
            related: vec!["other".to_string()],
            category: "Test".to_string(),
        };

        let formatted = format_help_entry(&entry);
        assert!(formatted.contains("test - Test"));
        assert!(formatted.contains("A test function"));
        assert!(formatted.contains("(test 1)"));
    }

    #[test]
    fn test_populate_builtin_help() {
        populate_builtin_help();
        assert!(get_help("cons").is_some());
        assert!(get_help("+").is_some());
        assert!(get_help("read-file").is_some());
        assert!(get_help("help").is_some());
    }
}

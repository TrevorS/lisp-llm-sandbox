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

/// Format a single help entry for display
pub fn format_help_entry(entry: &HelpEntry) -> String {
    let mut output = String::new();

    // Header with name and category
    output.push_str(&format!("{} - {}\n", entry.name, entry.category));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Signature
    output.push_str("Signature:\n");
    output.push_str(&format!("  {}\n\n", entry.signature));

    // Description
    output.push_str("Description:\n");
    for line in entry.description.lines() {
        output.push_str(&format!("  {}\n", line));
    }
    output.push('\n');

    // Examples
    if !entry.examples.is_empty() {
        output.push_str("Examples:\n");
        for example in &entry.examples {
            output.push_str(&format!("  {}\n", example));
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
    register_help(HelpEntry {
        name: "+".to_string(),
        signature: "(+ num1 num2 ...)".to_string(),
        description: "Returns the sum of all arguments.".to_string(),
        examples: vec![
            "(+ 1 2 3) => 6".to_string(),
            "(+ 10) => 10".to_string(),
            "(+) => 0".to_string(),
        ],
        related: vec!["-".to_string(), "*".to_string(), "/".to_string()],
        category: "Arithmetic".to_string(),
    });

    register_help(HelpEntry {
        name: "-".to_string(),
        signature: "(- num1 num2 ...)".to_string(),
        description: "Subtracts subsequent arguments from the first.\nWith one argument, returns its negation.".to_string(),
        examples: vec![
            "(- 10 3 2) => 5".to_string(),
            "(- 5) => -5".to_string(),
        ],
        related: vec!["+".to_string(), "*".to_string(), "/".to_string()],
        category: "Arithmetic".to_string(),
    });

    register_help(HelpEntry {
        name: "*".to_string(),
        signature: "(* num1 num2 ...)".to_string(),
        description: "Returns the product of all arguments.".to_string(),
        examples: vec![
            "(* 2 3 4) => 24".to_string(),
            "(* 5) => 5".to_string(),
            "(*) => 1".to_string(),
        ],
        related: vec!["+".to_string(), "-".to_string(), "/".to_string()],
        category: "Arithmetic".to_string(),
    });

    register_help(HelpEntry {
        name: "/".to_string(),
        signature: "(/ num1 num2 ...)".to_string(),
        description:
            "Divides the first argument by subsequent arguments.\nInteger division in Lisp."
                .to_string(),
        examples: vec!["(/ 20 4) => 5".to_string(), "(/ 100 2 5) => 10".to_string()],
        related: vec![
            "+".to_string(),
            "-".to_string(),
            "*".to_string(),
            "%".to_string(),
        ],
        category: "Arithmetic".to_string(),
    });

    register_help(HelpEntry {
        name: "%".to_string(),
        signature: "(% num1 num2)".to_string(),
        description: "Returns the remainder when num1 is divided by num2.".to_string(),
        examples: vec!["(% 17 5) => 2".to_string(), "(% 10 3) => 1".to_string()],
        related: vec!["/".to_string()],
        category: "Arithmetic".to_string(),
    });

    // Comparison
    register_help(HelpEntry {
        name: "=".to_string(),
        signature: "(= val1 val2 ...)".to_string(),
        description: "Tests if all arguments are equal. Works with numbers, strings, symbols."
            .to_string(),
        examples: vec![
            "(= 5 5) => #t".to_string(),
            "(= 5 5 5) => #t".to_string(),
            "(= 5 6) => #f".to_string(),
            "(= \"hello\" \"hello\") => #t".to_string(),
        ],
        related: vec![
            "<".to_string(),
            ">".to_string(),
            "<=".to_string(),
            ">=".to_string(),
        ],
        category: "Comparison".to_string(),
    });

    register_help(HelpEntry {
        name: "<".to_string(),
        signature: "(< num1 num2 ...)".to_string(),
        description: "Tests if each argument is strictly less than the next.".to_string(),
        examples: vec![
            "(< 1 2 3) => #t".to_string(),
            "(< 1 1) => #f".to_string(),
            "(< 5 3) => #f".to_string(),
        ],
        related: vec![
            ">".to_string(),
            "<=".to_string(),
            ">=".to_string(),
            "=".to_string(),
        ],
        category: "Comparison".to_string(),
    });

    register_help(HelpEntry {
        name: ">".to_string(),
        signature: "(> num1 num2 ...)".to_string(),
        description: "Tests if each argument is strictly greater than the next.".to_string(),
        examples: vec!["(> 3 2 1) => #t".to_string(), "(> 3 3) => #f".to_string()],
        related: vec![
            "<".to_string(),
            "<=".to_string(),
            ">=".to_string(),
            "=".to_string(),
        ],
        category: "Comparison".to_string(),
    });

    register_help(HelpEntry {
        name: "<=".to_string(),
        signature: "(<= num1 num2 ...)".to_string(),
        description: "Tests if each argument is less than or equal to the next.".to_string(),
        examples: vec![
            "(<= 1 2 2 3) => #t".to_string(),
            "(<= 5 5) => #t".to_string(),
        ],
        related: vec![
            "<".to_string(),
            ">".to_string(),
            ">=".to_string(),
            "=".to_string(),
        ],
        category: "Comparison".to_string(),
    });

    register_help(HelpEntry {
        name: ">=".to_string(),
        signature: "(>= num1 num2 ...)".to_string(),
        description: "Tests if each argument is greater than or equal to the next.".to_string(),
        examples: vec![
            "(>= 3 2 2 1) => #t".to_string(),
            "(>= 5 5) => #t".to_string(),
        ],
        related: vec![
            "<".to_string(),
            ">".to_string(),
            "<=".to_string(),
            "=".to_string(),
        ],
        category: "Comparison".to_string(),
    });

    // Logic
    register_help(HelpEntry {
        name: "and".to_string(),
        signature: "(and val1 val2 ...)".to_string(),
        description: "Logical AND. Returns #f if any argument is falsy, otherwise returns the last argument.\nShort-circuits: stops evaluating after first falsy value.".to_string(),
        examples: vec![
            "(and #t #t #t) => #t".to_string(),
            "(and #t #f #t) => #f".to_string(),
            "(and 1 2 3) => 3".to_string(),
        ],
        related: vec!["or".to_string(), "not".to_string()],
        category: "Logic".to_string(),
    });

    register_help(HelpEntry {
        name: "or".to_string(),
        signature: "(or val1 val2 ...)".to_string(),
        description: "Logical OR. Returns the first truthy value or #f if all are falsy.\nShort-circuits: stops evaluating after first truthy value.".to_string(),
        examples: vec![
            "(or #f #f #t) => #t".to_string(),
            "(or #f #f) => #f".to_string(),
            "(or nil 2) => 2".to_string(),
        ],
        related: vec!["and".to_string(), "not".to_string()],
        category: "Logic".to_string(),
    });

    register_help(HelpEntry {
        name: "not".to_string(),
        signature: "(not val)".to_string(),
        description: "Logical NOT. Returns #t if val is falsy (#f or nil), otherwise #f."
            .to_string(),
        examples: vec![
            "(not #f) => #t".to_string(),
            "(not #t) => #f".to_string(),
            "(not nil) => #t".to_string(),
            "(not 5) => #f".to_string(),
        ],
        related: vec!["and".to_string(), "or".to_string()],
        category: "Logic".to_string(),
    });

    // List operations
    register_help(HelpEntry {
        name: "cons".to_string(),
        signature: "(cons elem list)".to_string(),
        description: "Constructs a new list by prepending elem to list.\nReturns a new list; original is not modified.".to_string(),
        examples: vec![
            "(cons 1 '(2 3)) => (1 2 3)".to_string(),
            "(cons 'a '(b c)) => (a b c)".to_string(),
            "(cons 1 nil) => (1)".to_string(),
        ],
        related: vec!["car".to_string(), "cdr".to_string(), "list".to_string()],
        category: "List operations".to_string(),
    });

    register_help(HelpEntry {
        name: "car".to_string(),
        signature: "(car list)".to_string(),
        description: "Returns the first element of a list. Also called 'head'.\nThrows error on empty list or non-list.".to_string(),
        examples: vec![
            "(car '(1 2 3)) => 1".to_string(),
            "(car '(a)) => a".to_string(),
        ],
        related: vec!["cdr".to_string(), "cons".to_string(), "nth".to_string()],
        category: "List operations".to_string(),
    });

    register_help(HelpEntry {
        name: "cdr".to_string(),
        signature: "(cdr list)".to_string(),
        description: "Returns all elements except the first. Also called 'tail'.\nReturns nil for single-element list.".to_string(),
        examples: vec![
            "(cdr '(1 2 3)) => (2 3)".to_string(),
            "(cdr '(a b)) => (b)".to_string(),
            "(cdr '(1)) => nil".to_string(),
        ],
        related: vec!["car".to_string(), "cons".to_string()],
        category: "List operations".to_string(),
    });

    register_help(HelpEntry {
        name: "list".to_string(),
        signature: "(list elem1 elem2 ...)".to_string(),
        description: "Creates a new list containing the given elements in order.".to_string(),
        examples: vec![
            "(list 1 2 3) => (1 2 3)".to_string(),
            "(list 'a 'b) => (a b)".to_string(),
            "(list) => nil".to_string(),
        ],
        related: vec!["cons".to_string(), "car".to_string(), "cdr".to_string()],
        category: "List operations".to_string(),
    });

    register_help(HelpEntry {
        name: "length".to_string(),
        signature: "(length list)".to_string(),
        description: "Returns the number of elements in a list.".to_string(),
        examples: vec![
            "(length '(1 2 3)) => 3".to_string(),
            "(length '()) => 0".to_string(),
            "(length '(a)) => 1".to_string(),
        ],
        related: vec!["empty?".to_string(), "list".to_string()],
        category: "List operations".to_string(),
    });

    register_help(HelpEntry {
        name: "empty?".to_string(),
        signature: "(empty? list)".to_string(),
        description:
            "Tests if a list is empty (nil or ()).\nReturns #t for empty lists, #f otherwise."
                .to_string(),
        examples: vec![
            "(empty? nil) => #t".to_string(),
            "(empty? '()) => #t".to_string(),
            "(empty? '(1)) => #f".to_string(),
        ],
        related: vec!["length".to_string(), "nil?".to_string()],
        category: "List operations".to_string(),
    });

    // Type predicates
    register_help(HelpEntry {
        name: "number?".to_string(),
        signature: "(number? val)".to_string(),
        description: "Tests if val is a number (integer or float).".to_string(),
        examples: vec![
            "(number? 42) => #t".to_string(),
            "(number? 3.14) => #t".to_string(),
            "(number? \"42\") => #f".to_string(),
        ],
        related: vec![
            "string?".to_string(),
            "symbol?".to_string(),
            "list?".to_string(),
        ],
        category: "Type predicates".to_string(),
    });

    register_help(HelpEntry {
        name: "string?".to_string(),
        signature: "(string? val)".to_string(),
        description: "Tests if val is a string.".to_string(),
        examples: vec![
            "(string? \"hello\") => #t".to_string(),
            "(string? 42) => #f".to_string(),
            "(string? 'hello) => #f".to_string(),
        ],
        related: vec!["number?".to_string(), "symbol?".to_string()],
        category: "Type predicates".to_string(),
    });

    register_help(HelpEntry {
        name: "list?".to_string(),
        signature: "(list? val)".to_string(),
        description: "Tests if val is a list (including nil).".to_string(),
        examples: vec![
            "(list? '(1 2 3)) => #t".to_string(),
            "(list? nil) => #t".to_string(),
            "(list? 42) => #f".to_string(),
        ],
        related: vec![
            "number?".to_string(),
            "string?".to_string(),
            "nil?".to_string(),
        ],
        category: "Type predicates".to_string(),
    });

    register_help(HelpEntry {
        name: "nil?".to_string(),
        signature: "(nil? val)".to_string(),
        description: "Tests if val is nil (empty list).".to_string(),
        examples: vec![
            "(nil? nil) => #t".to_string(),
            "(nil? '()) => #t".to_string(),
            "(nil? 0) => #f".to_string(),
        ],
        related: vec!["empty?".to_string(), "list?".to_string()],
        category: "Type predicates".to_string(),
    });

    register_help(HelpEntry {
        name: "symbol?".to_string(),
        signature: "(symbol? val)".to_string(),
        description: "Tests if val is a symbol (e.g., from 'hello or var names).".to_string(),
        examples: vec![
            "(symbol? 'hello) => #t".to_string(),
            "(symbol? \"hello\") => #f".to_string(),
            "(symbol? hello) => error (undefined variable)".to_string(),
        ],
        related: vec!["string?".to_string(), "number?".to_string()],
        category: "Type predicates".to_string(),
    });

    register_help(HelpEntry {
        name: "bool?".to_string(),
        signature: "(bool? val)".to_string(),
        description: "Tests if val is a boolean (#t or #f).".to_string(),
        examples: vec![
            "(bool? #t) => #t".to_string(),
            "(bool? #f) => #t".to_string(),
            "(bool? 1) => #f".to_string(),
        ],
        related: vec!["number?".to_string(), "string?".to_string()],
        category: "Type predicates".to_string(),
    });

    // Console I/O
    register_help(HelpEntry {
        name: "print".to_string(),
        signature: "(print val1 val2 ...)".to_string(),
        description: "Prints values to stdout without newline. Returns nil.".to_string(),
        examples: vec![
            "(print \"hello\") => outputs: hello".to_string(),
            "(print 1 2 3) => outputs: 1 2 3".to_string(),
        ],
        related: vec!["println".to_string()],
        category: "Console I/O".to_string(),
    });

    register_help(HelpEntry {
        name: "println".to_string(),
        signature: "(println val1 val2 ...)".to_string(),
        description: "Prints values to stdout with newline at end. Returns nil.".to_string(),
        examples: vec![
            "(println \"hello\") => outputs: hello".to_string(),
            "(println \"a\" \"b\") => outputs: a b".to_string(),
        ],
        related: vec!["print".to_string()],
        category: "Console I/O".to_string(),
    });

    // Filesystem I/O
    register_help(HelpEntry {
        name: "read-file".to_string(),
        signature: "(read-file path)".to_string(),
        description: "Reads and returns the contents of a file as a string.\nPath is relative to allowed sandbox directories.".to_string(),
        examples: vec![
            "(read-file \"data/input.txt\") => \"file contents\"".to_string(),
        ],
        related: vec!["write-file".to_string(), "file-exists?".to_string()],
        category: "Filesystem I/O".to_string(),
    });

    register_help(HelpEntry {
        name: "write-file".to_string(),
        signature: "(write-file path contents)".to_string(),
        description: "Writes contents to a file, creating it if it doesn't exist.\nReturns #t on success. Path is relative to sandbox.".to_string(),
        examples: vec![
            "(write-file \"data/output.txt\" \"hello\") => #t".to_string(),
        ],
        related: vec!["read-file".to_string(), "file-exists?".to_string()],
        category: "Filesystem I/O".to_string(),
    });

    register_help(HelpEntry {
        name: "file-exists?".to_string(),
        signature: "(file-exists? path)".to_string(),
        description: "Tests if a file exists and is accessible in sandbox.\nReturns #t or #f."
            .to_string(),
        examples: vec![
            "(file-exists? \"data/file.txt\") => #t".to_string(),
            "(file-exists? \"nonexistent.txt\") => #f".to_string(),
        ],
        related: vec!["file-size".to_string(), "read-file".to_string()],
        category: "Filesystem I/O".to_string(),
    });

    register_help(HelpEntry {
        name: "file-size".to_string(),
        signature: "(file-size path)".to_string(),
        description: "Returns the size of a file in bytes.\nThrows error if file doesn't exist."
            .to_string(),
        examples: vec!["(file-size \"data/file.txt\") => 1024".to_string()],
        related: vec!["file-exists?".to_string(), "read-file".to_string()],
        category: "Filesystem I/O".to_string(),
    });

    register_help(HelpEntry {
        name: "list-files".to_string(),
        signature: "(list-files directory)".to_string(),
        description: "Returns a list of filenames in a directory.\nDoes not include . or .., returns only names not full paths.".to_string(),
        examples: vec![
            "(list-files \"data\") => (\"file1.txt\" \"file2.txt\")".to_string(),
        ],
        related: vec!["file-exists?".to_string()],
        category: "Filesystem I/O".to_string(),
    });

    // Network I/O
    register_help(HelpEntry {
        name: "http-get".to_string(),
        signature: "(http-get url)".to_string(),
        description: "Performs an HTTP GET request and returns the response body as a string.\nURL must be in allowed addresses list. Has 30 second timeout.\nWARNING: DNS lookup cannot be interrupted, may hang if DNS is slow.".to_string(),
        examples: vec![
            "(http-get \"https://example.com\") => \"<html>...\"".to_string(),
        ],
        related: vec!["http-post".to_string()],
        category: "Network I/O".to_string(),
    });

    register_help(HelpEntry {
        name: "http-post".to_string(),
        signature: "(http-post url body)".to_string(),
        description: "Performs an HTTP POST request and returns the response body as a string.\nURL must be in allowed addresses. Sends body as plain text. 30 second timeout.\nWARNING: DNS lookup cannot be interrupted, may hang if DNS is slow.".to_string(),
        examples: vec![
            "(http-post \"https://api.example.com\" \"data\") => \"response\"".to_string(),
        ],
        related: vec!["http-get".to_string()],
        category: "Network I/O".to_string(),
    });

    // Error handling
    register_help(HelpEntry {
        name: "error".to_string(),
        signature: "(error message)".to_string(),
        description: "Raises an error with the given message. Always throws.".to_string(),
        examples: vec!["(error \"invalid input\") => Error: invalid input".to_string()],
        related: vec!["error?".to_string(), "error-msg".to_string()],
        category: "Error handling".to_string(),
    });

    register_help(HelpEntry {
        name: "error?".to_string(),
        signature: "(error? val)".to_string(),
        description: "Tests if val is an error value.".to_string(),
        examples: vec!["(error? (error \"test\")) => would throw before testing".to_string()],
        related: vec!["error".to_string(), "error-msg".to_string()],
        category: "Error handling".to_string(),
    });

    register_help(HelpEntry {
        name: "error-msg".to_string(),
        signature: "(error-msg error)".to_string(),
        description: "Extracts the message from an error value.".to_string(),
        examples: vec!["(error-msg (error \"test\")) => would throw before extracting".to_string()],
        related: vec!["error".to_string(), "error?".to_string()],
        category: "Error handling".to_string(),
    });

    // Help system
    register_help(HelpEntry {
        name: "help".to_string(),
        signature: "(help) or (help 'function-name)".to_string(),
        description: "Show help information. With no arguments, displays quick reference.\nWith a function name, shows detailed documentation for that function.".to_string(),
        examples: vec![
            "(help) => shows quick reference".to_string(),
            "(help 'cons) => detailed help for cons".to_string(),
            "(help 'map) => help for user or stdlib function".to_string(),
        ],
        related: vec!["doc".to_string()],
        category: "Help system".to_string(),
    });

    register_help(HelpEntry {
        name: "doc".to_string(),
        signature: "(doc function)".to_string(),
        description: "Returns the docstring of a function as a string.\nWorks with user-defined functions that have docstrings.".to_string(),
        examples: vec![
            "(doc factorial) => \"Computes factorial\"".to_string(),
        ],
        related: vec!["help".to_string()],
        category: "Help system".to_string(),
    });

    // Special Forms
    register_help(HelpEntry {
        name: "define".to_string(),
        signature: "(define name value)\n(define (name param1 param2 ...) body)".to_string(),
        description: "Binds a name to a value in the current scope.\nCan define variables or functions with the shorthand syntax.".to_string(),
        examples: vec![
            "(define x 42)".to_string(),
            "(define (square x) (* x x))".to_string(),
            "(square 5) => 25".to_string(),
        ],
        related: vec!["let".to_string(), "lambda".to_string()],
        category: "Special Forms".to_string(),
    });

    register_help(HelpEntry {
        name: "lambda".to_string(),
        signature: "(lambda (param1 param2 ...) body)\n(lambda (param1 param2 ...) \"docstring\" body)".to_string(),
        description: "Creates an anonymous function that captures the current lexical environment.\nOptional docstring available via help system.".to_string(),
        examples: vec![
            "((lambda (x) (* x x)) 5) => 25".to_string(),
            "(define square (lambda (x) (* x x)))".to_string(),
            "(map (lambda (x) (+ x 1)) '(1 2 3)) => (2 3 4)".to_string(),
        ],
        related: vec!["define".to_string(), "let".to_string()],
        category: "Special Forms".to_string(),
    });

    register_help(HelpEntry {
        name: "if".to_string(),
        signature: "(if condition then-expr else-expr)".to_string(),
        description: "Conditional evaluation. Evaluates then-expr if condition is truthy, else-expr otherwise.\n#f and nil are falsy; everything else is truthy.".to_string(),
        examples: vec![
            "(if #t 1 2) => 1".to_string(),
            "(if (= 5 5) \"yes\" \"no\") => \"yes\"".to_string(),
            "(if #f 1 2) => 2".to_string(),
        ],
        related: vec!["begin".to_string(), "and".to_string(), "or".to_string()],
        category: "Special Forms".to_string(),
    });

    register_help(HelpEntry {
        name: "begin".to_string(),
        signature: "(begin expr1 expr2 ...)".to_string(),
        description: "Evaluates expressions in sequence and returns the value of the last one.\nUsed to group multiple expressions where only one is expected.".to_string(),
        examples: vec![
            "(begin (define x 1) (+ x 1)) => 2".to_string(),
            "(begin 1 2 3) => 3".to_string(),
        ],
        related: vec!["if".to_string(), "let".to_string()],
        category: "Special Forms".to_string(),
    });

    register_help(HelpEntry {
        name: "let".to_string(),
        signature: "(let ((var1 val1) (var2 val2) ...) body)".to_string(),
        description: "Creates local bindings for expressions.\nVariables are bound simultaneously (all right-side expressions use outer scope).\nBody has access to all bindings.".to_string(),
        examples: vec![
            "(let ((x 1) (y 2)) (+ x y)) => 3".to_string(),
            "(let ((x 5)) (* x x)) => 25".to_string(),
        ],
        related: vec!["define".to_string(), "lambda".to_string()],
        category: "Special Forms".to_string(),
    });

    register_help(HelpEntry {
        name: "quote".to_string(),
        signature: "'expr or (quote expr)".to_string(),
        description: "Returns the expression unevaluated as a data structure.\nUseful for working with code as data and creating lists.".to_string(),
        examples: vec![
            "'(1 2 3) => (1 2 3)".to_string(),
            "'hello => hello".to_string(),
            "(quote (+ 1 2)) => (+ 1 2)".to_string(),
        ],
        related: vec!["quasiquote".to_string()],
        category: "Special Forms".to_string(),
    });

    register_help(HelpEntry {
        name: "quasiquote".to_string(),
        signature: "`expr or (quasiquote expr)".to_string(),
        description: "Like quote, but allows selective evaluation using unquote (,) and unquote-splicing (,@).\nPrimarily used for writing macros.".to_string(),
        examples: vec![
            "`(1 2 3) => (1 2 3)".to_string(),
            "`(1 ,(+ 1 1) 3) => (1 2 3)".to_string(),
        ],
        related: vec!["quote".to_string(), "unquote".to_string(), "defmacro".to_string()],
        category: "Special Forms".to_string(),
    });

    register_help(HelpEntry {
        name: "unquote".to_string(),
        signature: ",expr (inside a quasiquote)".to_string(),
        description: "Evaluates an expression inside a quasiquote context.\nOnly meaningful within a quasiquote.".to_string(),
        examples: vec![
            "`(1 ,(+ 1 1) 3) => (1 2 3)".to_string(),
        ],
        related: vec!["quasiquote".to_string(), "unquote-splicing".to_string()],
        category: "Special Forms".to_string(),
    });

    register_help(HelpEntry {
        name: "unquote-splicing".to_string(),
        signature: ",@expr (inside a quasiquote)".to_string(),
        description: "Evaluates an expression and splices its elements into the surrounding list.\nOnly meaningful within a quasiquote context.".to_string(),
        examples: vec![
            "`(1 ,@'(2 3) 4) => (1 2 3 4)".to_string(),
        ],
        related: vec!["quasiquote".to_string(), "unquote".to_string()],
        category: "Special Forms".to_string(),
    });

    register_help(HelpEntry {
        name: "defmacro".to_string(),
        signature: "(defmacro (name param1 param2 ...) body)".to_string(),
        description: "Defines a macro that transforms code at compile-time.\nMacros receive unevaluated arguments and return code to be evaluated.\nUse with quasiquote and unquote for code generation.".to_string(),
        examples: vec![
            "(defmacro (unless cond body) `(if (not ,cond) ,body))".to_string(),
        ],
        related: vec!["lambda".to_string(), "quasiquote".to_string()],
        category: "Special Forms".to_string(),
    });
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

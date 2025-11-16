// ABOUTME: Registry for standard library function documentation
//
// This module registers help entries for all stdlib functions with proper categorization.
// When stdlib modules are loaded, their functions get registered here with the "Standard Library"
// category, distinguishing them from true user-defined functions that appear later.
//
// ## Naming Convention
//
// Stdlib functions use NAMESPACED naming (module:function) to distinguish them from core
// primitives. This is a Clojure-style convention where:
//
// - Namespaced functions (json:encode, http:body, map:query) are domain-specific helpers
// - Kebab-case functions (http-request, map-get, filter) are core primitives
//
// For full details, see CLAUDE.md "Naming Conventions" section.

use crate::help::HelpEntry;

/// Register all standard library functions with proper categorization
/// This ensures stdlib functions show up in "Standard Library" category,
/// not as "User-defined" (which reserves for actual user code)
pub fn register_stdlib_functions() {
    // Core module functions
    register_core_functions();
    // Math module functions
    register_math_functions();
    // String module functions
    register_string_functions();
    // Test module functions
    register_test_functions();
    // HTTP module functions
    register_http_functions();
    // Concurrency module functions
    register_concurrency_functions();
}

fn register_core_functions() {
    let functions = vec![
        ("map", "(map f lst)", "Apply function to each element, returning new list.\n\n**Parameters:**\n- f: Function to apply to each element\n- lst: Input list\n\n**Returns:** New list with f applied to each element\n\n**Time Complexity:** O(n) where n is list length\n\n**Examples:**\n- (map (lambda (x) (* x 2)) '(1 2 3)) => (2 4 6)"),
        ("filter", "(filter pred lst)", "Keep only elements satisfying predicate.\n\n**Parameters:**\n- pred: Predicate function returning boolean\n- lst: Input list\n\n**Returns:** New list containing only elements where pred returns true\n\n**Time Complexity:** O(n) where n is list length\n\n**Examples:**\n- (filter (lambda (x) (> x 2)) '(1 2 3 4 5)) => (3 4 5)"),
        ("reduce", "(reduce f initial lst)", "Fold/reduce a list to a single value using function.\n\n**Parameters:**\n- f: Binary function (accumulator element -> result)\n- initial: Initial accumulator value\n- lst: Input list\n\n**Returns:** Final accumulated value\n\n**Time Complexity:** O(n) where n is list length\n\n**Examples:**\n- (reduce + 0 '(1 2 3 4)) => 10"),
        ("compose", "(compose f g)", "Compose two functions into a single function.\n\n**Parameters:**\n- f: Outer function\n- g: Inner function\n\n**Returns:** Function that applies g then f\n\n**Examples:**\n- ((compose (lambda (x) (* x 2)) (lambda (x) (+ x 1))) 5) => 12"),
        ("partial", "(partial f arg)", "Partially apply a function with one argument.\n\n**Parameters:**\n- f: Function to partially apply\n- arg: Argument to bind\n\n**Returns:** Function with arg bound\n\n**Examples:**\n- (define add5 (partial + 5))\n- (add5 3) => 8"),
        ("reverse", "(reverse lst)", "Reverse a list.\n\n**Parameters:**\n- lst: Input list\n\n**Returns:** New list with elements in reverse order\n\n**Time Complexity:** O(n) where n is list length"),
        ("append", "(append lst1 lst2)", "Concatenate two lists.\n\n**Parameters:**\n- lst1: First list\n- lst2: Second list\n\n**Returns:** New list with all elements\n\n**Time Complexity:** O(n) where n is length of first list"),
        ("member", "(member elem lst)", "Check if element is in list.\n\n**Parameters:**\n- elem: Element to find\n- lst: List to search\n\n**Returns:** First tail of list starting with elem, or nil\n\n**Time Complexity:** O(n) where n is list length"),
        ("nth", "(nth n lst)", "Get the nth element of a list (0-indexed).\n\n**Parameters:**\n- n: Index (0-based)\n- lst: List\n\n**Returns:** Element at index n, or nil if out of bounds"),
        ("last", "(last lst)", "Get the last element of a list.\n\n**Parameters:**\n- lst: Input list\n\n**Returns:** Last element, or nil if empty"),
        ("take", "(take n lst)", "Take first n elements of a list.\n\n**Parameters:**\n- n: Number of elements\n- lst: Input list\n\n**Returns:** New list with first n elements"),
        ("drop", "(drop n lst)", "Drop first n elements of a list.\n\n**Parameters:**\n- n: Number of elements to skip\n- lst: Input list\n\n**Returns:** New list without first n elements"),
        ("zip", "(zip lst1 lst2)", "Combine two lists into pairs.\n\n**Parameters:**\n- lst1: First list\n- lst2: Second list\n\n**Returns:** List of pairs [elem1 elem2]\n\n**Time Complexity:** O(n) where n is length of shorter list"),
        ("map:query", "(map:query m key default)", "Get value from map with default.\n\n**Parameters:**\n- m: Map to query\n- key: Keyword key\n- default: Default value if key not found"),
        ("map:select", "(map:select m keys)", "Select subset of map by keys.\n\n**Parameters:**\n- m: Source map\n- keys: List of keywords to select"),
        ("map:update", "(map:update m key f)", "Update map value using function.\n\n**Parameters:**\n- m: Map to update\n- key: Keyword key\n- f: Function to apply to current value"),
        ("map:from-entries", "(map:from-entries entries)", "Build map from list of key-value pairs.\n\n**Parameters:**\n- entries: List of [key value] pairs"),
        ("map:filter", "(map:filter pred m)", "Filter map entries by predicate.\n\n**Parameters:**\n- pred: Predicate function taking [key value] pair\n- m: Map to filter"),
        ("map:map-values", "(map:map-values f m)", "Transform all values in map.\n\n**Parameters:**\n- f: Function to apply to each value\n- m: Map to transform"),
    ];

    for (name, sig, desc) in functions {
        crate::help::register_help(HelpEntry {
            name: name.to_string(),
            signature: sig.to_string(),
            description: desc.to_string(),
            examples: vec![],
            related: vec![],
            category: "Standard Library: Core".to_string(),
        });
    }
}

fn register_math_functions() {
    let functions = vec![
        ("abs", "(abs n)", "Absolute value.\n\n**Parameters:**\n- n: Number\n\n**Returns:** Absolute value of n\n\n**Time Complexity:** O(1)"),
        ("min", "(min lst)", "Find minimum value in list.\n\n**Parameters:**\n- lst: List of numbers\n\n**Returns:** Smallest number\n\n**Time Complexity:** O(n)"),
        ("max", "(max lst)", "Find maximum value in list.\n\n**Parameters:**\n- lst: List of numbers\n\n**Returns:** Largest number\n\n**Time Complexity:** O(n)"),
        ("square", "(square n)", "Square a number.\n\n**Parameters:**\n- n: Number\n\n**Returns:** n * n\n\n**Time Complexity:** O(1)"),
        ("cube", "(cube n)", "Cube a number.\n\n**Parameters:**\n- n: Number\n\n**Returns:** n * n * n\n\n**Time Complexity:** O(1)"),
        ("even?", "(even? n)", "Check if number is even.\n\n**Parameters:**\n- n: Number\n\n**Returns:** true if even, false otherwise\n\n**Time Complexity:** O(1)"),
        ("odd?", "(odd? n)", "Check if number is odd.\n\n**Parameters:**\n- n: Number\n\n**Returns:** true if odd, false otherwise\n\n**Time Complexity:** O(1)"),
        ("sum", "(sum lst)", "Sum all numbers in a list.\n\n**Parameters:**\n- lst: List of numbers\n\n**Returns:** Sum of all elements\n\n**Time Complexity:** O(n)"),
        ("product", "(product lst)", "Multiply all numbers in a list.\n\n**Parameters:**\n- lst: List of numbers\n\n**Returns:** Product of all elements\n\n**Time Complexity:** O(n)"),
        ("factorial", "(factorial n)", "Compute factorial of n (non-tail-recursive).\n\n**Parameters:**\n- n: Non-negative integer\n\n**Returns:** n!\n\n**Time Complexity:** O(n)\n\n**Note:** Not tail-recursive, may stack overflow for large n"),
        ("all", "(all pred lst)", "Check if all elements satisfy predicate.\n\n**Parameters:**\n- pred: Predicate function\n- lst: List to check"),
        ("any", "(any pred lst)", "Check if any element satisfies predicate.\n\n**Parameters:**\n- pred: Predicate function\n- lst: List to check"),
        ("count", "(count pred lst)", "Count elements satisfying predicate.\n\n**Parameters:**\n- pred: Predicate function\n- lst: List to check"),
        ("range", "(range n)", "Generate list of numbers from 0 to n-1.\n\n**Parameters:**\n- n: Upper bound (exclusive)"),
    ];

    for (name, sig, desc) in functions {
        crate::help::register_help(HelpEntry {
            name: name.to_string(),
            signature: sig.to_string(),
            description: desc.to_string(),
            examples: vec![],
            related: vec![],
            category: "Standard Library: Math".to_string(),
        });
    }
}

fn register_string_functions() {
    let functions = vec![
        ("string-capitalize", "(string-capitalize s)", "Capitalize first character of string.\n\n**Parameters:**\n- s: String to capitalize\n\n**Returns:** String with first char uppercase\n\n**Time Complexity:** O(n)"),
        ("string-concat", "(string-concat lst)", "Concatenate list of strings.\n\n**Parameters:**\n- lst: List of strings\n\n**Returns:** Single concatenated string\n\n**Time Complexity:** O(n)"),
        ("string-reverse", "(string-reverse s)", "Reverse a string.\n\n**Parameters:**\n- s: String to reverse\n\n**Returns:** Reversed string\n\n**Time Complexity:** O(n)"),
        ("string-repeat", "(string-repeat s n)", "Repeat string n times.\n\n**Parameters:**\n- s: String to repeat\n- n: Number of repetitions\n\n**Returns:** Concatenated result\n\n**Time Complexity:** O(n*m) where m is string length"),
        ("string-words", "(string-words s)", "Split string into words by whitespace.\n\n**Parameters:**\n- s: String to split\n\n**Returns:** List of word strings\n\n**Time Complexity:** O(n)"),
        ("string-lines", "(string-lines s)", "Split string into lines by newline.\n\n**Parameters:**\n- s: String to split\n\n**Returns:** List of line strings\n\n**Time Complexity:** O(n)"),
        ("string-pad-left", "(string-pad-left s width char)", "Pad string on left to width.\n\n**Parameters:**\n- s: String to pad\n- width: Target width\n- char: Padding character\n\n**Returns:** Padded string"),
    ];

    for (name, sig, desc) in functions {
        crate::help::register_help(HelpEntry {
            name: name.to_string(),
            signature: sig.to_string(),
            description: desc.to_string(),
            examples: vec![],
            related: vec![],
            category: "Standard Library: String".to_string(),
        });
    }
}

fn register_test_functions() {
    let functions = vec![
        ("print-test-summary", "(print-test-summary results)", "Print formatted test results from run-all-tests.\n\n**Parameters:**\n- results: Result map with :passed, :failed, :total, :tests\n\n**Returns:** nil (prints to console)"),
        ("print-test-details", "(print-test-details tests)", "Print each test result with status.\n\n**Parameters:**\n- tests: List of test result maps with :name, :status, :message\n\n**Returns:** nil (prints to console)"),
    ];

    for (name, sig, desc) in functions {
        crate::help::register_help(HelpEntry {
            name: name.to_string(),
            signature: sig.to_string(),
            description: desc.to_string(),
            examples: vec![],
            related: vec![],
            category: "Standard Library: Testing".to_string(),
        });
    }
}

fn register_http_functions() {
    let functions = vec![
        ("http:check-status", "(http:check-status response)", "Check if HTTP response status is success (2xx).\n\n**Parameters:**\n- response: HTTP response map with :status\n\n**Returns:** true if 2xx, false otherwise"),
        ("http:body", "(http:body response)", "Extract body from HTTP response.\n\n**Parameters:**\n- response: HTTP response map with :body\n\n**Returns:** Response body string"),
        ("http:status", "(http:status response)", "Extract status code from HTTP response.\n\n**Parameters:**\n- response: HTTP response map with :status\n\n**Returns:** Status code number"),
    ];

    for (name, sig, desc) in functions {
        crate::help::register_help(HelpEntry {
            name: name.to_string(),
            signature: sig.to_string(),
            description: desc.to_string(),
            examples: vec![],
            related: vec![],
            category: "Standard Library: HTTP".to_string(),
        });
    }
}

fn register_concurrency_functions() {
    let functions = vec![
        ("parallel-map", "(parallel-map f lst)", "Map function over list in parallel using spawn.\n\n**Parameters:**\n- f: Function to apply\n- lst: Input list\n\n**Returns:** List of results from parallel execution"),
        ("parallel-map-link", "(parallel-map-link f lst)", "Map function over list in parallel using spawn-link.\n\n**Parameters:**\n- f: Function to apply\n- lst: Input list\n\n**Returns:** List of results, errors propagated"),
        ("pmap", "(pmap f lst)", "Alias for parallel-map.\n\n**Parameters:**\n- f: Function to apply\n- lst: Input list"),
        ("parallel-for-each", "(parallel-for-each f lst)", "Execute function on each element in parallel, discard results.\n\n**Parameters:**\n- f: Side-effect function\n- lst: Input list\n\n**Returns:** nil"),
        ("fan-out", "(fan-out f inputs)", "Execute function on multiple inputs in parallel.\n\n**Parameters:**\n- f: Function to execute\n- inputs: List of input values"),
        ("parallel-pipeline", "(parallel-pipeline stages input)", "Execute pipeline stages in parallel.\n\n**Parameters:**\n- stages: List of functions\n- input: Initial input value"),
    ];

    for (name, sig, desc) in functions {
        crate::help::register_help(HelpEntry {
            name: name.to_string(),
            signature: sig.to_string(),
            description: desc.to_string(),
            examples: vec![],
            related: vec![],
            category: "Standard Library: Concurrency".to_string(),
        });
    }
}

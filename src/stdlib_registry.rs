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
        ("factorial", "(factorial n)", "Compute factorial of n.\n\n**Parameters:**\n- n: Non-negative integer\n\n**Returns:** n!\n\n**Time Complexity:** O(n)"),
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
        ("upcase", "(upcase s)", "Convert string to uppercase.\n\n**Parameters:**\n- s: String\n\n**Returns:** Uppercase version\n\n**Time Complexity:** O(n) where n is string length"),
        ("downcase", "(downcase s)", "Convert string to lowercase.\n\n**Parameters:**\n- s: String\n\n**Returns:** Lowercase version\n\n**Time Complexity:** O(n) where n is string length"),
        ("trim", "(trim s)", "Remove leading and trailing whitespace.\n\n**Parameters:**\n- s: String\n\n**Returns:** Trimmed string\n\n**Time Complexity:** O(n) where n is string length"),
        ("starts-with?", "(starts-with? s prefix)", "Check if string starts with prefix.\n\n**Parameters:**\n- s: String\n- prefix: Prefix to check\n\n**Returns:** true if starts with prefix\n\n**Time Complexity:** O(m) where m is prefix length"),
        ("ends-with?", "(ends-with? s suffix)", "Check if string ends with suffix.\n\n**Parameters:**\n- s: String\n- suffix: Suffix to check\n\n**Returns:** true if ends with suffix\n\n**Time Complexity:** O(m) where m is suffix length"),
        ("contains?", "(contains? s substring)", "Check if string contains substring.\n\n**Parameters:**\n- s: String\n- substring: Substring to find\n\n**Returns:** true if contains\n\n**Time Complexity:** O(n*m) where n is string length and m is substring length"),
        ("split", "(split s delimiter)", "Split string by delimiter.\n\n**Parameters:**\n- s: String\n- delimiter: Split character or string\n\n**Returns:** List of string parts\n\n**Time Complexity:** O(n) where n is string length"),
        ("join", "(join lst sep)", "Join list of strings with separator.\n\n**Parameters:**\n- lst: List of strings\n- sep: Separator string\n\n**Returns:** Joined string\n\n**Time Complexity:** O(n) where n is total characters"),
        ("replace", "(replace s old new)", "Replace all occurrences of substring.\n\n**Parameters:**\n- s: String\n- old: Substring to replace\n- new: Replacement string\n\n**Returns:** New string with replacements\n\n**Time Complexity:** O(n*m) where n is string length"),
        ("length", "(length s)", "Get string length.\n\n**Parameters:**\n- s: String\n\n**Returns:** Number of characters\n\n**Time Complexity:** O(1)"),
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
        ("assert", "(assert condition)", "Assert that condition is true, fail with error if false.\n\n**Parameters:**\n- condition: Boolean expression\n\n**Returns:** true if assertion passes\n\n**Raises:** Error if condition is false\n\n**Time Complexity:** O(1)"),
        ("assert-equal", "(assert-equal a b)", "Assert that two values are equal.\n\n**Parameters:**\n- a: First value\n- b: Second value\n\n**Returns:** true if equal\n\n**Raises:** Error if not equal\n\n**Time Complexity:** O(1)"),
        ("assert-error", "(assert-error expr)", "Assert that expression raises an error.\n\n**Parameters:**\n- expr: Expression that should error\n\n**Returns:** true if error occurred\n\n**Time Complexity:** O(1)"),
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
        ("http:parse-response", "(http:parse-response response-map)", "Parse HTTP response into structured format.\n\n**Parameters:**\n- response-map: Map with :status, :headers, :body\n\n**Returns:** Parsed response object\n\n**Time Complexity:** O(n) where n is response body size"),
        ("http:build-query", "(http:build-query params)", "Build URL query string from parameter map.\n\n**Parameters:**\n- params: Map of query parameters\n\n**Returns:** URL-encoded query string\n\n**Time Complexity:** O(n) where n is number of parameters"),
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

// ABOUTME: Help and documentation system for the Lisp interpreter
// Provides first-class documentation for built-in and user-defined functions
// Renders markdown documentation with syntax highlighting using termimad

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use termimad::MadSkin;

// Forward declarations
use crate::env::Environment;
use crate::value::Value;

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
}

impl Default for HelpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-local help registry
thread_local! {
    static HELP_REGISTRY: RefCell<HelpRegistry> = RefCell::new(HelpRegistry::new());
    static CURRENT_ENV: RefCell<Option<Arc<Environment>>> = const { RefCell::new(None) };
}

/// Set the current environment for help lookup (needed for user-defined functions)
pub fn set_current_env(env: Option<Arc<Environment>>) {
    CURRENT_ENV.with(|e| {
        *e.borrow_mut() = env;
    });
}

/// Get help for a Lisp-defined function from the environment
fn get_lisp_function_help(name: &str) -> Option<HelpEntry> {
    CURRENT_ENV.with(|env_ref| {
        let env_opt = env_ref.borrow();
        if let Some(env) = env_opt.as_ref() {
            if let Some(val) = env.get(name) {
                match val {
                    Value::Lambda {
                        params, docstring, ..
                    } => {
                        // Build signature from parameters
                        let mut sig = format!("({}", name);
                        for param in &params {
                            sig.push(' ');
                            sig.push_str(param);
                        }
                        sig.push(')');

                        return Some(HelpEntry {
                            name: name.to_string(),
                            signature: sig,
                            description: docstring.clone().unwrap_or_default(),
                            examples: Vec::new(),
                            related: Vec::new(),
                            category: "User-defined".to_string(),
                        });
                    }
                    Value::Macro { params, .. } => {
                        // Build signature from parameters
                        let mut sig = format!("({}", name);
                        for param in &params {
                            sig.push(' ');
                            sig.push_str(param);
                        }
                        sig.push(')');

                        return Some(HelpEntry {
                            name: name.to_string(),
                            signature: sig,
                            description: "(macro)".to_string(),
                            examples: Vec::new(),
                            related: Vec::new(),
                            category: "Macro".to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }
        None
    })
}

/// Register a help entry in the global registry
pub fn register_help(entry: HelpEntry) {
    HELP_REGISTRY.with(|reg| {
        reg.borrow_mut().register(entry);
    });
}

/// Get a help entry by name (checks environment first for shadowing, then registry)
pub fn get_help(name: &str) -> Option<HelpEntry> {
    // Try environment first to handle shadowing (user-defined functions override stdlib)
    if let Some(entry) = get_lisp_function_help(name) {
        return Some(entry);
    }

    // Fall back to registry for builtins and registered help
    HELP_REGISTRY.with(|reg| reg.borrow().get(name))
}

/// Get all entries organized by category
pub fn all_by_category() -> HashMap<String, Vec<HelpEntry>> {
    HELP_REGISTRY.with(|reg| reg.borrow().by_category())
}

/// Format a single help entry for display with markdown rendering and syntax highlighting
pub fn format_help_entry(entry: &HelpEntry) -> String {
    let skin = MadSkin::default();
    let mut output = String::new();

    // Header with name and category
    output.push_str(&format!("# {} - {}\n\n", entry.name, entry.category));

    // Signature
    output.push_str("## Signature\n\n");
    output.push_str(&format!("`{}`\n\n", entry.signature));

    // Description (rendered as markdown)
    output.push_str("## Description\n\n");
    output.push_str(&entry.description);
    output.push_str("\n\n");

    // Examples with code block formatting
    if !entry.examples.is_empty() {
        output.push_str("## Examples\n\n");
        for example in &entry.examples {
            output.push_str("```lisp\n");
            output.push_str(example);
            output.push_str("\n```\n\n");
        }
    }

    // Related functions
    if !entry.related.is_empty() {
        output.push_str("## See Also\n\n");
        output.push_str(&entry.related.join(", "));
        output.push_str("\n\n");
    }

    // Render markdown with termimad
    skin.term_text(&output).to_string()
}

/// Format quick reference showing all functions
pub fn format_quick_reference() -> String {
    let mut output = String::new();

    let by_cat = all_by_category();
    let total = by_cat.values().map(|v| v.len()).sum::<usize>();

    output.push_str(&format!("Available Functions ({} total)\n", total));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    // Get all categories and sort them, with preferred categories first
    let preferred_order = vec![
        "Special Forms",
        "Arithmetic",
        "Comparison",
        "Logic",
        "Type predicates",
        "List operations",
        "String manipulation",
        "Maps",
        "Testing",
        "Console I/O",
        "Filesystem I/O",
        "Network I/O",
        "Error handling",
        "Help system",
    ];

    // Add any categories not in preferred list (sorted alphabetically)
    let mut other_categories: Vec<&str> = by_cat
        .keys()
        .filter(|cat| !preferred_order.contains(&cat.as_str()))
        .map(|s| s.as_str())
        .collect();
    other_categories.sort();

    let all_categories = preferred_order
        .into_iter()
        .chain(other_categories)
        .collect::<Vec<_>>();

    for category in all_categories {
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
    #[serial_test::serial]
    fn test_user_defined_function_shadows_stdlib_help() {
        use crate::env::Environment;
        use std::rc::Rc;

        // Create an environment and define a user function
        let env = Rc::new(Environment::new());
        let user_sum = Value::Lambda {
            params: vec!["x".to_string(), "y".to_string()],
            body: Box::new(Value::Symbol("+".to_string())),
            env: Rc::clone(&env),
            docstring: Some("Add two numbers together".to_string()),
        };
        env.define("sum".to_string(), user_sum);

        // Set the current environment for help lookup
        set_current_env(Some(Rc::clone(&env)));

        // Get help should return the user-defined version, not stdlib
        let help = get_help("sum");
        assert!(help.is_some());
        let entry = help.unwrap();
        assert_eq!(entry.name, "sum");
        assert_eq!(entry.category, "User-defined");
        assert_eq!(entry.description, "Add two numbers together");
        assert_eq!(entry.signature, "(sum x y)");

        // Clean up
        set_current_env(None);
    }
}

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
}

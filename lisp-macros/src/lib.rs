//! Procedural macros for lisp-llm-sandbox builtin functions
//!
//! Provides the `#[builtin]` attribute macro for defining Lisp builtins
//! with rustdoc-style documentation that is automatically converted to
//! help entries and registration code.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, ItemFn, Meta};

/// A parsed markdown documentation with structured sections
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DocMarkdown {
    summary: String,
    examples: Vec<String>,
    see_also: Vec<String>,
    full_markdown: String,
}

/// Extract rustdoc comments from function attributes
fn extract_doc_comments(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            // Check if this is a doc comment (/// or ///)
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &nv.value
                    {
                        return Some(lit_str.value());
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse markdown sections from documentation
fn parse_doc_markdown(raw_doc: &str) -> DocMarkdown {
    let mut summary = String::new();
    let mut examples = Vec::new();
    let mut see_also = Vec::new();
    let mut current_section = "summary";
    let mut current_content = String::new();

    for line in raw_doc.lines() {
        let trimmed = line.trim();

        // Check for section headers
        if let Some(rest) = trimmed.strip_prefix("# ") {
            // Save previous section
            match current_section {
                "summary" => summary = current_content.trim().to_string(),
                "examples" => {
                    // Parse code blocks from examples
                    for code_block in current_content.split("```") {
                        let trimmed_block = code_block.trim();
                        if let Some(code_str) = trimmed_block.strip_prefix("lisp") {
                            let code = code_str.trim().to_string();
                            if !code.is_empty() {
                                examples.push(code);
                            }
                        }
                    }
                }
                "see also" => {
                    see_also = current_content
                        .trim()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                _ => {}
            }

            // Parse new section header
            let header = rest.trim().to_lowercase();
            current_section = if header.contains("example") {
                "examples"
            } else if header.contains("see") || header.contains("related") {
                "see also"
            } else {
                "other"
            };
            current_content.clear();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last section
    match current_section {
        "summary" => summary = current_content.trim().to_string(),
        "examples" => {
            for code_block in current_content.split("```") {
                let trimmed_block = code_block.trim();
                if let Some(code_str) = trimmed_block.strip_prefix("lisp") {
                    let code = code_str.trim().to_string();
                    if !code.is_empty() {
                        examples.push(code);
                    }
                }
            }
        }
        "see also" => {
            see_also = current_content
                .trim()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        _ => {}
    }

    DocMarkdown {
        summary,
        examples,
        see_also,
        full_markdown: raw_doc.to_string(),
    }
}

/// Parse builtin attribute arguments: name = "...", category = "...", related(...)
fn parse_builtin_args(attr_stream: TokenStream) -> (String, String, Vec<String>) {
    let attr_str = attr_stream.to_string();

    // Simple parsing - look for name = "..." and category = "..."
    let mut name = String::new();
    let mut category = String::new();
    let mut related = Vec::new();

    // Parse name
    if let Some(start) = attr_str.find("name = \"") {
        let rest = &attr_str[start + 8..];
        if let Some(end) = rest.find('"') {
            name = rest[..end].to_string();
        }
    }

    // Parse category
    if let Some(start) = attr_str.find("category = \"") {
        let rest = &attr_str[start + 12..];
        if let Some(end) = rest.find('"') {
            category = rest[..end].to_string();
        }
    }

    // Parse related functions
    if let Some(start) = attr_str.find("related") {
        let rest = &attr_str[start..];
        if let Some(paren_start) = rest.find('(') {
            if let Some(paren_end) = rest.find(')') {
                let related_str = &rest[paren_start + 1..paren_end];
                related = related_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }

    (name, category, related)
}

/// Attribute macro for defining Lisp builtin functions
///
/// Extracts rustdoc comments and generates both the function and a registration
/// function that binds it to the environment and registers help documentation.
///
/// # Attribute Arguments
///
/// - `name`: The Lisp name for this builtin (e.g., "+")
/// - `category`: Category for help organization (e.g., "Arithmetic")
/// - `related`: Related builtin functions to list in help
///
/// # Example
///
/// ```ignore
/// #[builtin(name = "+", category = "Arithmetic", related(sub, mul, div))]
/// /// Returns the sum of all arguments.
/// ///
/// /// # Examples
/// /// ```lisp
/// /// (+ 1 2 3) => 6
/// /// ```
/// pub fn add(args: &[Value]) -> Result<Value, EvalError> {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn builtin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    // Extract metadata from attribute
    let (lisp_name, category, related) = parse_builtin_args(attr);

    // Extract function metadata
    let fn_name = func.sig.ident.clone();
    let fn_ident_str = fn_name.to_string();

    // Use provided name or fall back to function name
    let name_to_use = if !lisp_name.is_empty() {
        lisp_name
    } else {
        fn_ident_str.clone()
    };

    // Extract and parse doc comments
    let raw_docs = extract_doc_comments(&func.attrs);
    let parsed_docs = parse_doc_markdown(&raw_docs);

    // Use parsed summary or full markdown as description
    let description = if !parsed_docs.summary.is_empty() {
        parsed_docs.summary.clone()
    } else {
        parsed_docs.full_markdown.clone()
    };

    // Generate the registration function name
    let register_fn_name = quote::format_ident!("register_{}", fn_name);
    let help_fn_name = quote::format_ident!("register_help_{}", fn_name);

    // Build the examples vector
    let examples = parsed_docs.examples.clone();

    // Build the related vector (from attribute)
    let related_vec = related;

    // Build the category (with fallback)
    let cat_to_use = if !category.is_empty() {
        category.clone()
    } else {
        "Other".to_string()
    };

    // Generate signature as "(name ...)"
    let signature = format!("({} ...)", name_to_use);

    // Generate the expanded code
    let expanded = quote! {
        #func

        /// Register the #fn_name builtin in the environment
        #[allow(dead_code)]
        pub fn #register_fn_name(env: std::rc::Rc<crate::env::Environment>) {
            env.define(
                #name_to_use.to_string(),
                crate::value::Value::BuiltIn(#fn_name)
            );
        }

        /// Register help entry for #fn_name
        #[allow(dead_code)]
        pub fn #help_fn_name() {
            crate::help::register_help(crate::help::HelpEntry {
                name: #name_to_use.to_string(),
                signature: #signature.to_string(),
                description: #description.to_string(),
                examples: vec![#(#examples.to_string()),*],
                related: vec![#(#related_vec.to_string()),*],
                category: #cat_to_use.to_string(),
            });
        }
    };

    TokenStream::from(expanded)
}

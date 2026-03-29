use crate::Violation;
use std::path::Path;
use syn::{Item, ItemStruct, ItemEnum, ItemImpl, ImplItem};

pub fn lint_type_separation(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let max_length_for_mixed_file = 50;
    let mut violations = Vec::new();

    let path_str = path.to_string_lossy();
    if path_str.contains("/tests/") || path_str.ends_with("_test.rs") || path_str.ends_with("tests.rs") {
        return violations;
    }

    let is_whitelisted = is_whitelisted_type_file(&path_str);
    
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let num_lines = source.lines().count();

    if is_whitelisted || num_lines <= max_length_for_mixed_file {
        return violations;
    }

    let mut has_pub_type = false;
    let mut has_logic_impl = false;
    let mut first_pub_type_line = 0;

    for item in &syntax.items {
        match item {
            Item::Struct(ItemStruct { vis, ident, .. }) => {
                if matches!(vis, syn::Visibility::Public(_)) {
                    has_pub_type = true;
                    first_pub_type_line = ident.span().start().line;
                }
            }
            Item::Enum(ItemEnum { vis, ident, .. }) => {
                if matches!(vis, syn::Visibility::Public(_)) {
                    has_pub_type = true;
                    first_pub_type_line = ident.span().start().line;
                }
            }
            Item::Impl(ItemImpl { items, .. }) => {
                for impl_item in items {
                    if let ImplItem::Fn(_) = impl_item {
                        has_logic_impl = true;
                    }
                }
            }
            _ => {}
        }
    }

    if has_pub_type && has_logic_impl {
        let rel_path = path.strip_prefix(std::env::current_dir().unwrap_or_default())
            .unwrap_or(path)
            .to_path_buf();
            
        violations.push(Violation {
            file: rel_path,
            line: first_pub_type_line,
            column: 1,
            message: format!(
                "Mixed logic and data. File ({num_lines} lines) defines pub struct/enum but also contains method logic. Move types to `types.rs` or `types/` dir, or keep file under {max_length_for_mixed_file} lines."
            ),
        });
    }

    violations
}

fn is_whitelisted_type_file(path_str: &str) -> bool {
    if path_str.ends_with("types.rs") || path_str.ends_with("type.rs") || path_str.ends_with("models.rs") || path_str.ends_with("model.rs") || path_str.ends_with("state.rs") {
        return true;
    }
    if path_str.contains("/types/") || path_str.contains("/models/") || path_str.contains("/state/") {
        return true;
    }
    if path_str.ends_with("lib.rs") || path_str.ends_with("main.rs") {
        return true;
    }
    false
}

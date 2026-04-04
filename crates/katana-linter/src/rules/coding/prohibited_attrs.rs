use crate::Violation;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub struct ProhibitedAttributesOps;

impl ProhibitedAttributesOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = ProhibitedAttributesVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct ProhibitedAttributesVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl ProhibitedAttributesVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_attrs(&mut self, attrs: &[syn::Attribute]) {
        for attr in attrs {
            self.check_single_attr(attr);
        }
    }

    fn check_single_attr(&mut self, attr: &syn::Attribute) {
        if !attr.path().is_ident("allow") {
            return;
        }
        let Ok(nested) = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        ) else {
            return;
        };
        for meta in nested {
            if let syn::Meta::Path(path) = meta
                && path.is_ident("dead_code")
            {
                self.violations.push(Violation {
                    file: self.file.clone(),
                    line: attr.span().start().line,
                    column: attr.span().start().column,
                    message:
                        "Prohibited attribute: #[allow(dead_code)] is NOT allowed by system policy."
                            .to_string(),
                });
            }
        }
    }
}

impl<'ast> Visit<'ast> for ProhibitedAttributesVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.check_attrs(&node.attrs);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.check_attrs(&node.attrs);
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.check_attrs(&node.attrs);
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.check_attrs(&node.attrs);
        syn::visit::visit_item_mod(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_allow_dead_code() {
        let code = r#"#[allow(dead_code)] fn foo() {}"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = ProhibitedAttributesOps::lint(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("dead_code"));
    }
}

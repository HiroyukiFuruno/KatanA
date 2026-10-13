use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub struct LazyCodeOps;

impl LazyCodeOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = LazyCodeVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct LazyCodeVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl LazyCodeVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_macro(&mut self, mac: &syn::Macro) {
        let name = mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        if name == "todo" || name == "unimplemented" || name == "dbg" {
            let (line, column) = LinterParserOps::span_location(mac.path.span());
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message: format!("Lazy code macro `{}!()` detected. Please implement the actual logic or remove debug prints.", name),
            });
        }
    }
}

impl<'ast> Visit<'ast> for LazyCodeVisitor {
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        self.check_macro(node);
        syn::visit::visit_macro(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if LinterParserOps::has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if LinterParserOps::has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if LinterParserOps::has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_impl_item_fn(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_todo() {
        let code = r#"fn foo() { todo!() }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = LazyCodeOps::lint(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("todo"));
    }

    #[test]
    fn detects_dbg() {
        let code = r#"fn foo() { let x = 1; dbg!(x); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = LazyCodeOps::lint(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("dbg"));
    }
}

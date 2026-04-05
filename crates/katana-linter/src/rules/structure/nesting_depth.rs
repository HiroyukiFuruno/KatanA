use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

const MAX_NESTING_DEPTH: usize = 3;

pub struct NestingDepthOps;

impl NestingDepthOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = NestingDepthVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct NestingDepthVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    current_depth: usize,
}

impl NestingDepthVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            current_depth: 0,
        }
    }

    fn check_depth(&mut self, span: proc_macro2::Span) {
        if self.current_depth > MAX_NESTING_DEPTH {
            let (line, column) = LinterParserOps::span_location(span);
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message: format!(
                    "Nesting depth {0} exceeds {MAX_NESTING_DEPTH} levels. Use early returns or extract helpers.",
                    self.current_depth
                ),
            });
        }
    }
}

impl<'ast> Visit<'ast> for NestingDepthVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.current_depth += 1;
        self.check_depth(node.if_token.span);
        syn::visit::visit_expr_if(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.current_depth += 1;
        self.check_depth(node.for_token.span);
        syn::visit::visit_expr_for_loop(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.current_depth += 1;
        self.check_depth(node.while_token.span);
        syn::visit::visit_expr_while(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        self.current_depth += 1;
        self.check_depth(node.match_token.span);
        syn::visit::visit_expr_match(self, node);
        self.current_depth -= 1;
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
    fn detects_deep_nesting() {
        let code = r#"
            fn foo() {
                if true {
                    if true {
                        if true {
                            if true {
                                /* WHY: This is depth 4, which should trigger the violation. */
                            }
                        }
                    }
                }
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = NestingDepthOps::lint(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("4"));
    }

    #[test]
    fn allows_shallow_nesting() {
        let code = r#"
            fn foo() {
                if true {
                    if true {
                        /* WHY: Depth 2 is within the allowed limit and should not trigger. */
                    }
                }
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = NestingDepthOps::lint(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }
}

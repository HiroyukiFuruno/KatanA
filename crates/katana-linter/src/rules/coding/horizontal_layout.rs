use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

pub struct HorizontalLayoutOps;

impl HorizontalLayoutOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = HorizontalLayoutVisitor {
            file_path: path.to_path_buf(),
            violations: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct HorizontalLayoutVisitor {
    file_path: PathBuf,
    violations: Vec<Violation>,
}

impl<'ast> Visit<'ast> for HorizontalLayoutVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "horizontal" {
            let (line, column) = LinterParserOps::span_location(node.method.span());

            self.violations.push(Violation {
                file: self.file_path.clone(),
                line,
                column,
                message: "Use `AlignCenter` instead of `ui.horizontal()` for vertical centering."
                    .to_string(),
            });
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

pub struct PerformanceOps;

impl PerformanceOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = PerformanceVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct PerformanceVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    /* WHY: Track conditional nesting; request_repaint() inside if/match arms is allowed — only unconditional top-level calls in UI loops are prohibited. */
    condition_depth: usize,
}

impl PerformanceVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            condition_depth: 0,
        }
    }
}

impl<'ast> Visit<'ast> for PerformanceVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.condition_depth += 1;
        syn::visit::visit_expr_if(self, node);
        self.condition_depth -= 1;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        self.condition_depth += 1;
        syn::visit::visit_expr_match(self, node);
        self.condition_depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.condition_depth += 1;
        syn::visit::visit_expr_while(self, node);
        self.condition_depth -= 1;
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let name = node.method.to_string();
        if (name == "request_repaint" || name == "set_title") && self.condition_depth == 0 {
            let (line, column) = LinterParserOps::span_location(node.method.span());
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message: format!(
                    "Unconditional `{name}()` call detected. Avoid frequent repaints or title updates in UI loops."
                ),
            });
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

pub struct HorizontalLayoutOps;

impl HorizontalLayoutOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        let lines: Vec<&str> = source.lines().collect();
        let mut visitor = HorizontalLayoutVisitor {
            file_path: path.to_path_buf(),
            source_lines: lines,
            violations: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct HorizontalLayoutVisitor<'a> {
    file_path: PathBuf,
    source_lines: Vec<&'a str>,
    violations: Vec<Violation>,
}

impl<'ast, 'a> Visit<'ast> for HorizontalLayoutVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "horizontal" {
            let (line, column) = LinterParserOps::span_location(node.method.span());

            // WHY: allow(horizontal_layout) on previous line suppresses this rule
            if line > 1
                && let Some(prev) = self.source_lines.get(line - 2)
                    && prev.contains("allow(horizontal_layout)") {
                        syn::visit::visit_expr_method_call(self, node);
                        return;
                    }

            self.violations.push(Violation {
                file: self.file_path.clone(),
                line,
                column,
                message: "Use `AlignCenter` instead of `ui.horizontal()` for vertical centering. Add `// allow(horizontal_layout)` above to suppress.".to_string(),
            });
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

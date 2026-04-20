use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

pub struct MinRectSizingOps;

impl MinRectSizingOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        if path
            .to_string_lossy()
            .replace('\\', "/")
            .contains("/tests/")
        {
            return Vec::new();
        }

        let source = std::fs::read_to_string(path).unwrap_or_default();
        let lines: Vec<&str> = source.lines().collect();

        let mut visitor = MinRectSizingVisitor {
            file_path: path.to_path_buf(),
            source_lines: lines,
            violations: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct MinRectSizingVisitor<'a> {
    file_path: PathBuf,
    source_lines: Vec<&'a str>,
    violations: Vec<Violation>,
}

impl MinRectSizingVisitor<'_> {
    fn is_suppressed(&self, line: usize) -> bool {
        if line > 1
            && let Some(prev) = self.source_lines.get(line - 2)
        {
            let trimmed = prev.trim();
            return trimmed.starts_with("/// WHY: allow(min_rect_sizing)")
                || trimmed.starts_with("/* WHY: allow(min_rect_sizing)");
        }
        false
    }

    fn is_min_rect_receiver(expr: &syn::Expr) -> bool {
        matches!(
            expr,
            syn::Expr::MethodCall(method)
                if method.method == "min_rect"
        )
    }
}

impl<'ast, 'a> Visit<'ast> for MinRectSizingVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let is_sizing_call = node.method == "width" || node.method == "height";
        if is_sizing_call && Self::is_min_rect_receiver(&node.receiver) {
            let (line, column) = LinterParserOps::span_location(node.method.span());
            if !self.is_suppressed(line) {
                self.violations.push(Violation {
                    file: self.file_path.clone(),
                    line,
                    column,
                    message:
                        "Do not derive parent-facing width/height from `ui.min_rect()`. This can leak intrinsic content size into any resizable parent layout and make it expand but not shrink. Use `available_width()`, `available_height()`, or `clip_rect()` instead."
                            .to_string(),
                });
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_min_rect_width_and_height_leaks() {
        let syntax = syn::parse_file(
            r#"
            fn demo(ui: &egui::Ui) {
                let w = ui.min_rect().width();
                let h = ui.min_rect().height();
                let _ = (w, h);
            }
            "#,
        )
        .expect("test source should parse");

        let violations = MinRectSizingOps::lint(Path::new("/tmp/demo.rs"), &syntax);
        assert_eq!(violations.len(), 2);
    }
}

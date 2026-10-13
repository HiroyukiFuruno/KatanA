use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

pub struct ScrollAreaInnerRectLeakOps;

impl ScrollAreaInnerRectLeakOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        let lines: Vec<&str> = source.lines().collect();

        let mut visitor = ScrollAreaInnerRectLeakVisitor {
            file_path: path.to_path_buf(),
            source_lines: lines,
            violations: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct ScrollAreaInnerRectLeakVisitor<'a> {
    file_path: PathBuf,
    source_lines: Vec<&'a str>,
    violations: Vec<Violation>,
}

impl ScrollAreaInnerRectLeakVisitor<'_> {
    fn is_suppressed(&self, line: usize) -> bool {
        if line > 1
            && let Some(prev) = self.source_lines.get(line - 2)
        {
            let trimmed = prev.trim();
            return trimmed.starts_with("/// WHY: allow(scrollarea_inner_rect_leak)")
                || trimmed.starts_with("/* WHY: allow(scrollarea_inner_rect_leak)");
        }
        false
    }

    fn is_rect_field(expr: &syn::Expr) -> bool {
        matches!(
            expr,
            syn::Expr::Field(field) if matches!(field.member, syn::Member::Named(ref ident) if ident == "rect")
        )
    }

    fn is_inner_rect_field(expr: &syn::Expr) -> bool {
        matches!(
            expr,
            syn::Expr::Field(field) if matches!(field.member, syn::Member::Named(ref ident) if ident == "inner_rect")
        )
    }
}

impl<'ast, 'a> Visit<'ast> for ScrollAreaInnerRectLeakVisitor<'a> {
    fn visit_expr_assign(&mut self, node: &'ast syn::ExprAssign) {
        if Self::is_rect_field(&node.left) && Self::is_inner_rect_field(&node.right) {
            let (line, column) = LinterParserOps::span_location(node.eq_token.spans[0]);
            if !self.is_suppressed(line) {
                self.violations.push(Violation {
                    file: self.file_path.clone(),
                    line,
                    column,
                    message:
                        "Do not assign `ScrollArea::inner_rect` directly to a parent-facing `rect`. That leaks unclipped content size into the parent layout and can cause ratchet growth (expand but not shrink)."
                            .to_string(),
                });
            }
        }
        syn::visit::visit_expr_assign(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_inner_rect_to_rect_assignment() {
        let syntax = syn::parse_file(
            r#"
            fn demo(scroll_output: egui::scroll_area::ScrollAreaOutput<egui::Response>) {
                let mut out_res = scroll_output.inner;
                out_res.rect = scroll_output.inner_rect;
            }
            "#,
        )
        .expect("test source should parse");

        let violations = ScrollAreaInnerRectLeakOps::lint(Path::new("/tmp/demo.rs"), &syntax);
        assert_eq!(violations.len(), 1);
    }
}

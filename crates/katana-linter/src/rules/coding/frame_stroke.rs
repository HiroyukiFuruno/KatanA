use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

/// WHY: Detects border/stroke patterns that can cause layout jitter on hover.
///
/// In egui, strokes on `Frame` or `painter().rect_stroke()` add pixels OUTSIDE
/// the element's rect. Without expansion compensation (box-sizing: border-box),
/// hover-time borders cause adjacent elements to shift.
///
/// Allowed patterns: theme_bridge (centralized expansion config), explicit suppression.
pub struct FrameStrokeOps;

impl FrameStrokeOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        let lines: Vec<&str> = source.lines().collect();
        let mut visitor = FrameStrokeVisitor {
            file_path: path.to_path_buf(),
            source_lines: lines,
            violations: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct FrameStrokeVisitor<'a> {
    file_path: PathBuf,
    source_lines: Vec<&'a str>,
    violations: Vec<Violation>,
}

impl FrameStrokeVisitor<'_> {
    fn is_suppressed(&self, line: usize) -> bool {
        if line > 1
            && let Some(prev) = self.source_lines.get(line - 2)
        {
            return prev.contains("allow(frame_stroke)");
        }
        false
    }

    fn is_theme_bridge(&self) -> bool {
        self.file_path
            .to_str()
            .is_some_and(|p| p.contains("theme_bridge"))
    }
}

impl<'ast> Visit<'ast> for FrameStrokeVisitor<'_> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        // WHY: theme_bridge is the centralized expansion config — skip it to avoid false positives.
        if self.is_theme_bridge() {
            return;
        }

        let method_name = node.method.to_string();

        // WHY: painter().rect_stroke() adds pixels outside the rect without expansion
        // WHY: compensation, causing border inflation on hover. Detect and require shrink().
        if method_name == "rect_stroke" {
            let (line, column) = LinterParserOps::span_location(node.method.span());
            if !self.is_suppressed(line) {
                self.violations.push(Violation {
                    file: self.file_path.clone(),
                    line,
                    column,
                    message: "Use `rect_filled` or compensate stroke width with `rect.shrink(stroke_width)` before `rect_stroke`. Add `// allow(frame_stroke)` above to suppress.".to_string(),
                });
            }
        }

        // WHY: .stroke() on a Frame builder adds pixels outside the widget rect. Detect this
        // WHY: by walking the receiver chain for Frame builder method names (fill/margin etc).
        if method_name == "stroke" && !node.args.is_empty() && Self::is_frame_context(node) {
            let (line, column) = LinterParserOps::span_location(node.method.span());
            if !self.is_suppressed(line) {
                self.violations.push(Violation {
                        file: self.file_path.clone(),
                        line,
                        column,
                        message: "Frame `.stroke()` can cause layout jitter on hover. Use theme visuals (expansion-compensated) or wrap in a widget. Add `// allow(frame_stroke)` above to suppress.".to_string(),
                    });
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

impl FrameStrokeVisitor<'_> {
    /// Heuristic: walk the receiver chain to see if this `.stroke()` is called
    /// on a `Frame` builder (e.g., `Frame::NONE.fill(...).stroke(...)`).
    fn is_frame_context(node: &syn::ExprMethodCall) -> bool {
        let mut expr = &*node.receiver;
        loop {
            match expr {
                syn::Expr::MethodCall(inner) => {
                    let name = inner.method.to_string();
                    // WHY: These are the chainable Frame builder methods — keep walking up.
                    if matches!(
                        name.as_str(),
                        "fill"
                            | "inner_margin"
                            | "outer_margin"
                            | "corner_radius"
                            | "rounding"
                            | "shadow"
                    ) {
                        expr = &inner.receiver;
                        continue;
                    }
                    return false;
                }
                syn::Expr::Path(path) => {
                    let path_str = path
                        .path
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");
                    return path_str.contains("Frame");
                }
                syn::Expr::Field(field) => {
                    // WHY: Frame::NONE and .frame are sentinel field accesses indicating a Frame builder start.
                    if let syn::Member::Named(ident) = &field.member
                        && (ident == "NONE" || ident == "frame")
                    {
                        return true;
                    }
                    expr = &field.base;
                    continue;
                }
                syn::Expr::Call(call) => {
                    if let syn::Expr::Path(path) = &*call.func {
                        let path_str = path
                            .path
                            .segments
                            .iter()
                            .map(|s| s.ident.to_string())
                            .collect::<Vec<_>>()
                            .join("::");
                        return path_str.contains("Frame");
                    }
                    return false;
                }
                _ => return false,
            }
        }
    }
}

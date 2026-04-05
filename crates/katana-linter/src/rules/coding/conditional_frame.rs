use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

/* WHY: Detects egui widgets that conditionally show a frame only on hover.

 These widgets cause layout jitter because their size changes between
 inactive and hovered states:

 Pattern that triggers this rule:
 ```ignore
    VIOLATION: frame appears only on hover
    ui.selectable_label(false, "Text");
 ```
*/
pub struct ConditionalFrameOps;

impl ConditionalFrameOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        let lines: Vec<&str> = source.lines().collect();

        let mut visitor = ConditionalFrameVisitor {
            file_path: path.to_path_buf(),
            source_lines: lines,
            violations: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct ConditionalFrameVisitor<'a> {
    file_path: PathBuf,
    source_lines: Vec<&'a str>,
    violations: Vec<Violation>,
}

impl ConditionalFrameVisitor<'_> {
    fn is_suppressed(&self, line: usize) -> bool {
        if line > 1
            && let Some(prev) = self.source_lines.get(line - 2)
        {
            let trimmed = prev.trim();
            return trimmed.starts_with("/// WHY: allow(conditional_frame)")
                || trimmed.starts_with("/* WHY: allow(conditional_frame)");
        }
        false
    }
}

impl<'ast> Visit<'ast> for ConditionalFrameVisitor<'_> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        match method_name.as_str() {
            /* WHY: selectable_label/selectable_value/menu_button all conditionally show */
            /* WHY: the frame based on hover state, causing layout jitter. Match all three. */
            "selectable_label" | "selectable_value" | "menu_button" => {
                let (line, column) = LinterParserOps::span_location(node.method.span());
                if !self.is_suppressed(line) {
                    let msg = match method_name.as_str() {
                        "selectable_label" => {
                            "Use `Button::selectable(selected, text).frame_when_inactive(true)` \
                             to prevent hover-jitter."
                        }
                        "selectable_value" => {
                            "Use `Button::selectable(val == variant, text).frame_when_inactive(true)` \
                             to prevent hover-jitter."
                        }
                        "menu_button" => {
                            "menu_button shows a frame only on hover, causing layout jitter. \
                             Wrap in a custom button with stable frame."
                        }
                        _ => unreachable!(),
                    };
                    self.violations.push(Violation {
                        file: self.file_path.clone(),
                        line,
                        column,
                        message: msg.to_string(),
                    });
                }
            }
            _ => {}
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

/* WHY: Detects `egui::Button::image(...)` calls that lack a `.fill(...)` in their method chain.

Without an explicit `.fill()`, icon-only buttons inherit `weak_bg_fill` from the current
widget state: `panel_bg` at rest, `highlight_bg` on hover. Other icon buttons that use
`.fill(icon_bg)` (TRANSPARENT in dark mode, a light gray in light mode) show only the accent
border on hover, with no fill change. This inconsistency makes half the icon buttons look
"highlighted" on hover while others show only a border, breaking visual uniformity.

Pattern that triggers this rule:
```ignore
/* WHY: VIOLATION: no .fill() */
ui.add(egui::Button::image(icon));

/* WHY: OK: explicit transparent background */
ui.add(egui::Button::image(icon).fill(icon_bg));
```*/
pub struct IconButtonFillOps;

impl IconButtonFillOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        /* WHY: icon/mod.rs is the canonical factory for Button::image — it is the
        only sanctioned call-site. All other callers must use Icon::button() or
        Icon::selected_button() instead of constructing Button::image directly. */
        if path.to_string_lossy().replace('\\', "/").contains("icon/mod.rs") {
            return Vec::new();
        }

        let mut visitor = IconButtonFillVisitor {
            file_path: path.to_path_buf(),
            violations: Vec::new(),
            in_fill_receiver: false,
        };
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct IconButtonFillVisitor {
    file_path: PathBuf,
    violations: Vec<Violation>,
    /* WHY: When visiting the receiver of a `.fill(x)` call we set this flag so that any */
    /// `Button::image(...)` found inside is known to be already under a fill.
    in_fill_receiver: bool,
}

impl IconButtonFillVisitor {
    fn is_button_image_call(call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(expr_path) = call.func.as_ref() else {
            return false;
        };
        let segs: Vec<String> = expr_path
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        /* WHY: Match only `Button::image` (pure SVG, no text) — `Button::image_and_text` is */
        /* WHY: ambiguous (may carry visible text) so we enforce fill there via code review only. */
        segs.last().map(|s| s == "image").unwrap_or(false) && segs.iter().any(|s| s == "Button")
    }
}

impl<'ast> Visit<'ast> for IconButtonFillVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "fill" {
            /* WHY: Receiver of `.fill(x)` is already chained to a fill — toggle flag so that */
            /* WHY: any `Button::image` inside is not double-reported. Args are not in the chain. */
            let was_in_fill = self.in_fill_receiver;
            self.in_fill_receiver = true;
            self.visit_expr(&node.receiver);
            self.in_fill_receiver = was_in_fill;

            for arg in &node.args {
                self.visit_expr(arg);
            }
            return;
        }

        /* WHY: Non-fill method calls must still descend so the visitor reaches `Button::image` */
        /* WHY: deeper in chains like `.on_hover_text(...).fill(x)` wrapping patterns. */
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if Self::is_button_image_call(node) && !self.in_fill_receiver {
            let span = match node.func.as_ref() {
                syn::Expr::Path(p) => p.path.segments.last().map(|s| s.ident.span()),
                _ => None,
            };
            if let Some(span) = span {
                let (line, column) = LinterParserOps::span_location(span);
                self.violations.push(Violation {
                    file: self.file_path.clone(),
                    line,
                    column,
                    message: "Icon-only `Button::image()` needs an explicit `.fill(icon_bg)` \
                              to ensure consistent background across all hover states. \
                              Use `fill(if ui.visuals().dark_mode { TRANSPARENT } else { from_gray(LIGHT_MODE_ICON_BG) })`."
                        .to_string(),
                });
            }
        }

        syn::visit::visit_expr_call(self, node);
    }
}

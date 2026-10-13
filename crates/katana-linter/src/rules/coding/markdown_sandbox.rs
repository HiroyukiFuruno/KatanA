use crate::Violation;
use std::path::Path;
use syn::visit::Visit;

/* WHY: Prevent Ratchet Layout Bug in Markdown Previews
   egui_commonmark internally uses `ScrollArea`s that greedily consume available width.
   If not strictly sandboxed via `ui.scope(|ui| ui.set_max_width(...))`,
   the min_rect will permanently expand, destroying split layouts.
*/
pub struct MarkdownSandboxOps;

impl MarkdownSandboxOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = MarkdownSandboxVisitor {
            violations: Vec::new(),
            found_viewer_invocation: false,
            found_set_max_width: false,
        };
        visitor.visit_file(syntax);

        /* WHY: If we found a viewer invocation but NO max_width usage in the file, error. */
        if visitor.found_viewer_invocation && !visitor.found_set_max_width {
            visitor.violations.push(Violation {
                file: path.to_path_buf(),
                line: 1, // file-level violation
                column: 1,
                message: "Layout Ratchet Bug: `CommonMarkViewer` is invoked but `set_max_width` is missing in this file. You MUST sandbox the viewer call within `ui.scope(|ui| { ui.set_max_width(ui.available_width()); ... })`.".to_string(),
            });
        }
        visitor.violations
    }
}

struct MarkdownSandboxVisitor {
    violations: Vec<Violation>,
    found_viewer_invocation: bool,
    found_set_max_width: bool,
}

impl<'ast> Visit<'ast> for MarkdownSandboxVisitor {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        if node
            .segments
            .last()
            .is_some_and(|s| s.ident == "CommonMarkViewer")
        {
            self.found_viewer_invocation = true;
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "set_max_width" {
            self.found_set_max_width = true;
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

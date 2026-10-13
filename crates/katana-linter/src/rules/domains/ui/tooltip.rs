use crate::Violation;
use crate::utils::{collect_rs_files, LinterParserOps};
use quote::quote;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

fn is_using_icon(node: &syn::ExprMethodCall) -> bool {
    node.args.iter().any(|arg| {
        let s = quote!(#arg).to_string();
        s.contains("Icon") || s.contains("icon")
    })
}

struct TooltipVisitor<'a> {
    file_path: &'a Path,
    violations: Vec<Violation>,
    in_tooltip_chain: bool,
}

impl<'a, 'ast> Visit<'ast> for TooltipVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        /* WHY: eframe/egui pattern is: ui.button(...).on_hover_text(...)
           In AST, on_hover_text is the OUTER call, and button is the RECEIVER. */

        let is_tooltip_method = matches!(
            method_name.as_str(),
            "on_hover_text" | "on_hover_ui" | "on_disabled_hover_text" | "on_hover_text_at"
        );

        let old_in_chain = self.in_tooltip_chain;
        if is_tooltip_method {
            self.in_tooltip_chain = true;
        }

        visit::visit_expr(self, &node.receiver);

        for arg in &node.args {
            visit::visit_expr(self, arg);
        }

        self.in_tooltip_chain = old_in_chain;

        if matches!(
            method_name.as_str(),
            "button" | "menu_image_button" | "image_button"
        ) && !self.in_tooltip_chain
            && is_using_icon(node)
        {
            let (line, col) = LinterParserOps::span_location(node.span());
            self.violations.push(Violation {
                file: self.file_path.to_path_buf(),
                line,
                column: col,
                message: format!(
                    "Missing tooltip for icon button `{}`. Toolbar buttons must provide `on_hover_text(...)` for accessibility.",
                    method_name
                ),
            });
        }
    }
}

pub struct TooltipOps;

impl TooltipOps {
    pub fn lint_egui_tooltips(workspace_root: &Path) -> Vec<Violation> {
        let views_dir = workspace_root.join("crates/katana-ui/src/views");
        let files = collect_rs_files(&views_dir);
        let mut violations = Vec::new();
    
        for file in files {
            let Ok(ast) = LinterParserOps::parse_file(&file) else {
                continue;
            };
            let mut visitor = TooltipVisitor {
                file_path: &file,
                violations: Vec::new(),
                in_tooltip_chain: false,
            };
            visitor.visit_file(&ast);
            violations.extend(visitor.violations);
        }
    
        violations
    }
}

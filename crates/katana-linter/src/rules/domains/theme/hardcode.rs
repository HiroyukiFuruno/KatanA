use crate::Violation;
use crate::utils::{LinterFileOps, LinterParserOps};
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

struct HardcodedColorVisitor<'a> {
    file_path: &'a Path,
    violations: Vec<Violation>,
}

pub struct HardcodedColorOps;

impl HardcodedColorOps {
    pub fn lint(workspace_root: &Path) -> Vec<Violation> {
        let ui_dir = workspace_root.join("crates/katana-ui/src");
        let ui_files = LinterFileOps::collect_rs_files(&ui_dir);
        let mut violations = Vec::new();

        for file in ui_files {
            let path_str = file.to_string_lossy();
            if path_str.contains("theme_bridge") || path_str.contains("svg_loader") {
                continue;
            }

            let Ok(ast) = LinterParserOps::parse_file(&file) else {
                continue;
            };
            let mut visitor = HardcodedColorVisitor {
                file_path: &file,
                violations: Vec::new(),
            };
            visitor.visit_file(&ast);
            violations.extend(visitor.violations);
        }

        violations
    }

    fn is_forbidden_color_constant(path_str: &str) -> bool {
        let forbidden = vec![
            "Color32::RED",
            "Color32::GREEN",
            "Color32::BLUE",
            "Color32::YELLOW",
            "Color32::WHITE",
            "Color32::BLACK",
            "Color32::TRANSPARENT",
            "Color32::LIGHT_GRAY",
            "Color32::DARK_GRAY",
        ];
        forbidden.iter().any(|&c| path_str.ends_with(c))
    }
}

impl<'a, 'ast> Visit<'ast> for HardcodedColorVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(expr_path) = &*node.func {
            let path_str = expr_path
                .path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if path_str.contains("Color32::from_rgb")
                || path_str.contains("Color32::from_rgba")
                || path_str.contains("Color32::from_black_alpha")
                || path_str.contains("Color32::from_white_alpha")
                || path_str.contains("Color32::from_gray")
            {
                let (line, col) = LinterParserOps::span_location(node.span());
                self.violations.push(Violation {
                    file: self.file_path.to_path_buf(),
                    line,
                    column: col,
                    message: format!(
                        "Hardcoded color detected: `{path_str}`. Please define this color in `ThemeColors` and use it via the theme system."
                    ),
                });
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        let path_str = node
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");

        if HardcodedColorOps::is_forbidden_color_constant(&path_str) {
            let (line, col) = LinterParserOps::span_location(node.span());
            self.violations.push(Violation {
                file: self.file_path.to_path_buf(),
                line,
                column: col,
                message: format!(
                    "Hardcoded color constant detected: `{path_str}`. Please define this color in `ThemeColors` and use it via the theme system."
                ),
            });
        }
        syn::visit::visit_expr_path(self, node);
    }
}

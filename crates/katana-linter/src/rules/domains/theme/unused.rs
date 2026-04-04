use crate::Violation;
use crate::utils::{LinterFileOps, LinterParserOps};
use std::collections::HashSet;
use std::path::Path;
use syn::visit::Visit;

struct ThemePropertyExtractor {
    properties: Vec<(String, usize, usize)>,
}

pub struct UnusedThemeColorOps;

impl UnusedThemeColorOps {
    pub fn lint(workspace_root: &Path) -> Vec<Violation> {
        let types_rs_path = workspace_root.join("crates/katana-platform/src/theme/types.rs");
        let types_ast = Self::get_types_ast(&types_rs_path);

        let mut extractor = ThemePropertyExtractor {
            properties: Vec::new(),
        };
        extractor.visit_file(&types_ast);

        let (general_access, settings_access) = Self::scan_ui_files(workspace_root);
        let mut violations = Vec::new();

        for (prop_name, line, col) in extractor.properties {
            Self::check_unused_property(
                &prop_name,
                line,
                col,
                &types_rs_path,
                &general_access,
                &mut violations,
            );
            Self::check_unexposed_property(
                &prop_name,
                line,
                col,
                &types_rs_path,
                &settings_access,
                &mut violations,
            );
        }
        violations
    }

    fn get_types_ast(path: &Path) -> syn::File {
        LinterParserOps::parse_file(path).unwrap_or_else(|e| {
            panic!("Failed to parse theme/types.rs for ast_linter_no_unused_theme_colors: {e:?}")
        })
    }

    fn scan_ui_files(workspace_root: &Path) -> (FieldAccessVisitor, FieldAccessVisitor) {
        let ui_dir = workspace_root.join("crates/katana-ui/src");
        let mut general_access = FieldAccessVisitor {
            used_fields: HashSet::new(),
        };
        let mut settings_access = FieldAccessVisitor {
            used_fields: HashSet::new(),
        };

        for file in LinterFileOps::collect_rs_files(&ui_dir) {
            let Ok(ast) = LinterParserOps::parse_file(&file) else {
                continue;
            };
            general_access.visit_file(&ast);
            if file.file_name().unwrap_or_default() == "theme.rs" {
                settings_access.visit_file(&ast);
            }
        }
        (general_access, settings_access)
    }

    fn check_unused_property(
        prop_name: &str,
        line: usize,
        col: usize,
        types_rs_path: &Path,
        general_access: &FieldAccessVisitor,
        violations: &mut Vec<Violation>,
    ) {
        if !general_access.used_fields.contains(prop_name) {
            violations.push(Violation {
                file: types_rs_path.to_path_buf(),
                line,
                column: col,
                message: format!(
                    "Theme color property `{prop_name}` is defined in ThemeColors but never accessed in UI code. Please wire it up to `katana-ui` or remove it."
                ),
            });
        }
    }

    fn check_unexposed_property(
        prop_name: &str,
        line: usize,
        col: usize,
        types_rs_path: &Path,
        settings_access: &FieldAccessVisitor,
        violations: &mut Vec<Violation>,
    ) {
        if !settings_access.used_fields.contains(prop_name) {
            violations.push(Violation {
                file: types_rs_path.to_path_buf(),
                line,
                column: col,
                message: format!(
                    "Theme color property `{prop_name}` is not exposed in `theme.rs`. All custom colors must be editable by the user."
                ),
            });
        }
    }
}

impl<'ast> Visit<'ast> for ThemePropertyExtractor {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.extract_color_properties(node);
        syn::visit::visit_item_struct(self, node);
    }
}

impl ThemePropertyExtractor {
    fn extract_color_properties(&mut self, node: &syn::ItemStruct) {
        let name = node.ident.to_string();
        if name != "ThemeColors"
            && name != "SystemColors"
            && name != "CodeColors"
            && name != "PreviewColors"
        {
            return;
        }

        for field in &node.fields {
            let Some(ident) = &field.ident else { continue };
            let syn::Type::Path(type_path) = &field.ty else {
                continue;
            };
            let Some(segment) = type_path.path.segments.last() else {
                continue;
            };

            let type_name = segment.ident.to_string();
            if type_name == "Rgb" || type_name == "Rgba" {
                let (line, col) = LinterParserOps::span_location(ident.span());
                self.properties.push((ident.to_string(), line, col));
            }
        }
    }
}

struct FieldAccessVisitor {
    used_fields: HashSet<String>,
}

impl<'ast> Visit<'ast> for FieldAccessVisitor {
    fn visit_expr_field(&mut self, node: &'ast syn::ExprField) {
        if let syn::Member::Named(ident) = &node.member {
            self.used_fields.insert(ident.to_string());
        }
        syn::visit::visit_expr_field(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if node.path.is_ident("vec") {
            let Ok(exprs) = node.parse_body_with(
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
            ) else {
                return syn::visit::visit_macro(self, node);
            };
            for expr in exprs {
                self.visit_expr(&expr);
            }
        }
        syn::visit::visit_macro(self, node);
    }
}

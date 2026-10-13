use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

use super::helpers::I18nHelperOps;

pub struct IconOps;

impl IconOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = IconFacadeVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct IconFacadeVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl IconFacadeVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_expr_for_raw_icon(&mut self, expr: &syn::Expr, context: &str) {
        match expr {
            syn::Expr::Lit(expr_lit) => {
                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                    let value = lit_str.value();
                    if I18nHelperOps::is_raw_icon(&value) {
                        let (line, column) = LinterParserOps::span_location(lit_str.span());
                        self.violations.push(Violation {
                            file: self.file.clone(),
                            line,
                            column,
                            message: format!(
                                "Raw icon string \"{value}\" detected in {context}. \
                                 Please use `Icon::Name.as_str()` instead."
                            ),
                        });
                    }
                }
            }
            syn::Expr::Reference(expr_ref) => self.check_expr_for_raw_icon(&expr_ref.expr, context),
            syn::Expr::Paren(expr_paren) => self.check_expr_for_raw_icon(&expr_paren.expr, context),
            syn::Expr::Group(expr_group) => self.check_expr_for_raw_icon(&expr_group.expr, context),
            _ => {}
        }
    }
}

impl<'ast> Visit<'ast> for IconFacadeVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if LinterParserOps::has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if I18nHelperOps::ui_methods().contains(&method_name.as_str()) {
            for arg in node.args.iter() {
                self.check_expr_for_raw_icon(arg, &format!("{}()", method_name));
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        self.check_ui_func_call(node);
        syn::visit::visit_expr_call(self, node);
    }
}

impl IconFacadeVisitor {
    fn check_ui_func_call(&mut self, node: &syn::ExprCall) {
        let syn::Expr::Path(expr_path) = &*node.func else {
            return;
        };
        let Some(last_segment) = expr_path.path.segments.last() else {
            return;
        };

        let func_name = last_segment.ident.to_string();
        if !I18nHelperOps::ui_functions().contains(&func_name.as_str()) {
            return;
        }

        let Some(type_name) = I18nHelperOps::extract_type_from_call(&node.func) else {
            return;
        };
        if !I18nHelperOps::ui_types_for_new().contains(&type_name.as_str()) {
            return;
        }

        for arg in node.args.iter() {
            self.check_expr_for_raw_icon(arg, &format!("{}::{}", type_name, func_name));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn lint_i18n_detects_raw_icon_in_label() {
        let code = r#"fn render(ui: &mut Ui) { ui.label("🔄"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = IconOps::lint(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("Raw icon string"));
    }
}

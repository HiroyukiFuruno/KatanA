use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

use super::helpers::I18nHelperOps;

pub struct I18nOps;

impl I18nOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = I18nHardcodeVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct I18nHardcodeVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl I18nHardcodeVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_string_literal_args(
        &mut self,
        args: &syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>,
        method_name: &str,
    ) {
        for arg in args.iter() {
            self.check_expr_for_hardcoded_string(arg, method_name);
        }
    }

    fn check_expr_for_hardcoded_string(&mut self, expr: &syn::Expr, method_name: &str) {
        match expr {
            syn::Expr::Lit(expr_lit) => self.check_lit_str(expr_lit, method_name),
            syn::Expr::Macro(expr_macro) => self.check_format_macro(expr_macro, method_name),
            syn::Expr::Reference(expr_ref) => {
                self.check_expr_for_hardcoded_string(&expr_ref.expr, method_name)
            }
            syn::Expr::Paren(expr_paren) => {
                self.check_expr_for_hardcoded_string(&expr_paren.expr, method_name)
            }
            syn::Expr::Group(expr_group) => {
                self.check_expr_for_hardcoded_string(&expr_group.expr, method_name)
            }
            _ => {}
        }
    }

    fn check_lit_str(&mut self, expr_lit: &syn::ExprLit, method_name: &str) {
        let syn::Lit::Str(lit_str) = &expr_lit.lit else {
            return;
        };
        let value = lit_str.value();
        if !LinterParserOps::is_allowed_string(&value) {
            let (line, column) = LinterParserOps::span_location(lit_str.span());
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message: format!(
                    "Hardcoded string \"{value}\" detected in {method_name}().\
                     Please use i18n::t() or i18n::tf()."
                ),
            });
        }
    }

    fn check_format_macro(&mut self, expr_macro: &syn::ExprMacro, method_name: &str) {
        if !I18nHelperOps::is_format_macro(&expr_macro.mac) {
            return;
        }

        let (line, column) = LinterParserOps::span_location(
            expr_macro
                .mac
                .path
                .segments
                .last()
                .map(|it| it.ident.span())
                .unwrap_or_else(proc_macro2::Span::call_site),
        );

        self.violations.push(Violation {
            file: self.file.clone(),
            line,
            column,
            message: format!(
                "Hardcoded string synthesis using format!() detected in {method_name}().\
                 Please use i18n::tf()."
            ),
        });
    }

    fn check_call_for_ui_violation(&mut self, node: &syn::ExprCall) {
        let syn::Expr::Path(expr_path) = &*node.func else {
            return;
        };
        let last_segment = expr_path
            .path
            .segments
            .last()
            .expect("syn::Path always has at least one segment");
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
        self.check_string_literal_args(&node.args, &format!("{type_name}::{func_name}"));
    }
}

impl<'ast> Visit<'ast> for I18nHardcodeVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if LinterParserOps::has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if I18nHelperOps::ui_methods().contains(&method_name.as_str()) {
            self.check_string_literal_args(&node.args, &method_name);
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        self.check_call_for_ui_violation(node);
        syn::visit::visit_expr_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_i18n_allows_symbol_strings() {
        let code = r#"fn render(ui: &mut Ui) { ui.label("x"); ui.label("●"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = I18nOps::lint(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn lint_i18n_detects_richtext_new_via_path_call() {
        let code = r#"
            fn render(ui: &mut Ui) {
                ui.label(egui::RichText::new("Hardcoded Text"));
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = I18nOps::lint(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }
}

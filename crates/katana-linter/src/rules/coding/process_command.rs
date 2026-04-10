use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub struct ProcessCommandOps;

impl ProcessCommandOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = ProcessCommandVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct ProcessCommandVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl ProcessCommandVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn report_violation_if_needed(&mut self, node: &syn::ExprCall) {
        /* WHY: Normalize to forward slashes for cross-platform path comparison */
        let file_str = self.file.to_string_lossy().replace('\\', "/");
        /* WHY: Allow the central ProcessService facade to use Command::new */
        if file_str.contains("system/process.rs") {
            return;
        }
        let (line, column) = LinterParserOps::span_location(node.span());
        self.violations.push(Violation {
            file: self.file.clone(),
            line,
            column,
            message: "Use of `Command::new` detected. You MUST use \
                      `crate::system::ProcessService::create_command` instead to \
                      enforce cross-platform silent execution (CREATE_NO_WINDOW) policies \
                      and prevent console windows from popping up on Windows."
                .to_string(),
        });
    }
}

impl<'ast> Visit<'ast> for ProcessCommandVisitor {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(expr_path) = &*node.func {
            let segments: Vec<_> = expr_path.path.segments.iter().collect();
            if segments.len() >= 2 {
                let last = segments[segments.len() - 1];
                let prev = segments[segments.len() - 2];
                if last.ident == "new" && prev.ident == "Command" {
                    self.report_violation_if_needed(node);
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if LinterParserOps::has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if LinterParserOps::has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if LinterParserOps::has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_impl_item_fn(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_raw_command_new() {
        let code = r#"fn call_process() { let mut cmd = std::process::Command::new("ls"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = ProcessCommandOps::lint(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("ProcessService::create_command")
        );
    }

    #[test]
    fn ignores_command_in_process_service() {
        let code = r#"fn call_process() { let mut cmd = std::process::Command::new("ls"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = ProcessCommandOps::lint(&PathBuf::from("system/process.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }
}

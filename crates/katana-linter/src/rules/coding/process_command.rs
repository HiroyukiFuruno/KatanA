use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub struct ProcessCommandOps;

/* WHY: Facade files that are allowed to call `std::process::Command::new` directly because
 * they apply the Windows `CREATE_NO_WINDOW` flag (or the documented Java visibility exception)
 * to every constructed `Command`. Paths are checked as repo-relative suffixes after
 * normalising backslashes to forward slashes, so they work on both POSIX and Windows hosts.
 * Adding a new facade requires updating this list AND extending the
 * `headless-process-enforcement` OpenSpec capability. */
pub const DEFAULT_ALLOWLIST: &[&str] = &[
    "crates/katana-core/src/system/process.rs",
    "crates/katana-ui/build_support/process.rs",
];

impl ProcessCommandOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        Self::lint_with_allowlist(path, syntax, DEFAULT_ALLOWLIST)
    }

    /* WHY: Allowlist-injecting entry point so external rule packages (notably the upcoming
     * `katana-ast-lint` crate produced by the `extract-katana-ast-lint` change) can keep
     * repo-local concerns in their adapter and never hard-code KatanA paths inside the rule. */
    pub fn lint_with_allowlist(
        path: &Path,
        syntax: &syn::File,
        allowlist: &[&str],
    ) -> Vec<Violation> {
        if is_allowed_facade(path, allowlist) {
            return Vec::new();
        }
        let mut visitor = ProcessCommandVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }

    /* WHY: Build scripts include the headless helper via `include!()`, but the same lint
     * still needs to confirm no raw `Command::new` slips into a `build.rs` without going
     * through `create_build_command`. The default allowlist already covers the facade, so
     * a build script that calls `Command::new` directly is flagged like any other file. */
    pub fn lint_build_script(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        Self::lint_with_allowlist(path, syntax, DEFAULT_ALLOWLIST)
    }
}

fn is_allowed_facade(path: &Path, allowlist: &[&str]) -> bool {
    let normalised = path.to_string_lossy().replace('\\', "/");
    allowlist
        .iter()
        .any(|allowed| normalised.ends_with(allowed))
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

    fn report_violation(&mut self, node: &syn::ExprCall) {
        let (line, column) = LinterParserOps::span_location(node.span());
        self.violations.push(Violation {
            file: self.file.clone(),
            line,
            column,
            message: "Use of `Command::new` detected. You MUST route process spawning through \
                      a sanctioned headless facade. In production code, call \
                      `katana_core::system::ProcessService::create_command`. In a `build.rs`, \
                      `include!(\"build_support/process.rs\")` and call \
                      `create_build_command`. This enforces the Windows `CREATE_NO_WINDOW` \
                      policy and prevents console windows from popping up on Windows. \
                      For Java specifically, prefer the Windows GUI-subsystem `javaw` \
                      launcher together with `create_command` instead of bypassing the \
                      flag — `Stdio::piped()` does not suppress console allocation when a \
                      GUI parent spawns a console-subsystem child."
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
                    self.report_violation(node);
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
    fn ignores_command_in_process_service_facade() {
        let code = r#"fn call_process() { let mut cmd = std::process::Command::new("ls"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = ProcessCommandOps::lint(
            &PathBuf::from("crates/katana-core/src/system/process.rs"),
            &syntax,
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn ignores_command_in_build_support_facade() {
        let code = r#"fn helper() { let _ = std::process::Command::new("rustc"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = ProcessCommandOps::lint(
            &PathBuf::from("crates/katana-ui/build_support/process.rs"),
            &syntax,
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn detects_raw_command_new_in_build_script() {
        let code = r#"fn main() { let _ = std::process::Command::new("rustc"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = ProcessCommandOps::lint_build_script(
            &PathBuf::from("crates/katana-ui/build.rs"),
            &syntax,
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("create_build_command"));
    }

    #[test]
    fn detects_raw_command_new_in_scripts() {
        let code = r#"fn launch() { let _ = std::process::Command::new("ffmpeg"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = ProcessCommandOps::lint(
            &PathBuf::from("scripts/screenshot/src/executor_harness.rs"),
            &syntax,
        );
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn allowlist_does_not_match_substring_only() {
        /* WHY: A file at `crates/some-feature/src/process.rs` must NOT be silently allowed
         * just because its name contains "process.rs". The allowlist anchors on the
         * full repo-relative suffix. */
        let code = r#"fn foo() { let _ = std::process::Command::new("ls"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = ProcessCommandOps::lint(
            &PathBuf::from("crates/some-feature/src/process.rs"),
            &syntax,
        );
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn allowlist_can_be_overridden() {
        let code = r#"fn foo() { let _ = std::process::Command::new("ls"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let custom = ["my/own/facade.rs"];
        let violations = ProcessCommandOps::lint_with_allowlist(
            &PathBuf::from("my/own/facade.rs"),
            &syntax,
            &custom,
        );
        assert_eq!(violations.len(), 0);
    }
}

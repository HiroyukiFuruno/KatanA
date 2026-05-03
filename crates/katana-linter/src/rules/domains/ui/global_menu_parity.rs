use crate::Violation;
use crate::utils::LinterParserOps;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use syn::visit::{self, Visit};

struct AppActionVisitor {
    actions: HashSet<String>,
}

impl<'ast> Visit<'ast> for AppActionVisitor {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        if node.segments.len() >= 2 {
            let last_two: Vec<_> = node.segments.iter().rev().take(2).collect();
            if last_two[1].ident == "AppAction" {
                self.actions.insert(last_two[0].ident.to_string());
            }
        }
        visit::visit_path(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let tokens_str = node.tokens.to_string();
        /* WHY: Tokens are separated by spaces, e.g. "AppAction :: Quit" */
        let parts: Vec<&str> = tokens_str.split_whitespace().collect();
        for i in 0..parts.len() {
            if parts[i] == "AppAction" && i + 2 < parts.len() && parts[i + 1] == "::" {
                /* WHY: If it's a valid identifier, insert it */
                let ident = parts[i + 2];
                if ident.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    self.actions.insert(ident.to_string());
                }
            }
        }
        visit::visit_macro(self, node);
    }
}

pub struct GlobalMenuParityOps;

impl GlobalMenuParityOps {
    pub fn lint(workspace_root: &Path) -> Vec<Violation> {
        let global_menu_paths = Self::global_menu_paths(workspace_root);
        let native_menu_path = workspace_root.join("crates/katana-ui/src/native_menu/mod.rs");

        if global_menu_paths.is_empty() {
            return vec![];
        }
        let Ok(native_ast) = LinterParserOps::parse_file(&native_menu_path) else {
            return vec![];
        };

        let mut global_visitor = AppActionVisitor {
            actions: HashSet::new(),
        };
        for path in &global_menu_paths {
            let Ok(global_ast) = LinterParserOps::parse_file(path) else {
                return vec![];
            };
            global_visitor.visit_file(&global_ast);
        }

        let mut native_visitor = AppActionVisitor {
            actions: HashSet::new(),
        };
        native_visitor.visit_file(&native_ast);

        /* WHY: Remove exceptions that are intentionally asymmetric between platforms */
        global_visitor.actions.remove("Quit"); // Quit is handled natively by NSApplication on macOS
        native_visitor.actions.remove("None"); // None is returned for unhandled tags

        let mut diff1: Vec<_> = global_visitor
            .actions
            .difference(&native_visitor.actions)
            .collect();
        let mut diff2: Vec<_> = native_visitor
            .actions
            .difference(&global_visitor.actions)
            .collect();
        diff1.sort();
        diff2.sort();

        let mut violations = Vec::new();
        if !diff1.is_empty() || !diff2.is_empty() {
            let msg = format!(
                "Menu parity violation! AppAction items mismatch between Windows/Linux (global_menu*.rs) and macOS (native_menu). In global_menu*.rs but not in native_menu/mod.rs: {:?}. In native_menu/mod.rs but not in global_menu*.rs: {:?}",
                diff1, diff2
            );
            violations.push(Violation {
                file: global_menu_paths[0].clone(),
                line: 1,
                column: 1,
                message: msg.clone(),
            });
            violations.push(Violation {
                file: native_menu_path,
                line: 1,
                column: 1,
                message: msg,
            });
        }

        violations
    }

    fn global_menu_paths(workspace_root: &Path) -> Vec<PathBuf> {
        let app_frame_dir = workspace_root.join("crates/katana-ui/src/views/app_frame");
        let Ok(entries) = std::fs::read_dir(app_frame_dir) else {
            return vec![];
        };
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| Self::is_global_menu_module(path))
            .collect();
        paths.sort();
        paths
    }

    fn is_global_menu_module(path: &Path) -> bool {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };
        file_name.starts_with("global_menu") && file_name.ends_with(".rs")
    }
}

use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub struct ProhibitedTypesOps;

impl ProhibitedTypesOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = ProhibitedTypesVisitor {
            file_path: path.to_path_buf(),
            violations: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct ProhibitedTypesVisitor {
    file_path: PathBuf,
    violations: Vec<Violation>,
}

impl<'ast> Visit<'ast> for ProhibitedTypesVisitor {
    fn visit_type_path(&mut self, ty_path: &'ast syn::TypePath) {
        let path_str = ty_path
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");

        /* WHY: Only the fully-qualified std::sync::RwLock is prohibited; a bare `RwLock` may be parking_lot or egui imported via `use`. */
        if path_str == "std::sync::RwLock" {
            let (line, column) = LinterParserOps::span_location(ty_path.path.span());
            self.violations.push(Violation {
                file: self.file_path.clone(),
                line,
                column,
                message: "Use `egui::mutex::RwLock` or `parking_lot::RwLock` instead of `std::sync::RwLock` for better performance and deadlock avoidance.".to_string(),
            });
        }
        syn::visit::visit_type_path(self, ty_path);
    }
}

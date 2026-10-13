use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

pub struct ErrorFirstOps;

impl ErrorFirstOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = ErrorFirstVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct ErrorFirstVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl ErrorFirstVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_if_let_ok(&mut self, node: &syn::ExprIf) {
        if let syn::Expr::Let(let_expr) = &*node.cond {
            let is_ok = match &*let_expr.pat {
                syn::Pat::TupleStruct(ts) => {
                    ts.path.segments.last().is_some_and(|s| s.ident == "Ok")
                }
                syn::Pat::Path(p) => p.path.segments.last().is_some_and(|s| s.ident == "Ok"),
                _ => false,
            };

            if is_ok {
                let (line, column) = LinterParserOps::span_location(let_expr.let_token.span);
                self.violations.push(Violation {
                    file: self.file.clone(),
                    line,
                    column,
                    message: "Do not nest success paths with `if let Ok(...)`. Use `?` or `let-else` to fail early.".to_string(),
                });
            }
        }
    }
}

impl<'ast> Visit<'ast> for ErrorFirstVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.check_if_let_ok(node);
        syn::visit::visit_expr_if(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_if_let_ok() {
        let code = r#"fn foo() { if let Ok(val) = result { val } }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = ErrorFirstOps::lint(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("if let Ok"));
    }
}

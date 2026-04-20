use crate::Violation;
use crate::utils::LinterParserOps;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

const MAX_FUNCTION_LINES: usize = 30;

pub struct FunctionLengthOps;

impl FunctionLengthOps {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = FunctionLengthVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct FunctionLengthVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl FunctionLengthVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_length(&mut self, name: &syn::Ident, block: &syn::Block) {
        let (start, _) = LinterParserOps::span_location(block.brace_token.span.join());
        let (end, _) = LinterParserOps::span_location(block.brace_token.span.join());
        /* WHY: Both span calls return the same token's location; end is approximated from brace span. */
        let lines = end.saturating_sub(start);
        if lines > MAX_FUNCTION_LINES {
            let (name_line, name_column) = LinterParserOps::span_location(name.span());
            self.violations.push(Violation {
                file: self.file.clone(),
                line: name_line,
                column: name_column,
                message: format!(
                    "Function `{name}` exceeds {MAX_FUNCTION_LINES}-line limit (current: {lines}). Extract helper methods."
                ),
            });
        }
    }
}

impl<'ast> Visit<'ast> for FunctionLengthVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.check_length(&node.sig.ident, &node.block);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.check_length(&node.sig.ident, &node.block);
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        syn::visit::visit_item_mod(self, node);
    }
}

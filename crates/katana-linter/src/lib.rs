#![deny(warnings, clippy::all)]
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_or_default,
    clippy::wildcard_imports,
    clippy::match_wild_err_arm,
    clippy::let_and_return,
    clippy::manual_ok_err,
    clippy::cognitive_complexity
)]

pub mod markdown;
pub mod rules;
pub mod utils;

use serde_json::Value;
use std::path::{Path, PathBuf};

/* WHY: Domain entities for linter violation reporting and JSON AST traversal. */
#[derive(Debug, Clone)]
pub struct Violation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonNodeKind {
    Object,
    Array,
    String,
    Number,
    Bool,
    Null,
}

impl JsonNodeKind {
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Object(_) => Self::Object,
            Value::Array(_) => Self::Array,
            Value::String(_) => Self::String,
            Value::Number(_) => Self::Number,
            Value::Bool(_) => Self::Bool,
            Value::Null => Self::Null,
        }
    }
}

impl std::fmt::Display for JsonNodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object => write!(f, "object"),
            Self::Array => write!(f, "array"),
            Self::String => write!(f, "string"),
            Self::Number => write!(f, "number"),
            Self::Bool => write!(f, "bool"),
            Self::Null => write!(f, "null"),
        }
    }
}

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/* WHY: Cache parsed ASTs to avoid redundant parsing across multiple linter tests.
We use thread_local to avoid requiring Sync on the syn::File type while still
achieving significant speedups since tests run in long-lived threads. */
thread_local! {
    static AST_CACHE: RefCell<HashMap<PathBuf, Rc<syn::File>>> = RefCell::new(HashMap::new());
}

pub struct AstLinterOps;

impl AstLinterOps {
    pub fn run(
        rule_name: &str,
        hint: &str,
        target_dirs: &[PathBuf],
        lint_fn: fn(&Path, &syn::File) -> Vec<Violation>,
    ) {
        let mut all_violations: Vec<Violation> = Vec::new();

        for target_dir in target_dirs {
            for file in &utils::LinterFileOps::collect_rs_files(target_dir) {
                match Self::get_cached_ast(file) {
                    Ok(syntax) => {
                        let violations = lint_fn(file, &syntax);
                        all_violations.extend(violations);
                    }
                    Err(errors) => {
                        all_violations.extend(errors);
                    }
                }
            }
        }

        utils::ViolationReporterOps::panic(rule_name, hint, &all_violations);
    }

    fn get_cached_ast(file: &Path) -> Result<Rc<syn::File>, Vec<Violation>> {
        AST_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if let Some(cached) = cache.get(file) {
                return Ok(Rc::clone(cached));
            }

            match utils::LinterParserOps::parse_file(file) {
                Ok(syntax) => {
                    let syntax_rc = Rc::new(syntax);
                    cache.insert(file.to_path_buf(), Rc::clone(&syntax_rc));
                    Ok(syntax_rc)
                }
                Err(errors) => Err(errors),
            }
        })
    }
}

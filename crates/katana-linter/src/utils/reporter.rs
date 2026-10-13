use crate::Violation;
use std::path::Path;

pub struct ViolationReporterOps;

impl ViolationReporterOps {
    pub fn format_violations(rule_name: &str, violations: &[Violation]) -> String {
        let mut msg = format!("\n[AST Linter Error] Rule: {}\n", rule_name);
        for v in violations {
            msg.push_str(&format!(
                "  {}:{}:{} — {}\n",
                v.file.display(),
                v.line,
                v.column,
                v.message
            ));
        }
        msg
    }

    pub fn panic(rule_name: &str, hint: &str, violations: &[Violation]) {
        if violations.is_empty() {
            return;
        }

        let mut msg = Self::format_violations(rule_name, violations);
        msg.push('\n');
        msg.push_str(&format!("💡 Fix: {}\n", hint));
        msg.push_str("📖 Details: See docs/coding-rules.md\n");

        panic!("{}", msg);
    }

    pub fn locale_violation(file: &Path, message: impl Into<String>) -> Violation {
        Violation {
            file: file.to_path_buf(),
            line: 0,
            column: 0,
            message: message.into(),
        }
    }
}

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct DiagnosticRange {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Debug, Clone)]
pub struct MarkdownDiagnostic {
    pub file: PathBuf,
    pub severity: DiagnosticSeverity,
    pub range: DiagnosticRange,
    pub message: String,
    pub rule_id: String,
}

impl std::fmt::Display for MarkdownDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sev = match self.severity {
            DiagnosticSeverity::Error => "ERROR",
            DiagnosticSeverity::Warning => "WARN",
            DiagnosticSeverity::Info => "INFO",
        };
        write!(
            f,
            "[{}] {} {}:{}:{} — {}",
            sev,
            self.rule_id,
            self.file.display(),
            self.range.start_line,
            self.range.start_column,
            self.message
        )
    }
}

pub trait MarkdownRule {
    fn id(&self) -> &'static str;
    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic>;
}

/* WHY: Section: Basic Markdown Rules
======================================================= */

pub struct HeadingStructureRule;

impl MarkdownRule for HeadingStructureRule {
    fn id(&self) -> &'static str {
        "md-heading-structure"
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let mut diagnostics = Vec::new();
        let mut last_level = 0;

        for (line_idx, line) in content.lines().enumerate() {
            if let Some(level) = get_heading_level(line) {
                let current_level = level;
                if last_level > 0 && current_level > last_level + 1 {
                    diagnostics.push(MarkdownDiagnostic {
                        file: file_path.to_path_buf(),
                        severity: DiagnosticSeverity::Warning,
                        range: DiagnosticRange {
                            start_line: line_idx + 1,
                            start_column: 1,
                            end_line: line_idx + 1,
                            end_column: line.len(),
                        },
                        message: format!(
                            "Heading level skipped from h{} to h{}",
                            last_level, current_level
                        ),
                        rule_id: self.id().to_string(),
                    });
                }
                last_level = current_level;
            }
        }
        diagnostics
    }
}

fn get_heading_level(line: &str) -> Option<usize> {
    if line.starts_with('#') {
        let count = line.chars().take_while(|c| *c == '#').count();
        if line[count..].starts_with(' ') {
            return Some(count);
        }
    }
    None
}

pub struct BrokenLinkRule;

impl MarkdownRule for BrokenLinkRule {
    fn id(&self) -> &'static str {
        "md-broken-link"
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let mut diagnostics = Vec::new();
        /* WHY: Running in workspace context lets us resolve local paths relative to the file. */
        let base_dir = file_path.parent().unwrap_or(Path::new(""));

        for (line_idx, line) in content.lines().enumerate() {
            let mut rest = line;
            let mut offset = 0;
            while let Some(start_idx) = rest.find("](") {
                let actual_start = offset + start_idx;
                rest = &rest[start_idx + 2..];
                offset += start_idx + 2;

                if let Some(end_idx) = rest.find(')') {
                    let link = &rest[..end_idx];
                    let absolute_end = offset + end_idx;

                    if !link.starts_with("http") && !link.starts_with('#') {
                        let target_path = base_dir.join(link);
                        if !target_path.exists() && !target_path.with_extension("md").exists() {
                            diagnostics.push(MarkdownDiagnostic {
                                file: file_path.to_path_buf(),
                                severity: DiagnosticSeverity::Error,
                                range: DiagnosticRange {
                                    start_line: line_idx + 1,
                                    start_column: actual_start + 1,
                                    end_line: line_idx + 1,
                                    end_column: absolute_end + 2,
                                },
                                message: format!("Broken local link: {}", link),
                                rule_id: self.id().to_string(),
                            });
                        }
                    }

                    rest = &rest[end_idx + 1..];
                    offset += end_idx + 1;
                }
            }
        }
        diagnostics
    }
}

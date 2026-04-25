use crate::app::lint_fix::LintFixApplication;
use crate::state::{AutofixDiagnostic, FileAutofixCandidate, FileAutofixRequest};
use katana_linter::rules::markdown::{DiagnosticSeverity, MarkdownDiagnostic, MarkdownLinterOps};
use std::collections::HashMap;

const CONTENT_START: &str = "<<KATANA_AUTOFIX_CONTENT>>";
const CONTENT_END: &str = "<<END_KATANA_AUTOFIX_CONTENT>>";

pub(crate) struct AutofixRequestBuilder;
pub(crate) struct AutofixPromptBuilder;
pub(crate) struct AutofixResponseNormalizer;

impl AutofixRequestBuilder {
    pub(crate) fn build(
        path: &std::path::Path,
        original_content: &str,
        diagnostics: &[MarkdownDiagnostic],
        linter_enabled: bool,
        severity_map: &HashMap<String, Option<DiagnosticSeverity>>,
        model: String,
    ) -> FileAutofixRequest {
        let fixes = diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.fix_info.clone())
            .collect::<Vec<_>>();
        let deterministic_fixed_content =
            LintFixApplication::apply_to_content(original_content, &fixes);
        let residual_diagnostics = Self::residual_diagnostics(
            path,
            &deterministic_fixed_content,
            linter_enabled,
            severity_map,
        );
        FileAutofixRequest {
            path: path.to_path_buf(),
            original_content: original_content.to_string(),
            deterministic_fixed_content,
            residual_diagnostics,
            model,
        }
    }

    fn residual_diagnostics(
        path: &std::path::Path,
        content: &str,
        linter_enabled: bool,
        severity_map: &HashMap<String, Option<DiagnosticSeverity>>,
    ) -> Vec<AutofixDiagnostic> {
        MarkdownLinterOps::evaluate_all(path, content, linter_enabled, severity_map)
            .iter()
            .filter(|diagnostic| diagnostic.official_meta.is_some())
            .map(AutofixDiagnostic::from_markdown)
            .collect()
    }
}

impl AutofixPromptBuilder {
    pub(crate) fn build(request: &FileAutofixRequest) -> String {
        format!(
            "You are fixing a Markdown file.\n\
Return only the full replacement Markdown content between {CONTENT_START} and {CONTENT_END}.\n\
Do not include explanations outside those markers.\n\n\
File path:\n{}\n\n\
Original content:\n```markdown\n{}\n```\n\n\
KML deterministic fixed content:\n```markdown\n{}\n```\n\n\
Residual diagnostics after KML fixes:\n{}\n",
            request.path.display(),
            request.original_content,
            request.deterministic_fixed_content,
            Self::format_diagnostics(&request.residual_diagnostics)
        )
    }

    fn format_diagnostics(diagnostics: &[AutofixDiagnostic]) -> String {
        if diagnostics.is_empty() {
            return "- none".to_string();
        }
        diagnostics
            .iter()
            .map(Self::format_diagnostic)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_diagnostic(diagnostic: &AutofixDiagnostic) -> String {
        format!(
            "- {} {} at {}:{}: {}",
            diagnostic.severity,
            diagnostic.rule_id,
            diagnostic.line,
            diagnostic.column,
            diagnostic.message
        )
    }
}

impl AutofixResponseNormalizer {
    pub(crate) fn normalize(
        request: &FileAutofixRequest,
        response_content: &str,
    ) -> Result<FileAutofixCandidate, String> {
        let proposal_content = Self::proposal_content(response_content)?;
        Ok(FileAutofixCandidate::new(request, proposal_content))
    }

    fn proposal_content(response_content: &str) -> Result<String, String> {
        let content = Self::extract_marked(response_content)
            .unwrap_or_else(|| Self::strip_markdown_fence(response_content));
        if content.is_empty() {
            Err("empty autofix content".to_string())
        } else {
            Ok(content)
        }
    }

    fn extract_marked(response_content: &str) -> Option<String> {
        let start = response_content.find(CONTENT_START)?;
        let content_start = start + CONTENT_START.len();
        let rest = &response_content[content_start..];
        let end = rest.find(CONTENT_END)?;
        Some(Self::trim_marker_padding(&rest[..end]))
    }

    fn trim_marker_padding(content: &str) -> String {
        let content = content
            .strip_prefix("\r\n")
            .or_else(|| content.strip_prefix('\n'))
            .unwrap_or(content);
        content.to_string()
    }

    fn strip_markdown_fence(response_content: &str) -> String {
        let trimmed_start = response_content.trim_start();
        if !trimmed_start.starts_with("```") {
            return trimmed_start.to_string();
        }
        let Some(first_line_end) = trimmed_start.find('\n') else {
            return trimmed_start.to_string();
        };
        let content_start = first_line_end + 1;
        let content = &trimmed_start[content_start..];
        if let Some(fence_start) = content.rfind("```") {
            content[..fence_start].to_string()
        } else {
            trimmed_start.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_contains_original_kml_fixed_content_and_residual_diagnostics() {
        let request = FileAutofixRequest {
            path: std::path::PathBuf::from("docs/test.md"),
            original_content: "#Title".to_string(),
            deterministic_fixed_content: "# Title".to_string(),
            residual_diagnostics: vec![AutofixDiagnostic {
                rule_id: "MD013".to_string(),
                message: "Line length".to_string(),
                severity: "Warning".to_string(),
                line: 3,
                column: 1,
            }],
            model: "llama3.2".to_string(),
        };

        let prompt = AutofixPromptBuilder::build(&request);

        assert!(prompt.contains("docs/test.md"));
        assert!(prompt.contains("#Title"));
        assert!(prompt.contains("# Title"));
        assert!(prompt.contains("MD013"));
    }

    #[test]
    fn normalizer_extracts_marked_file_content() {
        let request = FileAutofixRequest {
            path: std::path::PathBuf::from("a.md"),
            original_content: "old".to_string(),
            deterministic_fixed_content: "old".to_string(),
            residual_diagnostics: Vec::new(),
            model: "llama3.2".to_string(),
        };
        let response = "note\n<<KATANA_AUTOFIX_CONTENT>>\nnew\n<<END_KATANA_AUTOFIX_CONTENT>>";

        let candidate = AutofixResponseNormalizer::normalize(&request, response).unwrap();

        assert_eq!(candidate.proposal_content, "new\n");
        assert_eq!(candidate.path, std::path::PathBuf::from("a.md"));
    }

    #[test]
    fn normalizer_preserves_trailing_newline_inside_markers() {
        let request = FileAutofixRequest {
            path: std::path::PathBuf::from("a.md"),
            original_content: "old".to_string(),
            deterministic_fixed_content: "old".to_string(),
            residual_diagnostics: Vec::new(),
            model: "llama3.2".to_string(),
        };
        let response = "<<KATANA_AUTOFIX_CONTENT>>\nnew\n<<END_KATANA_AUTOFIX_CONTENT>>";

        let candidate = AutofixResponseNormalizer::normalize(&request, response).unwrap();

        assert_eq!(candidate.proposal_content, "new\n");
    }
}

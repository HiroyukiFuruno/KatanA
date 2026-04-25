use super::diff_preview::DiffPreview;
use katana_linter::rules::markdown::MarkdownDiagnostic;
use std::path::PathBuf;

pub type AutofixResponseReceiver = std::sync::mpsc::Receiver<Result<FileAutofixCandidate, String>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutofixDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub severity: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileAutofixRequest {
    pub path: PathBuf,
    pub original_content: String,
    pub deterministic_fixed_content: String,
    pub residual_diagnostics: Vec<AutofixDiagnostic>,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileAutofixCandidate {
    pub path: PathBuf,
    pub original_content: String,
    pub proposal_content: String,
    pub model: String,
    pub diff: DiffPreview,
}

pub struct AutofixState {
    pub is_pending: bool,
    pub error: Option<String>,
    pub candidate: Option<FileAutofixCandidate>,
    pub response_rx: Option<AutofixResponseReceiver>,
}

impl Default for AutofixState {
    fn default() -> Self {
        Self::new()
    }
}

impl AutofixState {
    pub fn new() -> Self {
        Self {
            is_pending: false,
            error: None,
            candidate: None,
            response_rx: None,
        }
    }

    pub fn begin_request(&mut self, response_rx: AutofixResponseReceiver) {
        self.is_pending = true;
        self.error = None;
        self.candidate = None;
        self.response_rx = Some(response_rx);
    }

    pub fn set_candidate(&mut self, candidate: FileAutofixCandidate) {
        self.is_pending = false;
        self.error = None;
        self.candidate = Some(candidate);
        self.response_rx = None;
    }

    pub fn set_error(&mut self, message: String) {
        self.is_pending = false;
        self.error = Some(message);
        self.candidate = None;
        self.response_rx = None;
    }

    pub fn clear_candidate(&mut self) {
        self.candidate = None;
    }
}

impl AutofixDiagnostic {
    pub fn from_markdown(diagnostic: &MarkdownDiagnostic) -> Self {
        let rule_id = diagnostic
            .official_meta
            .as_ref()
            .map(|meta| meta.code.to_string())
            .unwrap_or_else(|| diagnostic.rule_id.clone());
        Self {
            rule_id,
            message: diagnostic.message.clone(),
            severity: format!("{:?}", diagnostic.severity),
            line: diagnostic.range.start_line,
            column: diagnostic.range.start_column,
        }
    }
}

impl FileAutofixCandidate {
    pub fn new(request: &FileAutofixRequest, proposal_content: String) -> Self {
        Self {
            path: request.path.clone(),
            original_content: request.original_content.clone(),
            diff: DiffPreview::from_contents(&request.original_content, &proposal_content),
            proposal_content,
            model: request.model.clone(),
        }
    }

    pub fn has_changes(&self) -> bool {
        self.diff.has_changes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_keeps_original_content_until_apply() {
        let request = FileAutofixRequest {
            path: PathBuf::from("a.md"),
            original_content: "old".to_string(),
            deterministic_fixed_content: "old".to_string(),
            residual_diagnostics: Vec::new(),
            model: "llama3.2".to_string(),
        };

        let candidate = FileAutofixCandidate::new(&request, "new".to_string());

        assert_eq!(candidate.original_content, "old");
        assert_eq!(candidate.proposal_content, "new");
        assert!(candidate.has_changes());
    }

    #[test]
    fn error_clears_pending_candidate() {
        let request = FileAutofixRequest {
            path: PathBuf::from("a.md"),
            original_content: "old".to_string(),
            deterministic_fixed_content: "old".to_string(),
            residual_diagnostics: Vec::new(),
            model: "llama3.2".to_string(),
        };
        let mut state = AutofixState::new();
        state.set_candidate(FileAutofixCandidate::new(&request, "new".to_string()));

        state.set_error("failed".to_string());

        assert!(state.candidate.is_none());
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterTranslations {
    pub rule_toggle: String,
    pub docs: String,
    pub fix: String,
    pub fix_all: String,
    pub disable_rule: String,
    pub disable_rule_desc: String,
    pub enable_linter: String,
    pub rule_severities: String,
    pub advanced_workspace_settings: String,
    pub search_placeholder: String,
    pub workspace_has_config: String,
    pub open_config: String,
    pub workspace_no_config: String,
    pub create_config: String,
    pub open_workspace_to_configure: String,
    pub severity_ignore: String,
    pub severity_warning: String,
    pub severity_error: String,
    pub use_workspace_local_config: String,
    #[serde(default)]
    pub autofix: String,
    #[serde(default)]
    pub autofix_pending: String,
    #[serde(default)]
    pub autofix_disabled: String,
    #[serde(default)]
    pub autofix_model_required: String,
    #[serde(default)]
    pub autofix_no_diagnostics: String,
    #[serde(default)]
    pub autofix_diff_title: String,
    #[serde(default)]
    pub autofix_apply: String,
    #[serde(default)]
    pub autofix_cancel: String,
    #[serde(default)]
    pub autofix_no_changes: String,
    #[serde(default)]
    pub autofix_stale: String,
    #[serde(default)]
    pub autofix_applied: String,
    #[serde(default)]
    pub autofix_failed: String,
    #[serde(default)]
    pub autofix_interrupted: String,
    #[serde(default)]
    pub view_on_github: String,
    #[serde(default)]
    pub rule_descriptions: std::collections::HashMap<String, String>,
}

impl Default for LinterTranslations {
    fn default() -> Self {
        Self {
            rule_toggle: "Toggle Linter Rule: {rule_code} ({rule_name})".to_string(),
            docs: "Docs".to_string(),
            fix: "Fix".to_string(),
            fix_all: "Fix All".to_string(),
            disable_rule: "Disable".to_string(),
            disable_rule_desc: "Disable this rule".to_string(),
            enable_linter: "Enable Markdown Linter".to_string(),
            rule_severities: "Rule Severities".to_string(),
            advanced_workspace_settings: "Advanced Workspace Settings".to_string(),
            search_placeholder: "Search rules…".to_string(),
            workspace_has_config: "Workspace has a .markdownlint.json configuration file."
                .to_string(),
            open_config: "Open Configuration".to_string(),
            workspace_no_config: "No .markdownlint.json found in current workspace.".to_string(),
            create_config: "Create Configuration".to_string(),
            open_workspace_to_configure:
                "Open a workspace to configure workspace-specific lint rules.".to_string(),
            severity_ignore: "Ignore".to_string(),
            severity_warning: "Warning".to_string(),
            severity_error: "Error".to_string(),
            use_workspace_local_config: "Use Workspace-Local Configuration".to_string(),
            autofix: "AI Fix File".to_string(),
            autofix_pending: "Generating AI autofix proposal...".to_string(),
            autofix_disabled: "AI autofix is disabled.".to_string(),
            autofix_model_required: "Select an Ollama model before running AI autofix.".to_string(),
            autofix_no_diagnostics: "No diagnostics are available for this file.".to_string(),
            autofix_diff_title: "AI Autofix Preview".to_string(),
            autofix_apply: "Apply".to_string(),
            autofix_cancel: "Cancel".to_string(),
            autofix_no_changes: "The proposal did not change the file.".to_string(),
            autofix_stale:
                "The file changed after the proposal was generated. Run AI autofix again."
                    .to_string(),
            autofix_applied: "AI autofix was applied, saved, and linted again.".to_string(),
            autofix_failed: "AI autofix failed: {error}".to_string(),
            autofix_interrupted: "AI autofix response was interrupted.".to_string(),
            view_on_github: "View on official GitHub".to_string(),
            rule_descriptions: std::collections::HashMap::new(),
        }
    }
}

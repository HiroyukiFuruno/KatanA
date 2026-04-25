use crate::app::DocumentOps;
use crate::app::autofix_request::AutofixRequestBuilder;
use crate::app_state::{SettingsSection, SettingsTab};
use crate::shell::KatanaApp;
use katana_linter::rules::markdown::DiagnosticSeverity;
use std::collections::HashMap;
use std::path::Path;

pub(crate) trait AutofixSupportOps {
    fn build_autofix_request(&mut self, path: &Path) -> Option<crate::state::FileAutofixRequest>;
    fn ensure_autofix_document_active(&mut self, path: &Path) -> bool;
    fn active_content_matches(&self, expected: &str) -> bool;
}

impl AutofixSupportOps for KatanaApp {
    fn build_autofix_request(&mut self, path: &Path) -> Option<crate::state::FileAutofixRequest> {
        let ollama = self.state.config.settings.settings().ai.ollama.clone();
        if !self.ensure_autofix_enabled(&ollama) {
            return None;
        }
        let diagnostics = self.state.diagnostics.get_file_diagnostics(path).to_vec();
        if !diagnostics
            .iter()
            .any(|diagnostic| diagnostic.official_meta.is_some())
        {
            self.state.autofix.set_error(
                crate::i18n::I18nOps::get()
                    .linter
                    .autofix_no_diagnostics
                    .clone(),
            );
            return None;
        }
        let Ok(content) = self.content_for_autofix(path) else {
            return None;
        };
        let (enabled, severity_map) = self.current_linter_options();
        Some(AutofixRequestBuilder::build(
            path,
            &content,
            &diagnostics,
            enabled,
            &severity_map,
            ollama.selected_model,
        ))
    }

    fn ensure_autofix_document_active(&mut self, path: &Path) -> bool {
        if let Some(index) = self
            .state
            .document
            .open_documents
            .iter()
            .position(|doc| doc.path == path)
        {
            self.state.document.active_doc_idx = Some(index);
            return true;
        }
        self.handle_select_document(path.to_path_buf(), true);
        self.state
            .active_document()
            .is_some_and(|doc| doc.path == path)
    }

    fn active_content_matches(&self, expected: &str) -> bool {
        self.state
            .active_document()
            .is_some_and(|doc| doc.buffer == expected)
    }
}

impl KatanaApp {
    fn ensure_autofix_enabled(&mut self, ollama: &katana_platform::OllamaSettings) -> bool {
        let messages = &crate::i18n::I18nOps::get().linter;
        if !ollama.autofix_enabled {
            self.state
                .autofix
                .set_error(messages.autofix_disabled.clone());
            return false;
        }
        if ollama.selected_model.trim().is_empty() {
            self.state
                .autofix
                .set_error(messages.autofix_model_required.clone());
            self.open_ai_settings();
            return false;
        }
        true
    }

    fn content_for_autofix(&mut self, path: &Path) -> Result<String, ()> {
        if let Some(doc) = self
            .state
            .document
            .open_documents
            .iter()
            .find(|doc| doc.path == path)
        {
            return Ok(doc.buffer.clone());
        }
        std::fs::read_to_string(path).map_err(|err| {
            self.state.autofix.set_error(crate::i18n::I18nOps::tf(
                &crate::i18n::I18nOps::get().linter.autofix_failed,
                &[("error", &err.to_string())],
            ));
        })
    }

    fn current_linter_options(&self) -> (bool, HashMap<String, Option<DiagnosticSeverity>>) {
        let linter_settings = &self.state.config.settings.settings().linter;
        let severity_map = linter_settings
            .rule_severity
            .iter()
            .map(|(rule_id, severity)| (rule_id.clone(), Self::diagnostic_severity(*severity)))
            .collect();
        (linter_settings.enabled, severity_map)
    }

    fn diagnostic_severity(
        severity: katana_platform::settings::types::RuleSeverity,
    ) -> Option<DiagnosticSeverity> {
        match severity {
            katana_platform::settings::types::RuleSeverity::Ignore => None,
            katana_platform::settings::types::RuleSeverity::Warning => {
                Some(DiagnosticSeverity::Warning)
            }
            katana_platform::settings::types::RuleSeverity::Error => {
                Some(DiagnosticSeverity::Error)
            }
        }
    }

    fn open_ai_settings(&mut self) {
        self.state.config.active_settings_tab = SettingsTab::Ai;
        self.state.config.active_settings_section = SettingsSection::Behavior;
        self.state.layout.show_settings = true;
    }
}

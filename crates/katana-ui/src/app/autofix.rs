use crate::app::DocumentOps;
use crate::app::autofix_request::{AutofixPromptBuilder, AutofixResponseNormalizer};
use crate::app::autofix_support::AutofixSupportOps;
use crate::app_state::StatusType;
use crate::shell::KatanaApp;
use eframe::egui;
use katana_core::ai::{AiProvider, AiRequest, OllamaProvider};
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;

const AUTOFIX_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

pub(crate) trait AutofixOps {
    fn request_file_autofix(&mut self, path: PathBuf);
    fn poll_autofix(&mut self, ctx: &egui::Context);
    fn apply_autofix_candidate(&mut self);
    fn cancel_autofix_candidate(&mut self);
}

impl AutofixOps for KatanaApp {
    fn request_file_autofix(&mut self, path: PathBuf) {
        if self.state.autofix.is_pending {
            return;
        }
        let Some(request) = self.build_autofix_request(&path) else {
            return;
        };
        let prompt = AutofixPromptBuilder::build(&request);
        let ollama = self.state.config.settings.settings().ai.ollama.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.state.autofix.begin_request(rx);
        std::thread::spawn(move || {
            let provider = OllamaProvider::new(
                ollama.endpoint,
                ollama.selected_model.clone(),
                ollama.timeout_secs,
            );
            let mut ai_request = AiRequest::new(prompt);
            ai_request.model = Some(ollama.selected_model);
            let result = provider
                .execute(&ai_request)
                .map_err(|err| err.to_string())
                .and_then(|response| {
                    AutofixResponseNormalizer::normalize(&request, &response.content)
                });
            let _ = tx.send(result);
        });
    }

    fn poll_autofix(&mut self, ctx: &egui::Context) {
        let result = match self.state.autofix.response_rx.as_ref() {
            Some(rx) => rx.try_recv(),
            None => return,
        };
        match result {
            Ok(Ok(candidate)) => self.handle_autofix_candidate(candidate),
            Ok(Err(message)) => self.handle_autofix_error(message),
            Err(TryRecvError::Empty) => ctx.request_repaint_after(AUTOFIX_POLL_INTERVAL),
            Err(TryRecvError::Disconnected) => {
                let message = crate::i18n::I18nOps::get()
                    .linter
                    .autofix_interrupted
                    .clone();
                self.handle_autofix_error(message);
            }
        }
    }

    fn apply_autofix_candidate(&mut self) {
        let Some(candidate) = self.state.autofix.candidate.clone() else {
            return;
        };
        if !self.ensure_autofix_document_active(&candidate.path) {
            return;
        }
        if !self.active_content_matches(&candidate.original_content) {
            self.state
                .autofix
                .set_error(crate::i18n::I18nOps::get().linter.autofix_stale.clone());
            return;
        }
        self.handle_update_buffer(candidate.proposal_content);
        self.handle_save_document();
        self.handle_action_refresh_diagnostics();
        self.state.autofix.clear_candidate();
        if self.state.active_document().is_some_and(|doc| doc.is_dirty) {
            self.copy_save_error_to_autofix();
        } else {
            self.state.layout.status_message = Some((
                crate::i18n::I18nOps::get().linter.autofix_applied.clone(),
                StatusType::Success,
            ));
        }
    }

    fn cancel_autofix_candidate(&mut self) {
        self.state.autofix.clear_candidate();
    }
}

impl KatanaApp {
    fn handle_autofix_candidate(&mut self, candidate: crate::state::FileAutofixCandidate) {
        if candidate.has_changes() {
            self.state.autofix.set_candidate(candidate);
        } else {
            let message = crate::i18n::I18nOps::get()
                .linter
                .autofix_no_changes
                .clone();
            self.state.autofix.set_error(message);
        }
    }

    fn handle_autofix_error(&mut self, message: String) {
        let error = crate::i18n::I18nOps::tf(
            &crate::i18n::I18nOps::get().linter.autofix_failed,
            &[("error", &message)],
        );
        self.state.autofix.set_error(error);
    }

    fn copy_save_error_to_autofix(&mut self) {
        let message = self
            .state
            .layout
            .status_message
            .as_ref()
            .map(|(message, _status)| message.clone())
            .unwrap_or_else(|| crate::i18n::I18nOps::get().status.save_failed.clone());
        self.state.autofix.set_error(message);
    }
}

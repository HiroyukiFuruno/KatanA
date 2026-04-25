use crate::shell::KatanaApp;
use eframe::egui;
use katana_core::ai::{AiProvider, AiRequest, OllamaProvider};
use std::sync::mpsc::TryRecvError;

const CHAT_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

pub(crate) trait ChatOps {
    fn submit_chat_message(&mut self);
    fn poll_chat(&mut self, ctx: &egui::Context);
}

impl ChatOps for KatanaApp {
    fn submit_chat_message(&mut self) {
        let prompt = self.state.chat.draft.trim().to_string();
        if prompt.is_empty() || self.state.chat.is_pending {
            return;
        }

        let ollama = self.state.config.settings.settings().ai.ollama.clone();
        let messages = &crate::i18n::I18nOps::get().chat;
        if !ollama.chat_enabled {
            self.state.chat.error = Some(messages.disabled_error.clone());
            return;
        }
        if ollama.selected_model.trim().is_empty() {
            self.state.chat.error = Some(messages.model_required.clone());
            return;
        }

        self.state.chat.selected_model = ollama.selected_model.clone();
        self.state.chat.push_user(prompt.clone());
        self.state.chat.draft.clear();
        self.state.chat.error = None;
        self.state.chat.is_pending = true;

        let (tx, rx) = std::sync::mpsc::channel();
        self.state.chat.response_rx = Some(rx);
        std::thread::spawn(move || {
            let provider = OllamaProvider::new(
                ollama.endpoint,
                ollama.selected_model.clone(),
                ollama.timeout_secs,
            );
            let mut request = AiRequest::new(prompt);
            request.model = Some(ollama.selected_model);
            let result = provider.execute(&request).map(|response| response.content);
            let _ = tx.send(result.map_err(|err| err.to_string()));
        });
    }

    fn poll_chat(&mut self, ctx: &egui::Context) {
        let result = match self.state.chat.response_rx.as_ref() {
            Some(rx) => rx.try_recv(),
            None => return,
        };

        match result {
            Ok(Ok(content)) => {
                self.state.chat.push_assistant(content);
                self.state.chat.error = None;
                self.state.chat.is_pending = false;
                self.state.chat.response_rx = None;
                ctx.request_repaint();
            }
            Ok(Err(message)) => {
                self.state.chat.error = Some(message);
                self.state.chat.is_pending = false;
                self.state.chat.response_rx = None;
                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(CHAT_POLL_INTERVAL);
            }
            Err(TryRecvError::Disconnected) => {
                self.state.chat.error =
                    Some(crate::i18n::I18nOps::get().chat.interrupted_error.clone());
                self.state.chat.is_pending = false;
                self.state.chat.response_rx = None;
                ctx.request_repaint();
            }
        }
    }
}

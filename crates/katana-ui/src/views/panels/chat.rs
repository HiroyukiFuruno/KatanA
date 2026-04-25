use crate::app_state::{AppAction, ChatRole};
use crate::shell::KatanaApp;
use eframe::egui;

const CHAT_PANEL_WIDTH: f32 = 360.0;
const CHAT_PANEL_MIN_WIDTH: f32 = 280.0;
const CHAT_OVERLAY_MARGIN: f32 = 12.0;
const CHAT_INPUT_ROWS: usize = 3;
const CHAT_OVERLAY_PADDING: i8 = 8;
const CHAT_MESSAGE_SPACING: f32 = 8.0;

pub(crate) struct ChatPanel<'a> {
    app: &'a mut KatanaApp,
}

impl<'a> ChatPanel<'a> {
    pub(crate) fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }

    pub(crate) fn show(self, ui: &mut egui::Ui) {
        if !self.app.state.layout.show_chat_panel {
            return;
        }

        if self.app.state.layout.chat_pinned {
            egui::Panel::right("local_llm_chat_panel")
                .resizable(true)
                .min_size(CHAT_PANEL_MIN_WIDTH)
                .default_size(CHAT_PANEL_WIDTH)
                .show_inside(ui, |ui| self.render_content(ui));
        } else {
            self.show_overlay(ui);
        }
    }

    fn show_overlay(self, ui: &mut egui::Ui) {
        let max_rect = ui.max_rect();
        let top = max_rect.top() + CHAT_OVERLAY_MARGIN;
        let left = max_rect.right() - CHAT_PANEL_WIDTH - CHAT_OVERLAY_MARGIN;
        egui::Area::new(egui::Id::new("local_llm_chat_overlay"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(left.max(max_rect.left()), top))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(egui::Margin::same(CHAT_OVERLAY_PADDING))
                    .show(ui, |ui| {
                        ui.set_width(CHAT_PANEL_WIDTH);
                        self.render_content(ui);
                    });
            });
    }

    fn render_content(self, ui: &mut egui::Ui) {
        Self::render_header(ui, self.app);
        ui.separator();
        Self::render_status(ui, self.app);
        Self::render_messages(ui, self.app);
        ui.separator();
        Self::render_input(ui, self.app);
    }

    fn render_header(ui: &mut egui::Ui, app: &mut KatanaApp) {
        let messages = &crate::i18n::I18nOps::get().chat;
        crate::widgets::AlignCenter::new()
            .content(|ui| {
                ui.heading(&messages.title);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(crate::Icon::Close.button(ui, crate::icon::IconSize::Small))
                        .on_hover_text(&messages.close)
                        .clicked()
                    {
                        app.pending_action = AppAction::ToggleChatPanel;
                    }
                    if ui
                        .add(crate::Icon::Pin.button(ui, crate::icon::IconSize::Small))
                        .on_hover_text(&messages.pin)
                        .clicked()
                    {
                        app.pending_action = AppAction::ToggleChatPinned;
                    }
                });
            })
            .show(ui);
    }

    fn render_status(ui: &mut egui::Ui, app: &mut KatanaApp) {
        let messages = &crate::i18n::I18nOps::get().chat;
        let ollama = &app.state.config.settings.settings().ai.ollama;
        if ollama.selected_model.trim().is_empty() {
            ui.label(egui::RichText::new(&messages.model_required).strong());
            if ui.button(&messages.open_settings).clicked() {
                app.state.config.active_settings_tab = crate::app_state::SettingsTab::Ai;
                app.state.config.active_settings_section =
                    crate::app_state::SettingsSection::Behavior;
                app.state.layout.show_settings = true;
            }
        } else {
            app.state.chat.selected_model = ollama.selected_model.clone();
            let model_label = crate::i18n::I18nOps::tf(
                &messages.model_label,
                &[("model", ollama.selected_model.as_str())],
            );
            ui.label(egui::RichText::new(model_label).weak());
        }

        if let Some(error) = &app.state.chat.error {
            ui.label(egui::RichText::new(error).color(ui.visuals().error_fg_color));
        }
    }

    fn render_messages(ui: &mut egui::Ui, app: &mut KatanaApp) {
        let messages = &crate::i18n::I18nOps::get().chat;
        egui::ScrollArea::vertical()
            .id_salt("local_llm_chat_messages")
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if app.state.chat.messages.is_empty() {
                    ui.label(egui::RichText::new(&messages.empty_session).weak());
                }
                for message in &app.state.chat.messages {
                    let label = match message.role {
                        ChatRole::User => messages.user.as_str(),
                        ChatRole::Assistant => messages.assistant.as_str(),
                    };
                    ui.label(egui::RichText::new(label).strong());
                    ui.label(&message.content);
                    ui.add_space(CHAT_MESSAGE_SPACING);
                }
                if app.state.chat.is_pending {
                    ui.label(egui::RichText::new(&messages.waiting).weak());
                }
            });
    }

    fn render_input(ui: &mut egui::Ui, app: &mut KatanaApp) {
        let messages = &crate::i18n::I18nOps::get().chat;
        ui.add(
            egui::TextEdit::multiline(&mut app.state.chat.draft)
                .desired_rows(CHAT_INPUT_ROWS)
                .hint_text(&messages.input_hint),
        );
        let enabled = app.state.chat.can_submit()
            && !app
                .state
                .config
                .settings
                .settings()
                .ai
                .ollama
                .selected_model
                .trim()
                .is_empty();
        if ui
            .add_enabled(enabled, egui::Button::new(&messages.send))
            .clicked()
        {
            app.pending_action = AppAction::SubmitChatMessage;
        }
    }
}

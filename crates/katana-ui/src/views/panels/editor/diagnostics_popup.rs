use eframe::egui;
use katana_markdown_linter::rules::markdown::MarkdownDiagnostic;

const DIAGNOSTIC_POPUP_PREFIX: &str = "editor_diagnostic_action_popup";
const POPUP_SECTION_SPACE: f32 = 4.0;

pub(crate) struct DiagnosticsPopupOps;

impl DiagnosticsPopupOps {
    pub(crate) fn show(
        ui: &mut egui::Ui,
        icon_response: &egui::Response,
        line_number: usize,
        line_diagnostics: &[&MarkdownDiagnostic],
        all_diagnostics: &[MarkdownDiagnostic],
        content: &str,
        action: &mut crate::app_state::AppAction,
    ) -> bool {
        let popup_id = ui.id().with((DIAGNOSTIC_POPUP_PREFIX, line_number));
        if icon_response.hovered() || icon_response.clicked() {
            Self::set_open(ui, popup_id, true);
        }

        let popup_response = egui::Popup::from_response(icon_response)
            .id(popup_id)
            .open(Self::is_open(ui, popup_id))
            .show(|ui| Self::show_content(ui, line_diagnostics, all_diagnostics, content, action));

        let popup_hovered = popup_response
            .as_ref()
            .is_some_and(|response| response.response.hovered());
        if icon_response.clicked_elsewhere() && !popup_hovered {
            Self::set_open(ui, popup_id, false);
        }
        if popup_response.is_some_and(|response| response.inner) {
            Self::set_open(ui, popup_id, false);
        }

        icon_response.hovered() || popup_hovered
    }

    fn show_content(
        ui: &mut egui::Ui,
        line_diagnostics: &[&MarkdownDiagnostic],
        all_diagnostics: &[MarkdownDiagnostic],
        content: &str,
        action: &mut crate::app_state::AppAction,
    ) -> bool {
        let before = action.clone();
        for (index, diagnostic) in line_diagnostics.iter().enumerate() {
            if index > 0 {
                ui.separator();
            }
            if let Some(meta) = diagnostic.official_meta.as_ref() {
                super::diagnostics_hover::DiagnosticsHoverOps::show_single_diagnostic_ui(
                    ui,
                    diagnostic,
                    meta,
                    all_diagnostics,
                    content,
                    action,
                );
            }
            ui.add_space(POPUP_SECTION_SPACE);
        }
        before != *action
    }

    fn is_open(ui: &egui::Ui, popup_id: egui::Id) -> bool {
        ui.memory(|memory| memory.data.get_temp::<bool>(popup_id).unwrap_or(false))
    }

    fn set_open(ui: &egui::Ui, popup_id: egui::Id, open: bool) {
        ui.memory_mut(|memory| memory.data.insert_temp(popup_id, open));
    }
}

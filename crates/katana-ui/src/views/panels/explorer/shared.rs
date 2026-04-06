use crate::app_state::AppAction;
use eframe::egui;

const ITEM_SPACING: f32 = 2.0;

pub(crate) struct SharedPathListRenderer;

impl SharedPathListRenderer {
    pub(crate) fn render_item(
        ui: &mut egui::Ui,
        path: &str,
        is_active: bool,
        action: &mut AppAction,
        is_persisted: bool,
    ) {
        let exists = std::path::Path::new(path).exists();
        let display_name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);

        let mut text = if is_active {
            egui::RichText::new(display_name).strong()
        } else {
            egui::RichText::new(display_name)
        };

        if !exists {
            text = text.strikethrough().color(ui.visuals().weak_text_color());
        }

        let tooltip = if !exists {
            format!("{} (Not Found)", path)
        } else {
            path.to_string()
        };

        let mut open_clicked = false;
        let mut remove_clicked = false;
        let remove_text = crate::i18n::I18nOps::get().action.remove_workspace.clone();

        crate::widgets::AlignCenter::new()
            .left(|ui| {
                ui.add_space(crate::shell::RECENT_WORKSPACES_HEADING_LEFT_PADDING);
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            })
            .left(|ui| {
                let btn = egui::Button::new(text).frame(false).truncate();
                let r = ui.add(btn).on_hover_text(&tooltip);
                if r.clicked() && !is_active {
                    open_clicked = true;
                }
                r
            })
            .right(|ui| {
                let remove_icon = crate::Icon::Remove.button(ui, crate::icon::IconSize::Small);
                let r = ui.add(remove_icon).on_hover_text(&remove_text);
                if r.clicked() {
                    remove_clicked = true;
                }
                r
            })
            .show(ui);

        if open_clicked {
            if exists {
                *action =
                    crate::app_state::AppAction::OpenWorkspace(std::path::PathBuf::from(path));
            } else {
                *action = crate::app_state::AppAction::ShowStatusMessage(
                    format!("Directory not found: {}. Please remove it.", path),
                    crate::app_state::StatusType::Error,
                );
            }
        }

        if remove_clicked {
            if is_persisted {
                *action = crate::app_state::AppAction::RemoveWorkspace(path.to_string());
            } else {
                *action = crate::app_state::AppAction::RemoveWorkspaceHistory(path.to_string());
            }
        }
    }

    pub(crate) fn render_with_scroll(
        ui: &mut egui::Ui,
        paths: &[String],
        current_root: Option<&str>,
        action: &mut AppAction,
        is_persisted: bool,
    ) {
        if paths.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(crate::shell::HISTORY_MODAL_EMPTY_BOTTOM_SPACING);
                ui.label(
                    crate::i18n::I18nOps::get()
                        .workspace
                        .no_recent_workspaces
                        .clone(),
                );
            });
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for path in paths.iter().rev() {
                    let is_active = current_root.is_some_and(|root| root == path);
                    Self::render_item(ui, path, is_active, action, is_persisted);
                    ui.add_space(ITEM_SPACING);
                }
            });
    }
}

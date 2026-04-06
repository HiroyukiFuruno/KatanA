use crate::app_state::{AppAction, ViewMode};
use eframe::egui;

const UPDATE_BTN_MARGIN_LEFT: f32 = 10.0;
const COLOR_SUCCESS_G: u8 = 200;

pub(crate) struct ViewModeBar {
    pub view_mode: ViewMode,
    pub is_changelog: bool,
    pub split_direction: katana_platform::SplitDirection,
    pub pane_order: katana_platform::PaneOrder,
    pub scroll_sync_enabled: bool,
    pub scroll_sync_override: Option<bool>,
    pub update_available: bool,
    pub update_checking: bool,
    pub show_search: bool,
}

impl ViewModeBar {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        view_mode: ViewMode,
        is_changelog: bool,
        split_direction: katana_platform::SplitDirection,
        pane_order: katana_platform::PaneOrder,
        scroll_sync_enabled: bool,
        scroll_sync_override: Option<bool>,
        update_available: bool,
        update_checking: bool,
        show_search: bool,
    ) -> Self {
        Self {
            view_mode,
            is_changelog,
            split_direction,
            pane_order,
            scroll_sync_enabled,
            scroll_sync_override,
            update_available,
            update_checking,
            show_search,
        }
    }

    #[allow(deprecated)]
    pub fn show(
        self,
        ui: &mut egui::Ui,
        search_state: &mut crate::state::search::SearchState,
    ) -> Option<AppAction> {
        let mut action: Option<AppAction> = None;
        let mut mode = self.view_mode;
        let prev = mode;
        let bar_height = ui.spacing().interact_size.y;
        let available_width = ui.available_width();
        ui.allocate_ui_with_layout(
            egui::vec2(available_width, bar_height),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                let icon_bg = Self::resolve_icon_bg(ui);
                let button_size = egui::vec2(bar_height, bar_height);
                if self.update_available && !self.update_checking {
                    self.render_update_badge(ui, &mut action);
                }
                let prev_is_split = prev == ViewMode::Split;
                let is_split = mode == ViewMode::Split;
                if !self.is_changelog {
                    super::view_mode_controls::render_refresh_button(ui, icon_bg, &mut action);
                    super::view_mode_controls::ModeButtons {
                        mode: &mut mode,
                        is_split,
                    }
                    .show(ui);
                }
                if !self.is_changelog && is_split && is_split == prev_is_split {
                    super::view_mode_split::SplitControls {
                        split_direction: self.split_direction,
                        pane_order: self.pane_order,
                        scroll_sync_enabled: self.scroll_sync_enabled,
                        scroll_sync_override: self.scroll_sync_override,
                        button_size,
                        icon_bg,
                        ui,
                    }
                    .show(&mut action);
                }
                if self.show_search {
                    self.render_search_button(ui, search_state, &mut action);
                }
            },
        );
        if mode != prev {
            action = Some(AppAction::SetViewMode(mode));
        }
        action
    }

    fn resolve_icon_bg(ui: &egui::Ui) -> egui::Color32 {
        if ui.visuals().dark_mode {
            crate::theme_bridge::TRANSPARENT
        } else {
            crate::theme_bridge::ThemeBridgeOps::from_gray(crate::shell_ui::LIGHT_MODE_ICON_BG)
        }
    }

    fn render_update_badge(&self, ui: &mut egui::Ui, action: &mut Option<AppAction>) {
        let badge_color = crate::theme_bridge::ThemeBridgeOps::from_rgb(0, COLOR_SUCCESS_G, 100);
        let badge_str = crate::i18n::I18nOps::get().update.update_available.clone();
        let badge_text = egui::RichText::new(badge_str).color(badge_color).strong();
        let btn = egui::Button::image_and_text(
            crate::icon::Icon::Action
                .image(crate::icon::IconSize::Small)
                .tint(badge_color),
            badge_text,
        )
        .sense(egui::Sense::click());
        if ui
            .add(btn)
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            *action = Some(AppAction::CheckForUpdates);
        }
        ui.add_space(UPDATE_BTN_MARGIN_LEFT);
    }

    fn render_search_button(
        &self,
        ui: &mut egui::Ui,
        search_state: &mut crate::state::search::SearchState,
        action: &mut Option<AppAction>,
    ) {
        ui.separator();
        let doc_search_tooltip = format!(
            "{} (Cmd+F)",
            crate::i18n::I18nOps::get().search.doc_search_title
        );
        let btn_color = if search_state.doc_search_open {
            ui.visuals().widgets.active.bg_fill
        } else {
            egui::Color32::default()
        };
        let toggle_resp = ui
            .add(
                egui::Button::image(
                    crate::Icon::Search.ui_image(ui, crate::icon::IconSize::Medium),
                )
                .fill(btn_color),
            )
            .on_hover_text(doc_search_tooltip);
        if toggle_resp.clicked() {
            if search_state.doc_search_open {
                search_state.doc_search_open = false;
            } else {
                *action = Some(AppAction::OpenDocSearch);
            }
        }
    }
}

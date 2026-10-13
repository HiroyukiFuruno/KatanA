use super::side_panels::{LIGHT_MODE_ICON_BG, PANEL_HEAD_SPACE, PreviewSidePanels};
use crate::app_state::{AppAction, ViewMode};
use crate::icon::IconSize;
use eframe::egui;
use katana_platform::{PaneOrder, SplitDirection};

use super::tangochou::TangochouWidget;

const SPLIT_SECTION_SPACE: f32 = 8.0;
const ICON_BTN_SIZE: f32 = 32.0;
const TOOLS_RIGHT_MARGIN: f32 = 8.0;

impl<'a> PreviewSidePanels<'a> {
    pub(super) fn render_tools_inner(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(PANEL_HEAD_SPACE);

            ui.indent("tools_content", |ui| {
                /* WHY: Reserve right margin so LabeledToggle doesn't clip. */
                ui.set_max_width(ui.available_width() - TOOLS_RIGHT_MARGIN);
                let i18n = crate::i18n::I18nOps::get();
                let view_mode = self.app.state.active_view_mode();
                let is_split = view_mode == ViewMode::Split;

                /* Split Toggle */
                let mut split_on = is_split;
                let split_resp = ui.add(
                    crate::widgets::LabeledToggle::new(i18n.view_mode.split.clone(), &mut split_on)
                        .position(crate::widgets::TogglePosition::Right)
                        .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                );
                if split_resp.changed() {
                    if split_on {
                        self.app.pending_action = AppAction::SetViewMode(ViewMode::Split);
                    } else {
                        self.app.pending_action = AppAction::SetViewMode(ViewMode::PreviewOnly);
                    }
                }

                ui.add_space(SPLIT_SECTION_SPACE);

                if is_split {
                    /* When Split is ON: Scroll Sync Setting */
                    let config_sync = self
                        .app
                        .state
                        .config
                        .settings
                        .settings()
                        .behavior
                        .scroll_sync_enabled;
                    let mut sync = self.app.state.scroll.sync_override.unwrap_or(config_sync);
                    let sync_resp = ui.add(
                        crate::widgets::LabeledToggle::new(
                            i18n.settings.behavior.scroll_sync.clone(),
                            &mut sync,
                        )
                        .position(crate::widgets::TogglePosition::Right)
                        .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                    );
                    if sync_resp.changed() {
                        self.app.pending_action = AppAction::ToggleScrollSync(sync);
                    }

                    ui.add_space(SPLIT_SECTION_SPACE);

                    crate::widgets::AlignCenter::new()
                        .content(|ui| {
                            let dir = self.app.state.active_split_direction();
                            let order = self.app.state.active_pane_order();

                            let icon_bg = if ui.visuals().dark_mode {
                                crate::theme_bridge::TRANSPARENT
                            } else {
                                crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_BG)
                            };

                            /* Split Direction Icon (Horizontal/Vertical) */
                            let (dir_icon, dir_tip, next_dir) = if dir == SplitDirection::Horizontal
                            {
                                (
                                    crate::Icon::SplitHorizontal,
                                    i18n.split_toggle.vertical.clone(),
                                    SplitDirection::Vertical,
                                )
                            } else {
                                (
                                    crate::Icon::SplitVertical,
                                    i18n.split_toggle.horizontal.clone(),
                                    SplitDirection::Horizontal,
                                )
                            };
                            let dir_img = dir_icon.ui_image(ui, IconSize::Medium);
                            let btn_dir = egui::Button::image(dir_img)
                                .min_size(egui::vec2(ICON_BTN_SIZE, ICON_BTN_SIZE))
                                .fill(icon_bg);
                            let resp_dir = ui.add(btn_dir).on_hover_text(&dir_tip);
                            resp_dir.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::WidgetType::Button,
                                    ui.is_enabled(),
                                    &dir_tip,
                                )
                            });
                            if resp_dir.clicked() {
                                self.app.pending_action = AppAction::SetSplitDirection(next_dir);
                            }

                            /* Swap Pane Order Icon (EditorFirst/PreviewFirst) */
                            let swap_icon = if dir == SplitDirection::Horizontal {
                                crate::Icon::SwapHorizontal
                            } else {
                                crate::Icon::SwapVertical
                            };
                            let swap_tip = if order == PaneOrder::EditorFirst {
                                i18n.split_toggle.preview_first.clone()
                            } else {
                                i18n.split_toggle.editor_first.clone()
                            };
                            let swap_img = swap_icon.ui_image(ui, IconSize::Medium);
                            let btn_swap = egui::Button::image(swap_img)
                                .min_size(egui::vec2(ICON_BTN_SIZE, ICON_BTN_SIZE))
                                .fill(icon_bg);
                            if ui.add(btn_swap).on_hover_text(swap_tip).clicked() {
                                let new_order = match order {
                                    PaneOrder::EditorFirst => PaneOrder::PreviewFirst,
                                    PaneOrder::PreviewFirst => PaneOrder::EditorFirst,
                                };
                                self.app.pending_action = AppAction::SetPaneOrder(new_order);
                            }
                        })
                        .show(ui);
                } else {
                    /* When Split is OFF: Tangochou (Code/Preview Switch) */
                    let i18n2 = crate::i18n::I18nOps::get();
                    if let Some(action) = (TangochouWidget {
                        view_mode: self.app.state.active_view_mode(),
                        label_front: &i18n2.view_mode.code,
                        label_back: &i18n2.view_mode.preview,
                    })
                    .show(ui)
                    {
                        self.app.pending_action = action;
                    }
                }
            });
        });
    }
}

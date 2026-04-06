use crate::app_state::AppAction;
use eframe::egui;

const TOGGLE_SPACING: f32 = 8.0;

pub(super) struct SplitControls<'a> {
    pub split_direction: katana_platform::SplitDirection,
    pub pane_order: katana_platform::PaneOrder,
    pub scroll_sync_enabled: bool,
    pub scroll_sync_override: Option<bool>,
    pub button_size: egui::Vec2,
    pub icon_bg: egui::Color32,
    pub ui: &'a mut egui::Ui,
}

impl<'a> SplitControls<'a> {
    pub fn show(&mut self, action: &mut Option<AppAction>) {
        self.ui.separator();
        self.render_direction_button(action);
        self.render_order_button(action);
        self.ui.separator();
        self.render_scroll_sync(action);
    }

    fn render_direction_button(&mut self, action: &mut Option<AppAction>) {
        let current_dir = self.split_direction;
        let (dir_icon, dir_tip) = match current_dir {
            katana_platform::SplitDirection::Horizontal => (
                crate::icon::Icon::SplitHorizontal,
                crate::i18n::I18nOps::get().split_toggle.vertical.clone(),
            ),
            katana_platform::SplitDirection::Vertical => (
                crate::icon::Icon::SplitVertical,
                crate::i18n::I18nOps::get().split_toggle.horizontal.clone(),
            ),
        };
        let resp = self
            .ui
            .add(
                egui::Button::image(
                    dir_icon
                        .image(crate::icon::IconSize::Medium)
                        .tint(self.ui.visuals().text_color()),
                )
                .min_size(self.button_size)
                .fill(self.icon_bg),
            )
            .on_hover_text(dir_tip);
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Toggle Split Direction")
        });
        if resp.clicked() {
            let new_dir = match current_dir {
                katana_platform::SplitDirection::Horizontal => {
                    katana_platform::SplitDirection::Vertical
                }
                katana_platform::SplitDirection::Vertical => {
                    katana_platform::SplitDirection::Horizontal
                }
            };
            *action = Some(AppAction::SetSplitDirection(new_dir));
        }
    }

    fn render_order_button(&mut self, action: &mut Option<AppAction>) {
        let current_order = self.pane_order;
        let order_tip = match current_order {
            katana_platform::PaneOrder::EditorFirst => crate::i18n::I18nOps::get()
                .split_toggle
                .preview_first
                .clone(),
            katana_platform::PaneOrder::PreviewFirst => crate::i18n::I18nOps::get()
                .split_toggle
                .editor_first
                .clone(),
        };
        let order_icon = if self.split_direction == katana_platform::SplitDirection::Horizontal {
            crate::icon::Icon::SwapHorizontal
        } else {
            crate::icon::Icon::SwapVertical
        };
        if self
            .ui
            .add(order_icon.button(self.ui, crate::icon::IconSize::Medium))
            .on_hover_text(order_tip)
            .clicked()
        {
            let new_order = match current_order {
                katana_platform::PaneOrder::EditorFirst => katana_platform::PaneOrder::PreviewFirst,
                katana_platform::PaneOrder::PreviewFirst => katana_platform::PaneOrder::EditorFirst,
            };
            *action = Some(AppAction::SetPaneOrder(new_order));
        }
    }

    fn render_scroll_sync(&mut self, action: &mut Option<AppAction>) {
        let mut is_on = self
            .scroll_sync_override
            .unwrap_or(self.scroll_sync_enabled);
        let toggle_resp = self.ui.add(
            crate::widgets::LabeledToggle::new(
                crate::i18n::I18nOps::get()
                    .settings
                    .behavior
                    .scroll_sync
                    .clone(),
                &mut is_on,
            )
            .position(crate::widgets::TogglePosition::Right)
            .alignment(crate::widgets::ToggleAlignment::Attached(TOGGLE_SPACING)),
        );
        if toggle_resp.clicked() {
            *action = Some(AppAction::ToggleScrollSync(is_on));
        }
    }
}

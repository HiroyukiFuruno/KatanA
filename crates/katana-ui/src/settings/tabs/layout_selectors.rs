use super::types::LayoutTabOps;
use eframe::egui;
use katana_platform::{PaneOrder, SplitDirection};

impl LayoutTabOps {
    pub(crate) fn render_split_direction_selector(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) {
        ui.label(
            crate::i18n::I18nOps::get()
                .settings
                .layout
                .split_direction
                .clone(),
        );
        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui| {
                let current = state.config.settings.settings().layout.split_direction;
                if ui
                    /* WHY: in popup/list context; future: standardize as atom */
                    .add(
                        egui::Button::selectable(
                            current == SplitDirection::Horizontal,
                            crate::i18n::I18nOps::get()
                                .settings
                                .layout
                                .horizontal
                                .clone(),
                        )
                        .frame_when_inactive(true),
                    )
                    .clicked()
                    && current != SplitDirection::Horizontal
                {
                    state.config.settings.settings_mut().layout.split_direction =
                        SplitDirection::Horizontal;
                    let _ = state.config.try_save_settings();
                }
                if ui
                    /* WHY: in popup/list context; future: standardize as atom */
                    .add(
                        egui::Button::selectable(
                            current == SplitDirection::Vertical,
                            crate::i18n::I18nOps::get().settings.layout.vertical.clone(),
                        )
                        .frame_when_inactive(true),
                    )
                    .clicked()
                    && current != SplitDirection::Vertical
                {
                    state.config.settings.settings_mut().layout.split_direction =
                        SplitDirection::Vertical;
                    let _ = state.config.try_save_settings();
                }
            })
            .show(ui);
    }

    pub(crate) fn render_pane_order_selector(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) {
        ui.label(
            crate::i18n::I18nOps::get()
                .settings
                .layout
                .pane_order
                .clone(),
        );
        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui| {
                let current = state.config.settings.settings().layout.pane_order;
                if ui
                    /* WHY: in popup/list context; future: standardize as atom */
                    .add(
                        egui::Button::selectable(
                            current == PaneOrder::EditorFirst,
                            crate::i18n::I18nOps::get()
                                .settings
                                .layout
                                .editor_first
                                .clone(),
                        )
                        .frame_when_inactive(true),
                    )
                    .clicked()
                    && current != PaneOrder::EditorFirst
                {
                    state.config.settings.settings_mut().layout.pane_order = PaneOrder::EditorFirst;
                    let _ = state.config.try_save_settings();
                }
                if ui
                    /* WHY: in popup/list context; future: standardize as atom */
                    .add(
                        egui::Button::selectable(
                            current == PaneOrder::PreviewFirst,
                            crate::i18n::I18nOps::get()
                                .settings
                                .layout
                                .preview_first
                                .clone(),
                        )
                        .frame_when_inactive(true),
                    )
                    .clicked()
                    && current != PaneOrder::PreviewFirst
                {
                    state.config.settings.settings_mut().layout.pane_order =
                        PaneOrder::PreviewFirst;
                    let _ = state.config.try_save_settings();
                }
            })
            .show(ui);
    }
}

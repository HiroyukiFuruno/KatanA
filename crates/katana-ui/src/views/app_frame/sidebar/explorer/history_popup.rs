use crate::shell::KatanaApp;

/* WHY: The history popup is a thin wrapper.
 * - Title heading
 * - SharedPathListRenderer handles item layout (truncated path + remove button)
 * - "Open workspace…" button at bottom
 * No custom layout code: reuses the same renderer as EmptyWorkspaceView. */
pub(crate) struct HistoryPopup;

impl HistoryPopup {
    pub(crate) fn render(ui: &mut egui::Ui, app: &mut KatanaApp) {
        let i18n = crate::i18n::I18nOps::get();

        crate::widgets::AlignCenter::new()
            .left(|ui| ui.heading(i18n.workspace.workspace_history_title.clone()))
            .right(|ui| {
                let r = ui.add(crate::Icon::Close.button(ui, crate::icon::IconSize::Small));
                if r.clicked() {
                    app.state.layout.active_rail_popup = None;
                }
                r
            })
            .show(ui);

        ui.separator();

        /* WHY: Clone to avoid simultaneous borrow. This list has no "active" item. */
        let histories = app.state.global_workspace.state().histories.clone();
        crate::views::panels::explorer::shared::SharedPathListRenderer::render_with_scroll(
            ui,
            &histories,
            None,
            &mut app.pending_action,
            false,
        );

        ui.separator();
        if ui
            .add_sized(
                [ui.available_width(), 0.0],
                egui::Button::new(i18n.workspace.open_workspace_button.clone()),
            )
            .clicked()
        {
            app.pending_action = crate::app_state::AppAction::PickOpenWorkspace;
            app.state.layout.active_rail_popup = None;
        }
    }
}

use super::types::*;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use crate::shell_logic::ShellLogicOps;
use eframe::egui;

/* WHY: Fixed height for Row 2 (breadcrumbs + meta) and Row 3 (controls). */
const TOOLBAR_ROW_HEIGHT: f32 = 28.0;

impl<'a> TabToolbar<'a> {
    pub(crate) fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }

    pub(crate) fn show(self, ui: &mut egui::Ui) {
        let app = self.app;
        egui::Panel::top("tab_toolbar").show_inside(ui, |ui| {
            let ws_root = app
                .state
                .workspace
                .data
                .as_ref()
                .map(|ws| ws.root.as_path());

            /* Row 1: Tab bar */
            let tab_action = crate::views::top_bar::TabBar::new(
                ws_root,
                &app.state.document.open_documents,
                app.state.document.active_doc_idx,
                &app.state.document.recently_closed_tabs,
                &app.state.document.tab_groups,
                &app.state.layout.inline_rename_group,
            )
            .show(ui);
            if let Some(a) = tab_action {
                app.pending_action = a;
            }

            let doc_info = app.state.active_document().map(|doc| {
                let p = doc.path.to_string_lossy();
                /* WHY: ChangeLog/Welcome/Guide/DiffReview are read-only virtual docs — no editor controls.
                 * Katana://Demo/ docs are interactive and keep full controls. */
                let is_virtual = p.starts_with("Katana://ChangeLog")
                    || p.starts_with("Katana://Welcome")
                    || p.starts_with("Katana://Guide")
                    || crate::app::LintFixReviewPath::is_review_path(&doc.path);
                /* WHY: LinterDocs get a special "View on GitHub" button in the toolbar. */
                let linter_rule_id =
                    crate::linter_docs::LinterDocIdentity::from_virtual_path(&doc.path)
                        .map(|identity| identity.rule_id().to_string());
                (doc.path.clone(), is_virtual, linter_rule_id)
            });
            if let Some((doc_path, is_virtual, linter_rule_id)) = doc_info {
                Self::render_document_toolbar(ui, app, doc_path, is_virtual, linter_rule_id);
            }
        });
    }

    fn render_document_toolbar(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        doc_path: std::path::PathBuf,
        is_virtual: bool,
        _linter_rule_id: Option<String>,
    ) {
        let mut out_action = None;
        let available_width = ui.available_width();

        /* Row 2: Breadcrumbs (left) + Meta info button (right).
         * Uses fixed height so Panel::top does not consume the full screen height. */
        ui.allocate_ui_with_layout(
            egui::vec2(available_width, TOOLBAR_ROW_HEIGHT),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                /* WHY: FB25 — GitHub button moved to preview pane (content.rs). */

                /* WHY: Inner left_to_right fills remaining width with breadcrumbs. */
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    if is_virtual {
                        return;
                    }
                    if let Some(a) = Self::render_breadcrumbs(ui, app, &doc_path) {
                        out_action = Some(a);
                    }
                });
            },
        );

        /* Row 3 (View mode controls) was removed as requested. */

        /* Row 4 (popup): Document search bar — shown below Row 3 when active. */
        if app.state.search.doc_search_open
            && let Some(a) =
                crate::views::top_bar::search::DocSearchBar::show(ui, &mut app.state.search)
        {
            out_action = Some(a);
        }

        if let Some(a) = out_action {
            app.pending_action = a;
        }
    }

    fn render_breadcrumbs(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        doc_path: &std::path::Path,
    ) -> Option<AppAction> {
        let ws_root = app.state.workspace.data.as_ref().map(|ws| ws.root.clone());
        let rel = ShellLogicOps::relative_full_path(doc_path, ws_root.as_deref());
        Breadcrumbs::new(app, &rel, ws_root.as_deref()).show(ui)
    }
}

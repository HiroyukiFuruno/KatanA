use crate::shell::KatanaApp;
use eframe::egui;

/* WHY: Margin for hit-testing pointer containment to avoid flicker at panel edges */
pub(super) const EXPLORER_HOVER_MARGIN: f32 = 8.0;
/* WHY: Gap between rail right edge and hover panel left edge */
const HOVER_PANEL_GAP: f32 = 2.0;
/* WHY: Shadow alpha scaled by animation progress for fade-in effect */
const HOVER_SHADOW_ALPHA: f32 = 48.0;
/* WHY: Right-side rounding only — panel sits flush against the rail on the left */
const HOVER_PANEL_ROUNDING: u8 = 8;

pub(super) struct ExplorerHoverOverlay;

impl ExplorerHoverOverlay {
    pub(super) fn show(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        anim: f32,
        rail_rect: egui::Rect,
        explorer_btn_rect: Option<egui::Rect>,
    ) {
        let panel_x = rail_rect.right() + HOVER_PANEL_GAP;
        let panel_y = rail_rect.top();
        let panel_height = rail_rect.height();
        let panel_width = crate::shell::FILE_TREE_PANEL_DEFAULT_WIDTH * anim;

        let area_resp = egui::Area::new(egui::Id::new("explorer_hover_overlay"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(panel_x, panel_y))
            .show(ui.ctx(), |ui| {
                let mut window_fill = ui.visuals().window_fill();
                window_fill = window_fill.gamma_multiply(anim);
                let shadow_color = crate::theme_bridge::ThemeBridgeOps::from_black_alpha(
                    (anim * HOVER_SHADOW_ALPHA) as u8,
                );
                let frame = egui::Frame::window(ui.style())
                    .fill(window_fill)
                    .shadow(egui::Shadow {
                        color: shadow_color,
                        ..Default::default()
                    })
                    .inner_margin(egui::Margin::ZERO)
                    .rounding(egui::CornerRadius {
                        ne: HOVER_PANEL_ROUNDING,
                        se: HOVER_PANEL_ROUNDING,
                        nw: 0,
                        sw: 0,
                    });
                frame.show(ui, |ui| {
                    ui.set_width(panel_width);
                    ui.set_min_height(panel_height);
                    let active_path = app
                        .state
                        .document
                        .active_doc_idx
                        .and_then(|idx| app.state.document.open_documents.get(idx))
                        .filter(|doc| !doc.is_reference)
                        .map(|doc| doc.path.to_path_buf());
                    let show_vertical_line = app
                        .state
                        .config
                        .settings
                        .settings()
                        .layout
                        .accordion_vertical_line;
                    crate::views::panels::explorer::ExplorerPanel::new(
                        &mut app.state.workspace,
                        &mut app.state.search,
                        &app.state.global_workspace.state().histories,
                        active_path.as_deref(),
                        &app.state.document.tab_groups,
                        &mut app.pending_action,
                        show_vertical_line,
                    )
                    .show(ui);
                });
            });

        let mut keep_open = false;
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            let over_overlay = area_resp
                .response
                .rect
                .expand(EXPLORER_HOVER_MARGIN)
                .contains(pos);
            let over_btn =
                explorer_btn_rect.is_some_and(|r| r.expand(EXPLORER_HOVER_MARGIN).contains(pos));
            if over_overlay || over_btn {
                keep_open = true;
            }
        }
        if !keep_open && ui.input(|i| i.pointer.hover_pos().is_some()) {
            if ui.ctx().memory(|mem| mem.any_popup_open()) {
                return;
            }
            ui.ctx()
                .data_mut(|d| d.insert_temp(egui::Id::new("explorer_hover_open"), false));
        }
    }
}

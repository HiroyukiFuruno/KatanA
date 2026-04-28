use super::row::DiffViewerRowOps;
use super::style::DiffViewerPalette;
use eframe::egui;

const DEFAULT_SPLIT_RATIO: f32 = 0.5_f32;
const SPLIT_MIN_RATIO: f32 = 0.1_f32;
const SPLIT_MAX_RATIO: f32 = 0.9_f32;
const SPLITTER_WIDTH: f32 = 6.0_f32;
const H_STRICT_RATIO: f32 = 5.0_f32;

pub(super) struct DiffViewerSplitOps;

impl DiffViewerSplitOps {
    pub(super) fn show(ui: &mut egui::Ui, file: &crate::diff_review::DiffReviewFile) {
        let palette = DiffViewerPalette::from_ui(ui);

        let ctx = ui.ctx().clone();

        /* WHY: Split ratio persistence (default 50/50) */
        let ratio_key = egui::Id::new(("diff_viewer_split_ratio", file.path.as_path()));
        let mut ratio = ctx
            .data(|d| d.get_temp::<f32>(ratio_key))
            .unwrap_or(DEFAULT_SPLIT_RATIO);
        ratio = ratio.clamp(SPLIT_MIN_RATIO, SPLIT_MAX_RATIO);

        /* WHY: Scroll-offset keys for one-frame delayed sync */
        let left_key = egui::Id::new(("diff_viewer_h_left_offset", file.path.as_path()));
        let right_key = egui::Id::new(("diff_viewer_h_right_offset", file.path.as_path()));
        let prev_left = ctx.data(|d| d.get_temp::<f32>(left_key)).unwrap_or(0.0_f32);
        let prev_right = ctx
            .data(|d| d.get_temp::<f32>(right_key))
            .unwrap_or(0.0_f32);

        let available_width = ui.available_width();
        let splitter_w = SPLITTER_WIDTH;
        let left_w = (available_width - splitter_w) * ratio;
        let right_w = (available_width - splitter_w) - left_w;
        let height = ui.available_height();

        /* WHY: Horizontal scroll direction lock.
        Trackpad diagonal input sends both X and Y components simultaneously.
        Without filtering, scrolling vertically also drifts the view horizontally.
        We suppress the horizontal component unless the gesture is clearly horizontal
        (|Δx| must exceed |Δy| by H_STRICT_RATIO). Scoped to when the pointer is
        inside this view so we don't affect unrelated scroll areas in the same frame. */
        if ui.rect_contains_pointer(ui.max_rect()) {
            let delta = ctx.input(|i| i.smooth_scroll_delta);
            if delta.x.abs() <= delta.y.abs() * H_STRICT_RATIO {
                ctx.input_mut(|i| i.smooth_scroll_delta.x = 0.0);
            }
        }

        let mut left_x = prev_left;
        let mut right_x = prev_right;

        /* WHY: Render two fixed-width columns with a draggable splitter in-between.
        left_to_right(Align::TOP) is used instead of ui.horizontal() (which uses Align::Center)
        because Align::Center would vertically center children within the strip, causing the right
        panel to start at a different Y than the left panel when strip height != child height. */
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui_h| {
            /* WHY: Left column (fixed width) */
            ui_h.allocate_ui_with_layout(
                egui::vec2(left_w, height),
                egui::Layout::top_down(egui::Align::Min),
                |ui_left| {
                    let code_width = DiffViewerRowOps::split_code_width(ui_left);
                    let left_area_id = ("diff_viewer_h_left", file.path.as_path());
                    let mut left_area = egui::ScrollArea::horizontal()
                        .id_salt(left_area_id)
                        .auto_shrink([false, false]);
                    if prev_left > 0.0 {
                        left_area = left_area.horizontal_scroll_offset(prev_left);
                    }

                    let left_out = left_area.show(ui_left, |ui_left_inner| {
                        for row in &file.model.split_rows {
                            match row {
                                crate::diff_review::SplitDiffRow::Line(line) => {
                                    crate::widgets::AlignCenter::new()
                                        .content(|ui_row| {
                                            DiffViewerRowOps::show_split_cell(
                                                ui_row,
                                                line.before.as_ref(),
                                                code_width,
                                                &palette,
                                            );
                                        })
                                        .show(ui_left_inner);
                                }
                                crate::diff_review::SplitDiffRow::Collapsed(block) => {
                                    DiffViewerRowOps::show_collapsed_side(
                                        ui_left_inner,
                                        block.line_count,
                                        code_width,
                                        &palette,
                                    );
                                }
                            }
                        }
                    });

                    left_x = left_out.state.offset.x;
                },
            );

                /* WHY: Splitter handle — allocate space first, then interact with a STABLE explicit ID.
                    allocate_exact_size() uses position-based auto-IDs; when ratio changes,
                    left_w changes, the splitter moves, the auto-ID changes, and drag tracking
                    breaks every frame. A stable file-path-based ID survives position changes. */
            let (splitter_rect, _) = ui_h.allocate_exact_size(
                egui::vec2(splitter_w, height),
                egui::Sense::hover(),
            );
            let splitter_id = egui::Id::new(("diff_splitter_handle", file.path.as_path()));
            let handle_resp = ui_h.interact(splitter_rect, splitter_id, egui::Sense::drag());
            if handle_resp.hovered() || handle_resp.dragged() {
                ui_h.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            let splitter_color = if handle_resp.hovered() || handle_resp.dragged() {
                ui_h.visuals().widgets.active.bg_stroke.color
            } else {
                ui_h.visuals().widgets.noninteractive.bg_stroke.color
            };
            ui_h.painter().rect_filled(splitter_rect, 0.0, splitter_color);
            if handle_resp.dragged() {
                /* WHY: drag_delta() is per-frame incremental, so accumulate into ratio directly. */
                let delta = handle_resp.drag_delta().x;
                ratio = (ratio + delta / available_width).clamp(SPLIT_MIN_RATIO, SPLIT_MAX_RATIO);
                ctx.data_mut(|d| d.insert_temp(ratio_key, ratio));
            }

            /* WHY: Right column (fixed width) */
            ui_h.allocate_ui_with_layout(
                egui::vec2(right_w, height),
                egui::Layout::top_down(egui::Align::Min),
                |ui_right| {
                    let code_width = DiffViewerRowOps::split_code_width(ui_right);
                    let right_area_id = ("diff_viewer_h_right", file.path.as_path());
                    let mut right_area = egui::ScrollArea::horizontal()
                        .id_salt(right_area_id)
                        .auto_shrink([false, false]);
                    if prev_right > 0.0 {
                        right_area = right_area.horizontal_scroll_offset(prev_right);
                    }

                    let right_out = right_area.show(ui_right, |ui_right_inner| {
                        for row in &file.model.split_rows {
                            match row {
                                crate::diff_review::SplitDiffRow::Line(line) => {
                                    crate::widgets::AlignCenter::new()
                                        .content(|ui_row| {
                                            DiffViewerRowOps::show_split_cell(
                                                ui_row,
                                                line.after.as_ref(),
                                                code_width,
                                                &palette,
                                            );
                                        })
                                        .show(ui_right_inner);
                                }
                                crate::diff_review::SplitDiffRow::Collapsed(block) => {
                                    DiffViewerRowOps::show_collapsed_side(
                                        ui_right_inner,
                                        block.line_count,
                                        code_width,
                                        &palette,
                                    );
                                }
                            }
                        }
                    });

                    right_x = right_out.state.offset.x;
                },
            );

            /* WHY: Sync decision: one-frame delayed write into ctx so the other column picks it up next frame. */
            let left_delta = (left_x - prev_left).abs();
            let right_delta = (right_x - prev_right).abs();
            const H_EPS: f32 = 1.0;
            if left_delta > H_EPS && left_delta > right_delta {
                ctx.data_mut(|d| d.insert_temp(right_key, left_x));
                ctx.data_mut(|d| d.insert_temp(left_key, left_x));
            } else if right_delta > H_EPS && right_delta > left_delta {
                ctx.data_mut(|d| d.insert_temp(left_key, right_x));
                ctx.data_mut(|d| d.insert_temp(right_key, right_x));
            } else {
                ctx.data_mut(|d| d.insert_temp(left_key, left_x));
                ctx.data_mut(|d| d.insert_temp(right_key, right_x));
            }
        });
    }
}

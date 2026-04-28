use super::split_column::{DiffViewerSplitColumnOps, SplitColumnParams};
use super::split_handle::DiffViewerSplitHandleOps;
use super::split_scroll::{suppress_diagonal_horizontal_scroll, sync_offsets, SplitOffsets, SplitScrollKeys};
use super::style::DiffViewerPalette;
use eframe::egui;

const DEFAULT_SPLIT_RATIO: f32 = 0.5_f32;
const SPLIT_MIN_RATIO: f32 = 0.1_f32;
const SPLIT_MAX_RATIO: f32 = 0.9_f32;

pub(super) struct DiffViewerSplitOps;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SplitSide {
    Before,
    After,
}

impl DiffViewerSplitOps {
    pub(super) fn show(ui: &mut egui::Ui, file: &crate::diff_review::DiffReviewFile) {
        let palette = DiffViewerPalette::from_ui(ui);
        let ctx = ui.ctx().clone();
        let mut pending_toggles = Vec::new();
        let keys = SplitScrollKeys::new(file);
        let offsets = SplitOffsets::load(&ctx, &keys);
        let mut ratio = split_ratio(&ctx, file);
        let layout = SplitLayout::new(ui.available_width(), ui.available_height(), ratio);

        suppress_diagonal_horizontal_scroll(ui, &ctx);
        let mut left_x = offsets.left;
        let mut right_x = offsets.right;

        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui_h| {
            left_x = Self::show_column(
                ui_h,
                ShowColumnArgs {
                    ctx: &ctx,
                    file,
                    palette: &palette,
                    width: layout.left_width,
                    previous_offset: offsets.left,
                    side: SplitSide::Before,
                },
                &mut pending_toggles,
            );
            ratio = DiffViewerSplitHandleOps::show(ui_h, file, layout, ratio);
            right_x = Self::show_column(
                ui_h,
                ShowColumnArgs {
                    ctx: &ctx,
                    file,
                    palette: &palette,
                    width: layout.right_width,
                    previous_offset: offsets.right,
                    side: SplitSide::After,
                },
                &mut pending_toggles,
            );
            sync_offsets(&ctx, keys, offsets, left_x, right_x);
        });

        if !pending_toggles.is_empty() {
            toggle_blocks(&ctx, pending_toggles);
        }
        store_split_ratio(&ctx, file, ratio);
    }

    fn show_column(
        ui: &mut egui::Ui,
        args: ShowColumnArgs<'_>,
        pending_toggles: &mut Vec<egui::Id>,
    ) -> f32 {
        let area_id = match args.side {
            SplitSide::Before => "diff_viewer_h_left",
            SplitSide::After => "diff_viewer_h_right",
        };
        ui.allocate_ui_with_layout(
            egui::vec2(args.width, ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui_column| {
                DiffViewerSplitColumnOps::show(
                    ui_column,
                    SplitColumnParams {
                        area_id,
                        ctx: args.ctx,
                        file: args.file,
                        palette: args.palette,
                        previous_offset: args.previous_offset,
                        side: args.side,
                    },
                    pending_toggles,
                )
            },
        )
        .inner
    }
}

struct ShowColumnArgs<'a> {
    ctx: &'a egui::Context,
    file: &'a crate::diff_review::DiffReviewFile,
    palette: &'a DiffViewerPalette,
    width: f32,
    previous_offset: f32,
    side: SplitSide,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SplitLayout {
    pub(super) available_width: f32,
    pub(super) height: f32,
    left_width: f32,
    right_width: f32,
}

impl SplitLayout {
    fn new(available_width: f32, height: f32, ratio: f32) -> Self {
        let left_width = (available_width - DiffViewerSplitHandleOps::WIDTH) * ratio;
        let right_width = (available_width - DiffViewerSplitHandleOps::WIDTH) - left_width;
        Self {
            available_width,
            height,
            left_width,
            right_width,
        }
    }
}

fn split_ratio(ctx: &egui::Context, file: &crate::diff_review::DiffReviewFile) -> f32 {
    ctx.data(|data| data.get_temp::<f32>(ratio_key(file)))
        .unwrap_or(DEFAULT_SPLIT_RATIO)
        .clamp(SPLIT_MIN_RATIO, SPLIT_MAX_RATIO)
}

fn store_split_ratio(ctx: &egui::Context, file: &crate::diff_review::DiffReviewFile, ratio: f32) {
    ctx.data_mut(|data| data.insert_temp(ratio_key(file), ratio));
}

fn ratio_key(file: &crate::diff_review::DiffReviewFile) -> egui::Id {
    egui::Id::new(("diff_viewer_split_ratio", file.path.as_path()))
}

fn toggle_blocks(ctx: &egui::Context, pending_toggles: Vec<egui::Id>) {
    for id in pending_toggles {
        let expanded = ctx.data(|data| data.get_temp::<bool>(id)).unwrap_or(false);
        ctx.data_mut(|data| data.insert_temp(id, !expanded));
    }
}

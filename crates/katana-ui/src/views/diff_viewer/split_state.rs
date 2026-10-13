use eframe::egui;

pub(super) struct DiffViewerSplitStateOps;

impl DiffViewerSplitStateOps {
    pub(super) fn is_block_expanded(
        ctx: &egui::Context,
        file: &crate::diff_review::DiffReviewFile,
        block: &crate::diff_review::UnchangedBlock,
    ) -> bool {
        ctx.data(|data| data.get_temp::<bool>(Self::block_id(file, block)))
            .unwrap_or(false)
    }

    pub(super) fn block_id(
        file: &crate::diff_review::DiffReviewFile,
        block: &crate::diff_review::UnchangedBlock,
    ) -> egui::Id {
        egui::Id::new((
            "diff_viewer_unchanged_block",
            file.path.as_path(),
            block.before_start_line_number,
            block.after_start_line_number,
            block.line_count,
        ))
    }
}

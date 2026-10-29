use super::{ImageLogicOps, LocalImageLoadingContext, ViewerState, egui};

pub(crate) struct LocalImageDisplayOptions<'a> {
    pub(crate) loading_context: LocalImageLoadingContext,
    pub(crate) viewer_state: Option<&'a mut ViewerState>,
    pub(crate) fullscreen_request: Option<&'a mut Option<usize>>,
}

impl ImageLogicOps {
    pub(crate) fn show_local_image(
        ui: &mut egui::Ui,
        path: &std::path::Path,
        id: usize,
        loader: &super::super::local_image_loader::LocalImageLoader,
        viewer_state: Option<&mut ViewerState>,
        fullscreen_request: Option<&mut Option<usize>>,
        draw_background: impl FnOnce(&mut egui::Ui, egui::Rect, bool),
    ) -> Option<egui::Rect> {
        Self::show_local_image_with_context(
            ui,
            path,
            id,
            loader,
            LocalImageDisplayOptions {
                loading_context: LocalImageLoadingContext::InlineMarkdown,
                viewer_state,
                fullscreen_request,
            },
            draw_background,
        )
    }
}

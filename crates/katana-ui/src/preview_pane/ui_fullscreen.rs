use super::{PreviewPane, RenderedSection};
use eframe::egui;

impl PreviewPane {
    pub(crate) fn render_fullscreen_modal(&mut self, ctx: &egui::Context) {
        let result = crate::preview_pane::FullscreenLogicOps::render_fullscreen_if_active(
            ctx,
            &self.sections,
            self.fullscreen_image,
            &mut self.fullscreen_viewer_state,
            &self.local_image_loader,
        );
        self.apply_fullscreen_result(result, Some(ctx));
    }

    pub(crate) fn apply_fullscreen_result(
        &mut self,
        result: Option<usize>,
        ctx: Option<&egui::Context>,
    ) {
        if result.is_none() && self.fullscreen_image.is_some() {
            /* WHY: 通常プレビューと画像監視を共有するため、終了時は全画面側のtexture所有だけを解放する。 */
            self.fullscreen_viewer_state.reset();
            if let Some(ctx) = ctx
                && !self.was_os_fullscreen_before_modal
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
        }
        self.fullscreen_image = result;
    }

    pub(crate) fn handle_fullscreen_request(
        &mut self,
        request: Option<usize>,
        ctx: Option<&egui::Context>,
    ) {
        if let Some(idx) = request {
            if self.fullscreen_image.is_none()
                && let Some(ctx) = ctx
            {
                let is_native_fs = ctx.input(|i| i.viewport().fullscreen).unwrap_or(false);
                self.was_os_fullscreen_before_modal = is_native_fs;
                if !is_native_fs {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                }
            }
            self.fullscreen_image = Some(idx);
        }
        if let Some(idx) = self.fullscreen_image {
            match self.sections.get(idx) {
                Some(RenderedSection::Image { .. } | RenderedSection::LocalImage { .. }) => {}
                _ => self.fullscreen_image = None,
            }
        }
    }
}

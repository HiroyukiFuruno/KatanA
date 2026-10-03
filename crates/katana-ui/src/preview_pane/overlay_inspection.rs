use eframe::egui;

const INSPECTION_ID: &str = "katana_preview_overlay_inspection";
pub(crate) const ACTIVE_ID: &str = "katana_preview_overlay_inspection_active";
pub(crate) const CODE_COPY_RENDER_ID: &str = "katana_preview_code_copy_control_renders";
pub(crate) const CODE_SELECTION_RENDER_ID: &str = "katana_preview_code_selection_renders";

#[derive(Clone, Copy, Debug, Default)]
pub struct PreviewOverlayInspection {
    pub diagram_control_renders: u32,
    pub image_control_renders: u32,
    pub code_copy_control_renders: u32,
    pub active_markdown_ranges: u32,
    pub hovered_markdown_spans: u32,
    pub image_hover_background_renders: u32,
    pub local_image_hover_background_renders: u32,
    pub code_selection_renders: u32,
    pub markdown_section_renders: u32,
    pub completed_ui_frame_nr: Option<u64>,
}

pub struct PreviewOverlayInspectionOps;

impl PreviewOverlayInspectionOps {
    pub fn reset(ctx: &egui::Context) {
        ctx.data_mut(|data| {
            data.insert_temp(
                egui::Id::new(INSPECTION_ID),
                PreviewOverlayInspection::default(),
            );
            data.insert_temp(egui::Id::new(ACTIVE_ID), true);
            data.insert_temp(egui::Id::new(CODE_COPY_RENDER_ID), 0_u32);
            data.insert_temp(egui::Id::new(CODE_SELECTION_RENDER_ID), 0_u32);
        });
    }

    pub fn snapshot(ctx: &egui::Context) -> Option<PreviewOverlayInspection> {
        ctx.data(|data| {
            let mut inspection: PreviewOverlayInspection =
                data.get_temp(egui::Id::new(INSPECTION_ID))?;
            inspection.code_copy_control_renders = data
                .get_temp(egui::Id::new(CODE_COPY_RENDER_ID))
                .unwrap_or_default();
            inspection.code_selection_renders = data
                .get_temp(egui::Id::new(CODE_SELECTION_RENDER_ID))
                .unwrap_or_default();
            Some(inspection)
        })
    }

    pub fn increment(ctx: &egui::Context, update: impl FnOnce(&mut PreviewOverlayInspection)) {
        ctx.data_mut(|data| {
            if data
                .get_temp::<bool>(egui::Id::new(ACTIVE_ID))
                .unwrap_or(false)
            {
                let inspection = data.get_temp_mut_or_default::<PreviewOverlayInspection>(
                    egui::Id::new(INSPECTION_ID),
                );
                update(inspection);
            }
        });
    }

    pub fn complete(ctx: &egui::Context, ui_frame_nr: u64) {
        ctx.data_mut(|data| {
            data.insert_temp(egui::Id::new(ACTIVE_ID), false);
            let inspection = data
                .get_temp_mut_or_default::<PreviewOverlayInspection>(egui::Id::new(INSPECTION_ID));
            inspection.completed_ui_frame_nr = Some(ui_frame_nr);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactive_increment_does_not_create_inspection() {
        let ctx = egui::Context::default();
        PreviewOverlayInspectionOps::increment(&ctx, |inspection| {
            inspection.diagram_control_renders += 1
        });
        assert!(PreviewOverlayInspectionOps::snapshot(&ctx).is_none());
    }

    #[test]
    fn completed_capture_records_its_single_ui_pass() {
        let ctx = egui::Context::default();
        PreviewOverlayInspectionOps::reset(&ctx);
        PreviewOverlayInspectionOps::increment(&ctx, |inspection| {
            inspection.diagram_control_renders += 1
        });
        PreviewOverlayInspectionOps::complete(&ctx, 42);
        let inspection = PreviewOverlayInspectionOps::snapshot(&ctx).expect("capture enabled");
        assert_eq!(inspection.diagram_control_renders, 1);
        assert_eq!(inspection.completed_ui_frame_nr, Some(42));
    }

    #[test]
    fn increment_after_complete_is_ignored() {
        let ctx = egui::Context::default();
        PreviewOverlayInspectionOps::reset(&ctx);
        PreviewOverlayInspectionOps::complete(&ctx, 7);
        PreviewOverlayInspectionOps::increment(&ctx, |inspection| {
            inspection.diagram_control_renders += 1
        });
        assert_eq!(
            PreviewOverlayInspectionOps::snapshot(&ctx)
                .expect("capture enabled")
                .diagram_control_renders,
            0
        );
    }
}

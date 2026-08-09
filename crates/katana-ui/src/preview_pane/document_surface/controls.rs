use eframe::egui;
use katana_document_viewer::{
    DocumentFitMode, DocumentSurfaceKind, DocumentViewerCommand, ViewerCapabilities, ViewerFeature,
};

use super::types::DocumentSurface;
use super::worker::DocumentWorkerCommand;
use super::worker_support::{MAX_DOCUMENT_RENDER_SCALE, MIN_DOCUMENT_RENDER_SCALE};

const ZOOM_STEP: f32 = 0.25;
const TOOLBAR_BORDER_WIDTH: f32 = 1.0;
const TOOLBAR_HORIZONTAL_MARGIN: i8 = 8;
const TOOLBAR_VERTICAL_MARGIN: i8 = 5;
const CONTROL_WIDTH: f32 = 30.0;
const CONTROL_HEIGHT: f32 = 28.0;

#[derive(Clone, Copy)]
struct DocumentControlColors {
    toolbar: egui::Color32,
    button: egui::Color32,
    border: egui::Color32,
}

pub(super) fn show_controls(
    surface: &mut DocumentSurface,
    ui: &mut egui::Ui,
    frame: &katana_document_viewer::DocumentFrame,
) {
    let messages = crate::i18n::I18nOps::get();
    let colors = document_control_colors(ui);
    let mut toolbar = egui::Frame::group(ui.style()).fill(colors.toolbar);
    toolbar.stroke = egui::Stroke::new(TOOLBAR_BORDER_WIDTH, colors.border);
    toolbar.inner_margin =
        egui::Margin::symmetric(TOOLBAR_HORIZONTAL_MARGIN, TOOLBAR_VERTICAL_MARGIN);
    toolbar.show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            let navigation = supports_navigation(&frame.capabilities);
            if icon_button(
                ui,
                crate::Icon::ChevronLeft,
                &messages.preview.document_controller.previous,
                navigation && frame.state.active_index > 0,
                colors,
            ) {
                surface.queue(DocumentWorkerCommand::Viewer(
                    DocumentViewerCommand::Previous,
                ));
            }
            let current = frame.state.active_index.saturating_add(1).to_string();
            let total = frame.state.item_count.to_string();
            ui.label(crate::i18n::I18nOps::tf(
                &messages.diff_review.file_counter,
                &[("current", &current), ("total", &total)],
            ));
            if icon_button(
                ui,
                crate::Icon::ChevronRight,
                &messages.preview.document_controller.next,
                navigation && frame.state.active_index.saturating_add(1) < frame.state.item_count,
                colors,
            ) {
                surface.queue(DocumentWorkerCommand::Viewer(DocumentViewerCommand::Next));
            }
            if supports(&frame.capabilities, ViewerFeature::Zoom) {
                ui.separator();
                if icon_button(
                    ui,
                    crate::Icon::ZoomOut,
                    &messages.preview.diagram_controller.zoom_out,
                    frame.state.zoom > MIN_DOCUMENT_RENDER_SCALE,
                    colors,
                ) {
                    surface.queue(DocumentWorkerCommand::Viewer(
                        DocumentViewerCommand::SetZoom(
                            (frame.state.zoom - ZOOM_STEP).max(MIN_DOCUMENT_RENDER_SCALE),
                        ),
                    ));
                }
                if icon_button(
                    ui,
                    crate::Icon::ResetView,
                    &messages.preview.document_controller.fit_page,
                    true,
                    colors,
                ) {
                    surface.set_fit(DocumentFitMode::Page);
                }
                if icon_button(
                    ui,
                    crate::Icon::ExpandAll,
                    &messages.preview.document_controller.fit_width,
                    true,
                    colors,
                ) {
                    surface.set_fit(DocumentFitMode::Width);
                }
                if icon_button(
                    ui,
                    crate::Icon::ZoomIn,
                    &messages.preview.diagram_controller.zoom_in,
                    frame.state.zoom < MAX_DOCUMENT_RENDER_SCALE,
                    colors,
                ) {
                    surface.queue(DocumentWorkerCommand::Viewer(
                        DocumentViewerCommand::SetZoom(
                            (frame.state.zoom + ZOOM_STEP).min(MAX_DOCUMENT_RENDER_SCALE),
                        ),
                    ));
                }
            }
            if supports(&frame.capabilities, ViewerFeature::CopyText)
                && frame.surface.kind() == DocumentSurfaceKind::Grid
            {
                ui.separator();
                if icon_button(
                    ui,
                    crate::Icon::Copy,
                    &messages.preview.document_controller.copy_active_cell,
                    true,
                    colors,
                ) {
                    surface.queue(DocumentWorkerCommand::Viewer(
                        DocumentViewerCommand::CopySelection,
                    ));
                }
            }
        });
    });
}

fn supports_navigation(capabilities: &ViewerCapabilities) -> bool {
    [
        ViewerFeature::PageNavigation,
        ViewerFeature::GridNavigation,
        ViewerFeature::SheetNavigation,
        ViewerFeature::SlideNavigation,
    ]
    .into_iter()
    .any(|feature| supports(capabilities, feature))
}

fn supports(capabilities: &ViewerCapabilities, feature: ViewerFeature) -> bool {
    capabilities.status(feature) == katana_document_viewer::ViewerFeatureStatus::Supported
}

fn icon_button(
    ui: &mut egui::Ui,
    icon: crate::Icon,
    tooltip: &str,
    enabled: bool,
    colors: DocumentControlColors,
) -> bool {
    let button = icon
        .button_on_fill(ui, crate::icon::IconSize::Medium, colors.button)
        .stroke(egui::Stroke::new(TOOLBAR_BORDER_WIDTH, colors.border))
        .min_size(egui::vec2(CONTROL_WIDTH, CONTROL_HEIGHT));
    ui.add_enabled(enabled, button)
        .on_hover_text(tooltip)
        .clicked()
}

fn document_control_colors(ui: &egui::Ui) -> DocumentControlColors {
    ui.data(|data| {
        data.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new("katana_theme_colors"))
    })
    .map_or_else(
        || DocumentControlColors {
            toolbar: ui.visuals().panel_fill,
            button: ui.visuals().widgets.inactive.weak_bg_fill,
            border: ui.visuals().widgets.noninteractive.bg_stroke.color,
        },
        |theme| DocumentControlColors {
            toolbar: crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                theme.system.panel_background,
            ),
            button: crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(theme.system.background),
            border: crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(theme.system.border),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{supports, supports_navigation};
    use katana_document_viewer::{ViewerCapabilities, ViewerFeature};

    #[test]
    fn capability_check_does_not_depend_on_surface_frame_storage() {
        let capabilities = ViewerCapabilities::static_page();

        assert!(supports(&capabilities, ViewerFeature::PageNavigation));
        assert!(supports(&capabilities, ViewerFeature::Zoom));
        assert!(!supports(&capabilities, ViewerFeature::GridNavigation));
        assert!(supports_navigation(&capabilities));
    }
}

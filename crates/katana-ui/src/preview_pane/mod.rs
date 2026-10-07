pub mod anchor_map;
pub mod background;
mod cancellable_read;
pub mod core_render;
mod core_render_diagram;
mod core_render_html_document;
mod core_render_lifecycle;
mod core_render_sections;
pub(crate) mod diagram_cache;
pub mod extension_table;
mod render_workers;
pub mod renderer;
pub mod types;
pub mod ui;
pub mod viewer_state;
pub use types::ViewerState;

#[cfg(test)]
mod images_failure_tests;
#[cfg(test)]
mod tests;

pub mod section;
mod section_show;
pub mod utils;
pub use section::*;
mod image_background;
mod image_background_region;
mod image_background_region_model;
pub mod image_fallback;
pub mod images;
pub use images::*;
mod document_surface;
pub mod fullscreen;
pub mod fullscreen_local;
pub mod fullscreen_svg;
mod image_html_surface;
mod image_raster;
mod local_image_loader;
#[cfg(feature = "screenshot-test-hooks")]
mod overlay_inspection;
#[cfg(feature = "screenshot-test-hooks")]
pub mod screenshot_test_hooks {
    pub use super::overlay_inspection::{PreviewOverlayInspection, PreviewOverlayInspectionOps};
}
pub mod slideshow;
pub use fullscreen::*;
pub mod html;
pub mod math;
pub use html::*;
pub use math::*;
pub use renderer::*;
pub use types::*;
pub(crate) mod section_images;
mod section_local_images;
pub(crate) use document_surface::{
    DocumentFailure, DocumentSurfaceSource, DocumentWorkerLifecycle,
};

#[cfg(test)]
pub(crate) fn html_browser_runtime_test_guard() -> std::sync::MutexGuard<'static, ()> {
    crate::test_render_env::RenderEnvLock::lock()
}

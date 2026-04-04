use egui_commonmark::CommonMarkCache;
use katana_core::markdown::DiagramKind;
use katana_core::markdown::outline::OutlineItem;
use katana_core::markdown::svg_rasterize::RasterizedSvg;

pub(crate) const DIAGRAM_SVG_DISPLAY_SCALE: f32 = 1.5;

pub(crate) const RENDER_POLL_INTERVAL_MS: u64 = 50;

#[derive(Default)]
pub struct PreviewPane {
    pub(crate) commonmark_cache: CommonMarkCache,
    pub sections: Vec<RenderedSection>,
    pub outline_items: Vec<OutlineItem>,
    pub heading_anchors: Vec<(std::ops::Range<usize>, egui::Rect)>,
    pub content_top_y: f32,
    pub visible_rect: Option<egui::Rect>,
    pub scroll_request: Option<usize>,
    pub render_rx: Option<std::sync::mpsc::Receiver<RenderMessage>>,
    pub is_loading: bool,
    pub cancel_token: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub(crate) md_file_path: std::path::PathBuf,
    pub concurrency_reduction_requested: bool,
    pub image_preload_queue: Vec<std::path::PathBuf>,
    pub image_cache: std::collections::HashSet<std::path::PathBuf>,
    pub viewer_states: Vec<ViewerState>,
    pub fullscreen_image: Option<usize>,
    pub fullscreen_viewer_state: ViewerState,
    pub was_os_fullscreen_before_modal: bool,
    pub(crate) repaint_ctx: Option<egui::Context>,
}

pub(crate) struct RenderJob {
    pub(crate) kind: DiagramKind,
    pub(crate) src: String,
    pub(crate) path: std::path::PathBuf,
    pub(crate) cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
    pub(crate) force: bool,
    pub(crate) source_lines: usize,
}

pub enum RenderMessage {
    Section {
        kind: String,
        source: String,
        section: RenderedSection,
    },
    ReduceConcurrency,
}

pub struct PreviewPaneUtilsOps;

#[derive(Debug, Clone)]
pub enum RenderedSection {
    Markdown(String),
    Image {
        svg_data: RasterizedSvg,
        alt: String,
        source_lines: usize,
    },
    LocalImage {
        path: std::path::PathBuf,
        alt: String,
        source_lines: usize,
    },
    Error {
        kind: String,
        _source: String,
        message: String,
        source_lines: usize,
    },
    CommandNotFound {
        tool_name: String,
        install_hint: String,
        _source: String,
        source_lines: usize,
    },
    NotInstalled {
        kind: String,
        download_url: String,
        install_path: std::path::PathBuf,
        source_lines: usize,
    },
    Pending {
        kind: String,
        source: String,
        source_lines: usize,
    },
}

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub dest: std::path::PathBuf,
}

#[derive(Clone)]
pub struct MathJaxCache(
    pub(crate) std::sync::Arc<egui::mutex::Mutex<std::collections::BTreeMap<String, String>>>,
);

impl Default for MathJaxCache {
    fn default() -> Self {
        Self(std::sync::Arc::new(egui::mutex::Mutex::new(
            Default::default(),
        )))
    }
}

pub struct MathLogicOps;

pub struct HtmlLogicOps;

pub struct RendererLogicOps;

#[derive(Clone, PartialEq)]
pub struct ViewerState {
    pub zoom: f32,
    pub pan: egui::Vec2,
    pub texture: Option<egui::TextureHandle>,
}

impl std::fmt::Debug for ViewerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewerState")
            .field("zoom", &self.zoom)
            .field("pan", &self.pan)
            .field("texture", &self.texture.as_ref().map(|t| t.id()))
            .finish()
    }
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            texture: None,
        }
    }
}

impl ViewerState {
    pub fn zoom_in(&mut self) {
        const VIEWER_ZOOM_STEP: f32 = 0.25;
        const VIEWER_ZOOM_MAX: f32 = 4.0;
        self.zoom = (self.zoom + VIEWER_ZOOM_STEP).min(VIEWER_ZOOM_MAX);
    }

    pub fn zoom_out(&mut self) {
        const VIEWER_ZOOM_STEP: f32 = 0.25;
        const VIEWER_ZOOM_MIN: f32 = 0.25;
        self.zoom = (self.zoom - VIEWER_ZOOM_STEP).max(VIEWER_ZOOM_MIN);
    }

    pub fn pan_by(&mut self, delta: egui::Vec2) {
        self.pan += delta;
    }

    pub fn pan_up(&mut self) {
        const VIEWER_PAN_STEP: f32 = 50.0;
        self.pan_by(egui::vec2(0.0, -VIEWER_PAN_STEP));
    }

    pub fn pan_down(&mut self) {
        const VIEWER_PAN_STEP: f32 = 50.0;
        self.pan_by(egui::vec2(0.0, VIEWER_PAN_STEP));
    }

    pub fn pan_left(&mut self) {
        const VIEWER_PAN_STEP: f32 = 50.0;
        self.pan_by(egui::vec2(-VIEWER_PAN_STEP, 0.0));
    }

    pub fn pan_right(&mut self) {
        const VIEWER_PAN_STEP: f32 = 50.0;
        self.pan_by(egui::vec2(VIEWER_PAN_STEP, 0.0));
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

pub struct ImageLogicOps;

pub struct SectionLogicOps;

pub struct FullscreenLogicOps;

use egui_commonmark::CommonMarkCache;
use katana_core::markdown::DiagramKind;
use katana_core::markdown::outline::OutlineItem;
use katana_core::markdown::svg_rasterize::RasterizedSvg;

pub(crate) const DIAGRAM_SVG_DISPLAY_SCALE: f32 = 1.5;

pub(crate) const RENDER_POLL_INTERVAL_MS: u64 = 50;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SectionLifecycle {
    pub is_loaded: bool,
    pub is_drawn: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentAnchorMapItem {
    pub kind: katana_core::markdown::outline::AnchorKind,
    pub index: Option<usize>,
    pub line_span: std::ops::Range<usize>,
    pub rect: Option<egui::Rect>,
}

#[derive(Default)]
pub struct PreviewPane {
    pub(crate) commonmark_cache: CommonMarkCache,
    pub sections: Vec<RenderedSection>,
    pub outline_items: Vec<OutlineItem>,
    pub document_anchors: Vec<katana_core::markdown::outline::DocumentAnchor>,
    pub anchor_map: Vec<DocumentAnchorMapItem>,
    pub heading_anchors: Vec<(std::ops::Range<usize>, egui::Rect)>,
    pub block_anchors: Vec<(std::ops::Range<usize>, egui::Rect)>,
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
    pub session_generation: u64,
    pub section_lifecycle: Vec<SectionLifecycle>,
}

pub(crate) struct RenderJob {
    pub(crate) kind: DiagramKind,
    pub(crate) src: String,
    pub(crate) path: std::path::PathBuf,
    pub(crate) cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
    pub(crate) force: bool,
    pub(crate) source_lines: usize,
    pub(crate) generation: u64,
    pub(crate) ordinal: usize,
}

pub enum RenderMessage {
    Section {
        generation: u64,
        ordinal: usize,
        section: RenderedSection,
    },
    ReduceConcurrency,
}

pub struct PreviewPaneUtilsOps;

#[derive(Debug, Clone)]
pub enum RenderedSection {
    Markdown(String, usize),
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
    pub closing_since: Option<f64>,
}

pub struct ImageLogicOps;
pub struct SectionLogicOps;
pub struct SectionImageOps;
pub struct FullscreenLogicOps;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub schema_version: String,
    pub name: String,
    #[serde(default)]
    pub fixture: Fixture,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Fixture {
    #[serde(default)]
    pub workspace_files: Vec<WorkspaceFile>,
    pub workspace_dir: Option<String>,
    #[serde(default)]
    pub settings: FixtureSettings,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceFile {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct FixtureSettings {
    pub theme: Option<String>,
    pub locale: Option<String>,
    pub explorer_visible: Option<bool>,
    pub no_extension: Option<bool>,
    pub linter_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Step {
    Launch(LaunchStep),
    Wait(WaitStep),
    Screenshot(ScreenshotStep),
    RecordStart(RecordStartStep),
    RecordStop(RecordStopStep),
    Scroll(ScrollStep),
    /// Export the active document as PNG using the app's export pipeline, then save to output_dir.
    ExportPng(ExportPngStep),
    /// Open a file by name from the workspace tree.
    OpenFile(OpenFileStep),
    /// Trigger a named UI action (e.g. toggle_toc, toggle_split_view).
    Action(ActionStep),
    Quit,
}

#[derive(Debug, Deserialize)]
pub struct ExportPngStep {
    pub output_name: String,
}

#[derive(Debug, Deserialize)]
pub struct LaunchStep {
    pub viewport: Option<Viewport>,
    #[serde(default = "default_wait_seconds")]
    pub wait_seconds: f64,
}

#[derive(Debug, Deserialize)]
pub struct WaitStep {
    pub seconds: f64,
}

#[derive(Debug, Deserialize)]
pub struct ScreenshotStep {
    pub output_name: String,
    /// Crop the rendered image to this rect (physical pixels, after pixels_per_point scaling).
    pub crop: Option<CropRect>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum VideoFormat {
    Webm,
    Mp4,
}

#[derive(Debug, Deserialize)]
pub struct RecordStartStep {
    pub output_name: String,
    /// Optional output format. Defaults to webm.
    pub format: Option<VideoFormat>,
    /// Optional target fps used for encoding. Defaults to 24.
    pub fps: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct RecordStopStep {}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ScrollDirection {
    Up,
    Down,
}

#[derive(Debug, Deserialize)]
pub struct ScrollStep {
    pub direction: ScrollDirection,
    /// Logical-pixel amount to scroll.
    pub pixels: f32,
    /// Duration for the scroll interaction.
    pub duration_seconds: f64,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct CropRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct OpenFileStep {
    pub file_name: String,
    #[serde(default = "default_open_file_wait")]
    pub wait_seconds: f64,
}

/// Named UI actions that the harness can trigger after launch.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UiAction {
    ToggleToc,
    ToggleSplitView,
    ToggleSettings,
    ToggleExplorer,
    ToggleSlideshow,
    ToggleStoryPanel,
    ToggleExportPanel,
    OpenChangelog,
    /// Open settings and navigate to a specific tab.
    /// Tab names: "theme", "icons", "font", "layout", "workspace", "updates", "behavior", "shortcuts"
    OpenSettingsTab { tab: String },
    /// Force-open a collapsing accordion by its egui Id source string.
    ForceOpenAccordion { id: String },
    /// Open the icons advanced-settings panel (full-height override table view).
    OpenIconsAdvancedPanel,
    /// Scroll down in the currently visible panel by the given logical-pixel amount.
    ScrollDown { amount: f32 },
    /// Directly set the vertical scroll offset of an egui ScrollArea by its id_salt string.
    SetScrollOffset { id: String, y: f32 },
    /// Open the first (top) section in the changelog accordion.
    OpenFirstChangelogSection,
    /// Set the editor view mode. mode: "preview_only" | "code_only" | "split"
    SetViewMode { mode: String },
    /// Open the command palette, type a query, and optionally execute the top result.
    RunCommandPalette {
        query: String,
        #[serde(default)]
        katana_mode: bool,
        #[serde(default)]
        execute_first: bool,
        #[serde(default)]
        keystroke_delay_seconds: Option<f64>,
        #[serde(default)]
        pause_after_seconds: Option<f64>,
    },
    /// Open global search and populate a query in the selected tab.
    RunGlobalSearch {
        query: String,
        tab: String,
        #[serde(default)]
        keystroke_delay_seconds: Option<f64>,
        #[serde(default)]
        pause_after_seconds: Option<f64>,
    },
    /// Open in-document search, type a query, and optionally advance matches.
    RunDocumentSearch {
        query: String,
        #[serde(default)]
        next_count: Option<u32>,
        #[serde(default)]
        keystroke_delay_seconds: Option<f64>,
        #[serde(default)]
        pause_after_seconds: Option<f64>,
    },
    /// Select a built-in theme preset from the visible Settings > Theme preset list.
    SelectThemePresetInSettings { preset: String },
    /// Advance slideshow pages as if paging through fullscreen content.
    SlideshowNavigate { direction: String, steps: u32, wait_seconds: f64 },
}

#[derive(Debug, Deserialize)]
pub struct ActionStep {
    pub action: UiAction,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

fn default_wait_seconds() -> f64 {
    3.0
}

fn default_open_file_wait() -> f64 {
    1.0
}

const SUPPORTED_SCHEMA_VERSION: &str = "1";

pub fn load(path: &std::path::Path) -> anyhow::Result<Request> {
    let json = std::fs::read_to_string(path)?;
    let req: Request = serde_json::from_str(&json)?;
    anyhow::ensure!(
        req.schema_version == SUPPORTED_SCHEMA_VERSION,
        "unsupported schema_version {:?}, expected {:?}",
        req.schema_version,
        SUPPORTED_SCHEMA_VERSION,
    );
    anyhow::ensure!(!req.name.is_empty(), "request.name must not be empty");
    anyhow::ensure!(!req.steps.is_empty(), "request.steps must not be empty");
    Ok(req)
}

use serde::Deserialize;
use std::collections::HashMap;

use crate::capture::PngBounds;

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
    pub http_server: Option<HttpServerFixture>,
}

#[derive(Debug, Deserialize)]
pub struct HttpServerFixture {
    pub mount_prefix: String,
    #[serde(default)]
    pub redirects: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum WorkspaceFile {
    Text { name: String, content: String },
    Copy { name: String, source: String },
}

impl WorkspaceFile {
    pub fn name(&self) -> &str {
        match self {
            Self::Text { name, .. } | Self::Copy { name, .. } => name,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct FixtureSettings {
    pub theme: Option<String>,
    pub preset: Option<String>,
    pub locale: Option<String>,
    pub explorer_visible: Option<bool>,
    pub no_extension: Option<bool>,
    pub linter_enabled: Option<bool>,
    pub slideshow_show_diagram_controls: Option<bool>,
    pub preview_show_diagram_controls: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Step {
    Launch(LaunchStep),
    Wait(WaitStep),
    Screenshot(ScreenshotStep),
    AssertScreenshotChanged(AssertScreenshotChangedStep),
    AssertScreenshotContainsRgb(AssertScreenshotContainsRgbStep),
    RecordStart(RecordStartStep),
    RecordStop(RecordStopStep),
    Scroll(ScrollStep),
    /// Export the active document as PNG using the app's export pipeline, then save to output_dir.
    ExportPng(ExportPngStep),
    /// Open a file by name from the workspace tree.
    OpenFile(OpenFileStep),
    /// Open an absolute workspace path.
    OpenWorkspace(OpenWorkspaceStep),
    /// Assert that the active document matches the expected screenshot state.
    AssertActiveDocument(AssertActiveDocumentStep),
    /// Assert that the active KRR frame retains the expected complete origin.
    AssertHtmlBrowserOrigin(AssertHtmlBrowserOriginStep),
    /// Assert pixels directly in the active KRR RGBA frame.
    AssertHtmlBrowserFrameContainsRgb(AssertHtmlBrowserFrameContainsRgbStep),
    /// Assert that the KRR frame dimensions equal the native HTML display rect.
    AssertHtmlBrowserViewportMatchesDisplayRect,
    /// Assert that the composed HTML display corners contain the page-owned color.
    AssertHtmlBrowserDisplayCornersRgb(AssertHtmlBrowserDisplayCornersRgbStep),
    /// Assert principal-document and subresource requests received by the loopback fixture.
    AssertHttpRequests(AssertHttpRequestsStep),
    /// Assert URL-tab history entries by final document URL suffix.
    AssertUrlHistory(AssertUrlHistoryStep),
    /// Assert that the active diff review state matches the expected content.
    AssertDiffReview(AssertDiffReviewStep),
    /// Trigger a named UI action (e.g. toggle_toc, toggle_split_view).
    Action(ActionStep),
    /// Drag a labelled source node to a labelled target node.
    Drag(DragStep),
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

#[derive(Debug, Deserialize)]
pub struct AssertScreenshotChangedStep {
    pub baseline: String,
    pub current: String,
    pub min_changed_pixels: u64,
}

#[derive(Debug, Deserialize)]
pub struct AssertScreenshotContainsRgbStep {
    pub screenshot: String,
    pub rgb: [u8; 3],
    #[serde(default)]
    pub tolerance: u8,
    pub min_pixels: u64,
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

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
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

#[derive(Debug, Deserialize)]
pub struct OpenWorkspaceStep {
    pub path: String,
    #[serde(default = "default_open_file_wait")]
    pub wait_seconds: f64,
}

#[derive(Debug, Deserialize)]
pub struct AssertActiveDocumentStep {
    pub path_contains: String,
}

#[derive(Debug, Deserialize)]
pub struct AssertHtmlBrowserOriginStep {
    pub origin_ends_with: String,
    #[serde(default = "default_async_assert_timeout_seconds")]
    pub timeout_seconds: f64,
}

#[derive(Debug, Deserialize)]
pub struct AssertHtmlBrowserFrameContainsRgbStep {
    pub rgb: [u8; 3],
    pub min_pixels: u64,
    #[serde(default = "default_async_assert_timeout_seconds")]
    pub timeout_seconds: f64,
}

#[derive(Debug, Deserialize)]
pub struct AssertHtmlBrowserDisplayCornersRgbStep {
    pub rgb: [u8; 3],
    #[serde(default)]
    pub tolerance: u8,
}

#[derive(Debug, Deserialize)]
pub struct AssertHttpRequestsStep {
    pub paths: Vec<String>,
    #[serde(default = "default_async_assert_timeout_seconds")]
    pub timeout_seconds: f64,
}

#[derive(Debug, Deserialize)]
pub struct AssertUrlHistoryStep {
    pub origin_suffixes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssertDiffReviewStep {
    #[serde(default)]
    pub file_count: Option<usize>,
    #[serde(default)]
    pub target_path_contains: Option<String>,
    #[serde(default)]
    pub before_contains: Option<String>,
    #[serde(default)]
    pub after_contains: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DragStep {
    pub from_label: String,
    pub to_label: String,
    /// Number of pointer move frames while dragging. Defaults to 20.
    #[serde(default)]
    pub move_steps: Option<u32>,
    /// Hold time on mouse down before move starts (seconds).
    #[serde(default)]
    pub hold_seconds: Option<f64>,
    /// Pause time after release (seconds).
    #[serde(default = "default_open_file_wait")]
    pub wait_seconds: f64,
}

/// Named UI actions that the harness can trigger after launch.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UiAction {
    /// Open a URL served by the request's loopback HTTP fixture.
    OpenFixtureUrl {
        path: String,
        wait_seconds: f64,
    },
    ToggleToc,
    ToggleSplitView,
    ToggleSettings,
    ToggleExplorer,
    ToggleSlideshow,
    ToggleStoryPanel,
    ToggleExportPanel,
    OpenChangelog,
    OpenHelpDemo,
    SelectNextTab,
    /// Select an already-open demo tab by its file name, e.g. "katana-architecture.md".
    SelectDemoTab {
        file_name: String,
    },
    /// Open the Problems panel without relying on screen coordinates.
    OpenProblemsPanel,
    /// Close the global search modal if it is open.
    CloseSearchModal,
    /// Close the in-document search bar if it is open.
    CloseDocSearch,
    /// Refresh diagnostics for currently open Markdown documents.
    RefreshDiagnostics,
    /// Reload the active document through the same action exposed by KatanA.
    RefreshDocument,
    /// Resize the active window or harness viewport.
    ResizeWindow {
        width: u32,
        height: u32,
        wait_seconds: f64,
    },
    /// Apply all lint fixes for the active file and open the diff review.
    ApplyLintFixesForActiveFile,
    /// Open settings and navigate to a specific tab.
    /// Tab names: "theme", "icons", "font", "layout", "workspace", "updates", "behavior", "shortcuts"
    OpenSettingsTab {
        tab: String,
    },
    /// Force-open a collapsing accordion by its egui Id source string.
    ForceOpenAccordion {
        id: String,
    },
    /// Open the icons advanced-settings panel (full-height override table view).
    OpenIconsAdvancedPanel,
    /// Scroll down in the currently visible panel by the given logical-pixel amount.
    ScrollDown {
        amount: f32,
    },
    /// Directly set the vertical scroll offset of an egui ScrollArea by its id_salt string.
    SetScrollOffset {
        id: String,
        y: f32,
    },
    /// Open the first (top) section in the changelog accordion.
    OpenFirstChangelogSection,
    /// Set the editor view mode. mode: "preview_only" | "code_only" | "split"
    SetViewMode {
        mode: String,
    },
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
    SelectThemePresetInSettings {
        preset: String,
    },
    /// Advance slideshow pages as if paging through fullscreen content.
    SlideshowNavigate {
        direction: String,
        steps: u32,
        wait_seconds: f64,
    },
    /// Click a widget by its accessibility label.
    ClickNode {
        label: String,
        button: ClickButton,
        wait_seconds: f64,
    },
    /// Move the pointer to a logical viewport coordinate without clicking.
    HoverAt {
        x: f32,
        y: f32,
        wait_seconds: f64,
    },
    /// Click a logical viewport coordinate.
    ClickAt {
        x: f32,
        y: f32,
        button: ClickButton,
        wait_seconds: f64,
    },
    /// Detect a rendered RGB region and click its center.
    ClickRgbRegion {
        rgb: [u8; 3],
        #[serde(default)]
        tolerance: u8,
        min_region_pixels: u64,
        #[serde(default)]
        search_bounds: Option<PngBounds>,
        button: ClickButton,
        wait_seconds: f64,
    },
    /// Type text into the currently focused control.
    TypeText {
        text: String,
        wait_seconds: f64,
    },
    /// Drag between two widgets by label.
    DragByLabel {
        from_label: String,
        to_label: String,
        #[serde(default)]
        move_steps: Option<u32>,
        #[serde(default)]
        hold_seconds: Option<f64>,
        #[serde(default = "default_open_file_wait")]
        wait_seconds: f64,
    },
    /// Confirm the currently open diff review file.
    ConfirmCurrentDiffReviewFile,
}

const fn default_drag_steps() -> u32 {
    20
}

impl DragStep {
    pub fn move_steps(&self) -> u32 {
        self.move_steps.unwrap_or(default_drag_steps())
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClickButton {
    Primary,
    Secondary,
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

fn default_async_assert_timeout_seconds() -> f64 {
    10.0
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

#[cfg(test)]
mod tests {
    use super::{Request, ScrollDirection, Step};

    #[test]
    fn browser_evidence_actions_are_valid_request_steps() {
        let request = serde_json::from_str::<Request>(
            r#"{
                "schema_version": "1",
                "name": "browser-evidence",
                "fixture": {
                    "http_server": {
                        "mount_prefix": "/app/",
                        "redirects": { "/start": "/app/index.html" }
                    }
                },
                "steps": [
                    {
                        "type": "action",
                        "action": {
                            "open_fixture_url": {
                                "path": "/start",
                                "wait_seconds": 0.5
                            }
                        }
                    },
                    {
                        "type": "action",
                        "action": {
                            "click_at": {
                                "x": 120.0,
                                "y": 240.0,
                                "button": "primary",
                                "wait_seconds": 0.5
                            }
                        }
                    },
                    {
                        "type": "action",
                        "action": {
                            "type_text": {
                                "text": "KRR input",
                                "wait_seconds": 0.5
                            }
                        }
                    },
                    {
                        "type": "action",
                        "action": {
                            "click_rgb_region": {
                                "rgb": [31, 95, 139],
                                "min_region_pixels": 100,
                                "search_bounds": {
                                    "x": 400,
                                    "y": 200,
                                    "width": 1200,
                                    "height": 800
                                },
                                "button": "primary",
                                "wait_seconds": 0.5
                            }
                        }
                    },
                    {
                        "type": "action",
                        "action": "refresh_document"
                    },
                    {
                        "type": "action",
                        "action": {
                            "resize_window": {
                                "width": 1100,
                                "height": 700,
                                "wait_seconds": 0.5
                            }
                        }
                    },
                    {
                        "type": "assert_screenshot_changed",
                        "baseline": "before",
                        "current": "after",
                        "min_changed_pixels": 100
                    },
                    {
                      "type": "assert_screenshot_contains_rgb",
                        "screenshot": "after",
                        "rgb": [184, 242, 208],
                        "tolerance": 2,
                      "min_pixels": 100
                    },
                    {
                      "type": "assert_html_browser_viewport_matches_display_rect"
                    },
                    {
                      "type": "assert_html_browser_display_corners_rgb",
                      "rgb": [216, 243, 220],
                      "tolerance": 2
                    },
                    {
                      "type": "assert_http_requests",
                      "paths": ["/start", "/app/index.html"]
                    },
                    {
                      "type": "assert_url_history",
                      "origin_suffixes": ["/app/index.html"]
                    }
                ]
            }"#,
        );

        assert!(request.is_ok(), "{request:?}");
    }

    #[test]
    fn scroll_steps_accept_horizontal_and_vertical_directions(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = serde_json::from_str::<Request>(
            r#"{
                "schema_version": "1",
                "name": "four-axis-scroll",
                "steps": [
                    { "type": "scroll", "direction": "up", "pixels": 10.0, "duration_seconds": 0.1 },
                    { "type": "scroll", "direction": "down", "pixels": 10.0, "duration_seconds": 0.1 },
                    { "type": "scroll", "direction": "left", "pixels": 10.0, "duration_seconds": 0.1 },
                    { "type": "scroll", "direction": "right", "pixels": 10.0, "duration_seconds": 0.1 }
                ]
            }"#,
        )?;
        let directions = request
            .steps
            .iter()
            .filter_map(|step| match step {
                Step::Scroll(scroll) => Some(scroll.direction),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            directions,
            vec![
                ScrollDirection::Up,
                ScrollDirection::Down,
                ScrollDirection::Left,
                ScrollDirection::Right,
            ]
        );
        Ok(())
    }
}

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
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Step {
    Launch(LaunchStep),
    Wait(WaitStep),
    Screenshot(ScreenshotStep),
    OpenFile(OpenFileStep),
    Quit,
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
}

#[derive(Debug, Deserialize)]
pub struct OpenFileStep {
    pub file_name: String,
    #[serde(default = "default_open_file_wait")]
    pub wait_seconds: f64,
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

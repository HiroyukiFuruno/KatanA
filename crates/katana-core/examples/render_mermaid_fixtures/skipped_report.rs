use serde::Serialize;
use std::path::Path;

type DynError = Box<dyn std::error::Error>;

#[derive(Default, Serialize)]
pub struct SkippedFixtureReport {
    skipped: Vec<SkippedFixture>,
}

impl SkippedFixtureReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, slug: &str, fixture_path: &Path, message: &str) {
        self.skipped.push(SkippedFixture {
            slug: slug.to_string(),
            file_name: fixture_path
                .file_name()
                .and_then(|it| it.to_str())
                .unwrap_or("fixture.md")
                .to_string(),
            message: message.to_string(),
        });
    }

    pub fn write(&self, output_dir: &Path) -> Result<(), DynError> {
        let output_path = output_dir.join("render-skipped.json");
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(output_path, format!("{json}\n"))?;
        Ok(())
    }
}

#[derive(Serialize)]
struct SkippedFixture {
    slug: String,
    #[serde(rename = "fileName")]
    file_name: String,
    message: String,
}

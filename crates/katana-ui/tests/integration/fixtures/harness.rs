use eframe::egui;
use egui_kittest::Harness;
// use egui_kittest::kittest::Queryable;
use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub const PANEL_WIDTH: f32 = 800.0;
// pub const CENTERING_TOLERANCE: f64 = 50.0;

#[derive(Clone)]
pub struct FixtureSnapshot {
    pub fixture_path: PathBuf,
    pub source: String,
    pub sections: Vec<RenderedSection>,
}

pub fn fixture_cache() -> &'static Mutex<HashMap<String, FixtureSnapshot>> {
    static FIXTURE_CACHE: OnceLock<Mutex<HashMap<String, FixtureSnapshot>>> = OnceLock::new();
    FIXTURE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn load_fixture_snapshot(filename: &str) -> FixtureSnapshot {
    if let Some(snapshot) = fixture_cache().lock().unwrap().get(filename).cloned() {
        return snapshot;
    }

    static GENERATOR_LOCK: Mutex<()> = Mutex::new(());
    let _guard = GENERATOR_LOCK.lock().unwrap();

    if let Some(snapshot) = fixture_cache().lock().unwrap().get(filename).cloned() {
        return snapshot;
    }

    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures")
        .join(filename);
    let source = std::fs::read_to_string(&fixture_path).unwrap();

    let mut pane = PreviewPane::default();
    pane.full_render(
        &source,
        &fixture_path,
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        2,
    );
    pane.wait_for_renders();
    let sections = std::mem::take(&mut pane.sections);

    let snapshot = FixtureSnapshot {
        fixture_path,
        source,
        sections,
    };
    fixture_cache()
        .lock()
        .unwrap()
        .insert(filename.to_string(), snapshot.clone());
    snapshot
}

pub fn load_fixture(filename: &str) -> (PreviewPane, PathBuf, String) {
    let snapshot = load_fixture_snapshot(filename);
    let mut pane = PreviewPane::default();
    pane.sections = snapshot.sections.clone();
    (pane, snapshot.fixture_path, snapshot.source)
}

pub fn extract_section(source: &str, start_marker: &str, end_marker: &str) -> String {
    let start_pos = source.find(start_marker).unwrap() + start_marker.len();
    let after_start = source[start_pos..]
        .find('\n')
        .map(|p| start_pos + p + 1)
        .unwrap_or(start_pos);
    let end_pos = source[after_start..]
        .find(end_marker)
        .map(|p| after_start + p)
        .unwrap_or(source.len());
    source[after_start..end_pos].trim().to_string()
}

pub fn build_harness(sections: Vec<RenderedSection>, width: f32, height: f32) -> Harness<'static> {
    Harness::builder()
        .with_size(egui::vec2(width, height))
        .build_ui(move |ui| {
            let mut pane = PreviewPane::default();
            pane.sections = sections.clone();
            pane.show_content(ui, None, None, None, None);
        })
}

use super::{AppAction, PreviewMenu, PreviewMenuAvailability, egui, make_app};
use crate::app::action::ActionOps;
use crate::preview_pane::DocumentWorkerLifecycle;
use crate::shell::{KatanaApp, TabPreviewCache};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const FRAME_TIMEOUT: Duration = Duration::from_secs(15);
const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);
const TEXTURE_SIDE_LIMIT: usize = 8_192;
const VIEWPORT_WIDTH: f32 = 960.0;
const VIEWPORT_HEIGHT: f32 = 640.0;

fn preview_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        )),
        max_texture_side: Some(TEXTURE_SIDE_LIMIT),
        ..egui::RawInput::default()
    }
}

fn fixture_paths() -> Option<Vec<PathBuf>> {
    let directory = std::env::var_os("KATANA_DOCUMENT_FIXTURE_DIR")?;
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .expect("fixture directory")
        .map(|entry| entry.expect("fixture entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("pptx"))
        })
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "explicit fixture directory has no PPTX");
    let worker = std::env::var_os("KATANA_KDV_OFFICE_WORKER")
        .map(PathBuf::from)
        .expect("acceptance requires an explicit release office worker");
    assert!(
        worker.is_file(),
        "explicit release office worker is missing"
    );
    Some(paths)
}

fn pptx_app(path: &Path) -> KatanaApp {
    let mut app = make_app();
    app.state
        .document
        .open_documents
        .push(katana_core::document::Document::new_empty(path));
    app.state.document.active_doc_idx = Some(0);
    let mut pane = crate::preview_pane::PreviewPane::default();
    pane.full_render_document_path(path, false);
    app.tab_previews.push(TabPreviewCache {
        path: path.to_path_buf(),
        pane,
        hash: 0,
    });
    app
}

fn await_frame(app: &mut KatanaApp, context: &egui::Context, index: usize) {
    let deadline = Instant::now() + FRAME_TIMEOUT;
    loop {
        let pane = &mut app.tab_previews[0].pane;
        let mut output = context.run_ui(preview_input(), |ui| {
            pane.show_document_surface(ui);
        });
        output.textures_delta.clear();
        assert!(
            pane.document_failure_for_test().is_none(),
            "{:?}",
            pane.document_failure_for_test()
        );
        if pane
            .document_slideshow_state()
            .is_some_and(|state| state.0 == index)
        {
            return;
        }
        assert!(Instant::now() < deadline, "PPTX frame deadline exceeded");
        std::thread::yield_now();
    }
}

fn assert_ready_actions(app: &mut KatanaApp, context: &egui::Context) {
    assert!(PreviewMenuAvailability::for_app(
        app,
        PreviewMenu::Slideshow
    ));
    app.process_action(context, AppAction::ToggleSlideshow);
    let mut output = context.run_ui(preview_input(), |ui| {
        app.show_system_modals(ui.ctx());
    });
    output.textures_delta.clear();
    assert!(app.state.layout.show_slideshow);
    app.tab_previews[0].pane.document_slideshow_step(true);
    assert!(PreviewMenuAvailability::for_app(
        app,
        PreviewMenu::Slideshow
    ));
    await_frame(app, context, 1);
    assert_active_tab_contract(app);
    assert_refresh_keeps_ready_frame(app);
}

fn assert_active_tab_contract(app: &mut KatanaApp) {
    let path = app.tab_previews[0].path.with_file_name("unopened.pptx");
    app.state
        .document
        .open_documents
        .push(katana_core::document::Document::new_empty(path));
    app.state.document.active_doc_idx = Some(1);
    assert!(!PreviewMenuAvailability::for_app(
        app,
        PreviewMenu::Slideshow
    ));
    app.state.document.active_doc_idx = Some(0);
    assert!(PreviewMenuAvailability::for_app(
        app,
        PreviewMenu::Slideshow
    ));
}

fn assert_refresh_keeps_ready_frame(app: &mut KatanaApp) {
    let path = app.tab_previews[0].path.clone();
    app.tab_previews[0]
        .pane
        .full_render_document_path(&path, false);
    assert!(PreviewMenuAvailability::for_app(
        app,
        PreviewMenu::Slideshow
    ));
}

fn await_closed(baseline: usize) {
    let deadline = Instant::now() + CLOSE_TIMEOUT;
    while DocumentWorkerLifecycle::live_count() > baseline {
        assert!(
            Instant::now() < deadline,
            "PPTX worker close deadline exceeded"
        );
        std::thread::yield_now();
    }
    assert_eq!(DocumentWorkerLifecycle::live_count(), baseline);
}

#[test]
fn external_pptx_ready_slideshow_accepts_active_frame_and_rejects_stale_tab() {
    let Some(paths) = fixture_paths() else { return };
    let _guard = crate::test_render_env::RenderEnvLock::lock();
    for (fixture, path) in paths.iter().enumerate() {
        let baseline = DocumentWorkerLifecycle::live_count();
        let context = egui::Context::default();
        let mut app = pptx_app(path);
        assert!(!PreviewMenuAvailability::for_app(
            &app,
            PreviewMenu::Slideshow
        ));
        await_frame(&mut app, &context, 0);
        assert_ready_actions(&mut app, &context);
        drop(app);
        await_closed(baseline);
        eprintln!(
            "KATANA_PPTX_SLIDESHOW_ACCEPTANCE fixture={fixture} ready=true next=true stale_rejected=true close=true"
        );
    }
}

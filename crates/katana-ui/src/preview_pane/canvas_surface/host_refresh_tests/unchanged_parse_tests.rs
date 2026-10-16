use super::{ActionOps, CanvasRefreshTest, FIXTURE, PARSE_TIMEOUT};
use crate::preview_pane::canvas_surface::state::CanvasLoadState;
use eframe::egui;
use std::sync::Arc;

#[test]
fn unchanged_refresh_returns_the_same_worker_document_arc() {
    let directory = tempfile::tempdir().expect("Canvas directory");
    let path = directory.path().join("graph.canvas");
    std::fs::write(&path, FIXTURE).expect("Canvas fixture");
    let context = egui::Context::default();
    let mut app = CanvasRefreshTest::app(&path);
    CanvasRefreshTest::finish(&mut app, &context);
    let initial = ready_document(&app);

    app.process_action(
        &context,
        crate::app_state::AppAction::RefreshDocument { is_manual: false },
    );
    let returned = worker_document(&app);
    assert!(
        Arc::ptr_eq(&initial, &returned),
        "unchanged Canvas was reparsed"
    );
}

fn worker_document(app: &crate::shell::KatanaApp) -> Arc<katana_core::canvas::CanvasDocument> {
    let request = app.tab_previews[0]
        .pane
        .canvas_request
        .as_ref()
        .expect("actual Canvas refresh request");
    let deadline = std::time::Instant::now() + PARSE_TIMEOUT;
    loop {
        assert!(
            std::time::Instant::now() < deadline,
            "Canvas refresh timed out"
        );
        if let Some(result) = request.poll().expect("actual Canvas worker result") {
            return result.result.expect("valid Canvas worker document");
        }
        std::thread::yield_now();
    }
}

fn ready_document(app: &crate::shell::KatanaApp) -> Arc<katana_core::canvas::CanvasDocument> {
    let surface = app.tab_previews[0]
        .pane
        .canvas_surface
        .as_ref()
        .expect("Canvas surface");
    let CanvasLoadState::Ready(document) = &surface.state else {
        panic!("Canvas must be Ready");
    };
    Arc::clone(document)
}

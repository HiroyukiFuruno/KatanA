use super::render_tests::{CanvasTest, FIXTURE};
use crate::app::{ActionOps, PreviewOps};
use eframe::egui;

mod coalescing_tests;
mod revision_tests;
mod unchanged_parse_tests;

const PARSE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
const PAN_POINTER_X: f32 = 400.0;
const PAN_POINTER_Y: f32 = 300.0;

struct CanvasRefreshTest;

impl CanvasRefreshTest {
    fn app(path: &std::path::Path) -> crate::shell::KatanaApp {
        let state = crate::app_state::AppState::new(
            katana_core::ai::AiProviderRegistry::new(),
            katana_core::plugin::PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        let mut app = crate::shell::KatanaApp::new(state);
        app.state
            .document
            .open_documents
            .push(katana_core::document::Document::new(
                path.to_path_buf(),
                String::new(),
            ));
        app.state.document.active_doc_idx = Some(0);
        app.full_refresh_preview(path, "", true, 1);
        app
    }

    fn finish(app: &mut crate::shell::KatanaApp, ctx: &egui::Context) {
        let deadline = std::time::Instant::now() + PARSE_TIMEOUT;
        let pane = &mut app.tab_previews[0].pane;
        while pane.canvas_request.is_some() {
            assert!(
                std::time::Instant::now() < deadline,
                "Canvas read did not finish"
            );
            pane.poll_renders(ctx);
            std::thread::yield_now();
        }
    }

    fn pan(app: &mut crate::shell::KatanaApp, ctx: &egui::Context) -> egui::Rect {
        let surface = app.tab_previews[0]
            .pane
            .canvas_surface
            .as_mut()
            .expect("Canvas surface");
        CanvasTest::frame(surface, ctx, vec![]);
        let pointer = egui::pos2(PAN_POINTER_X, PAN_POINTER_Y);
        CanvasTest::frame(surface, ctx, vec![egui::Event::PointerMoved(pointer)]);
        let before = surface.view_rect();
        CanvasTest::frame(
            surface,
            ctx,
            vec![CanvasTest::pointer_button(pointer, true)],
        );
        CanvasTest::frame(
            surface,
            ctx,
            vec![egui::Event::PointerMoved(pointer + egui::vec2(80.0, 30.0))],
        );
        CanvasTest::frame(
            surface,
            ctx,
            vec![CanvasTest::pointer_button(pointer, false)],
        );
        assert_ne!(surface.view_rect(), before, "actual Scene pan");
        surface.view_rect()
    }

    fn draw(app: &mut crate::shell::KatanaApp, ctx: &egui::Context) -> egui::Rect {
        let surface = app.tab_previews[0]
            .pane
            .canvas_surface
            .as_mut()
            .expect("Canvas surface");
        CanvasTest::frame(surface, ctx, vec![]);
        surface.view_rect()
    }
}

#[test]
fn automatic_canvas_reload_retains_the_panned_view_for_unchanged_file_bytes() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("graph.canvas");
    std::fs::write(&path, FIXTURE).expect("Canvas fixture");
    let context = egui::Context::default();
    let mut app = CanvasRefreshTest::app(&path);
    CanvasRefreshTest::finish(&mut app, &context);
    let panned = CanvasRefreshTest::pan(&mut app, &context);
    app.process_action(
        &context,
        crate::app_state::AppAction::RefreshDocument { is_manual: false },
    );
    CanvasRefreshTest::finish(&mut app, &context);
    assert_eq!(CanvasRefreshTest::draw(&mut app, &context), panned);
}

#[test]
fn manual_canvas_reload_can_reset_view_and_automatic_reload_sees_changed_bytes() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("graph.canvas");
    std::fs::write(&path, FIXTURE).expect("Canvas fixture");
    let context = egui::Context::default();
    let mut app = CanvasRefreshTest::app(&path);
    CanvasRefreshTest::finish(&mut app, &context);
    let panned = CanvasRefreshTest::pan(&mut app, &context);
    app.process_action(
        &context,
        crate::app_state::AppAction::RefreshDocument { is_manual: true },
    );
    CanvasRefreshTest::finish(&mut app, &context);
    assert_ne!(CanvasRefreshTest::draw(&mut app, &context), panned);
    std::fs::write(&path, "{invalid").expect("changed Canvas fixture");
    app.process_action(
        &context,
        crate::app_state::AppAction::RefreshDocument { is_manual: false },
    );
    CanvasRefreshTest::finish(&mut app, &context);
    let surface = app.tab_previews[0]
        .pane
        .canvas_surface
        .as_ref()
        .expect("Canvas surface");
    assert!(matches!(
        surface.state,
        super::state::CanvasLoadState::Failed(_)
    ));
}

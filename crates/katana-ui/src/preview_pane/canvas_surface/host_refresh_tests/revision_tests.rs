use eframe::egui;
use katana_core::canvas::{CanvasDocument, NodeContent};

use super::super::{CanvasLoadError, CanvasSurface, state::CanvasLoadState};
use super::{ActionOps, CanvasRefreshTest, CanvasTest};

const ORIGINAL_TEXT: &str = "original canvas revision";
const UPDATED_TEXT: &str = "updated canvas revision";

pub(super) struct RevisionCase;

impl RevisionCase {
    pub(super) fn source(text: &str, x: i64, width: i64) -> String {
        serde_json::json!({"nodes":[{"id":"a","type":"text","x":x,"y":0,"width":width,"height":80,"text":text}]}).to_string()
    }

    pub(super) fn app(
        path: &std::path::Path,
        source: &str,
        ctx: &egui::Context,
    ) -> crate::shell::KatanaApp {
        std::fs::write(path, source).expect("initial Canvas file");
        let mut app = CanvasRefreshTest::app(path);
        CanvasRefreshTest::finish(&mut app, ctx);
        app
    }

    fn surface(app: &crate::shell::KatanaApp) -> &CanvasSurface {
        app.tab_previews[0]
            .pane
            .canvas_surface
            .as_ref()
            .expect("Canvas surface")
    }

    fn ready_model(app: &crate::shell::KatanaApp) -> &CanvasDocument {
        let CanvasLoadState::Ready(document) = &Self::surface(app).state else {
            panic!("Canvas must be Ready");
        };
        document
    }

    fn reload(app: &mut crate::shell::KatanaApp, ctx: &egui::Context, source: &str) {
        std::fs::write(app.state.active_path().expect("active file"), source)
            .expect("updated Canvas file");
        app.process_action(
            ctx,
            crate::app_state::AppAction::RefreshDocument { is_manual: false },
        );
        CanvasRefreshTest::finish(app, ctx);
    }

    fn draw(app: &mut crate::shell::KatanaApp, ctx: &egui::Context) -> egui::FullOutput {
        let surface = app.tab_previews[0]
            .pane
            .canvas_surface
            .as_mut()
            .expect("Canvas surface");
        CanvasTest::frame(surface, ctx, vec![])
    }

    pub(super) fn assert_updated_document(
        app: &crate::shell::KatanaApp,
        text: &str,
        geometry: (f64, f64),
    ) {
        let node = &Self::ready_model(app).nodes[0];
        assert_eq!(node.content, NodeContent::Text { text: text.into() });
        assert_eq!((node.geometry.x, node.geometry.width), geometry);
    }

    pub(super) fn assert_updated_frame(
        app: &mut crate::shell::KatanaApp,
        ctx: &egui::Context,
        visible: &str,
        hidden: &str,
    ) {
        let output = Self::draw(app, ctx);
        let texts = CanvasTest::texts(&output).join("\n");
        assert!(texts.contains(visible));
        assert!(!texts.contains(hidden));
        assert!(app.tab_previews[0].pane.is_all_drawn());
        assert!(!app.tab_previews[0].pane.is_any_pending());
    }

    pub(super) fn assert_failed(app: &mut crate::shell::KatanaApp, ctx: &egui::Context) {
        assert!(matches!(
            &Self::surface(app).state,
            CanvasLoadState::Failed(CanvasLoadError::InvalidCanvas(_))
        ));
        let output = Self::draw(app, ctx);
        assert!(
            CanvasTest::texts(&output)
                .join("\n")
                .contains(CanvasTest::labels().load_error)
        );
    }
}

#[test]
fn automatic_reload_updates_valid_model_viewport_and_real_text_shapes() {
    let directory = tempfile::tempdir().expect("Canvas directory");
    let path = directory.path().join("graph.canvas");
    let context = egui::Context::default();
    let initial = RevisionCase::source(ORIGINAL_TEXT, 0, 120);
    let mut app = RevisionCase::app(&path, &initial, &context);
    let panned = CanvasRefreshTest::pan(&mut app, &context);
    let changed = RevisionCase::source(UPDATED_TEXT, 700, 320);
    RevisionCase::reload(&mut app, &context, &changed);
    RevisionCase::assert_updated_document(&app, UPDATED_TEXT, (700.0, 320.0));
    RevisionCase::assert_updated_frame(&mut app, &context, UPDATED_TEXT, ORIGINAL_TEXT);
    let surface = RevisionCase::surface(&app);
    assert_ne!(surface.view_rect(), panned);
    assert!(surface.view_rect().contains_rect(surface.layout.rects[0]));
}

#[test]
fn failed_reload_recovers_ready_shapes_when_original_bytes_return() {
    let directory = tempfile::tempdir().expect("Canvas directory");
    let path = directory.path().join("graph.canvas");
    let context = egui::Context::default();
    let original = RevisionCase::source(ORIGINAL_TEXT, 0, 120);
    let mut app = RevisionCase::app(&path, &original, &context);
    let revision = RevisionCase::surface(&app)
        .revision
        .expect("source revision");
    RevisionCase::reload(&mut app, &context, "{invalid");
    RevisionCase::assert_failed(&mut app, &context);
    RevisionCase::reload(&mut app, &context, &original);
    RevisionCase::assert_updated_document(&app, ORIGINAL_TEXT, (0.0, 120.0));
    assert_eq!(RevisionCase::surface(&app).revision, Some(revision));
    RevisionCase::assert_updated_frame(
        &mut app,
        &context,
        ORIGINAL_TEXT,
        CanvasTest::labels().load_error,
    );
}

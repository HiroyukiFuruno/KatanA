use super::PreviewPane;

const FIXTURE: &str = include_str!("../../../../assets/fixtures/katana/read-only-preview.canvas");
const PARSE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

struct CanvasHostTest;

impl CanvasHostTest {
    fn finish(pane: &mut PreviewPane, ctx: &egui::Context) {
        let deadline = std::time::Instant::now() + PARSE_TIMEOUT;
        while pane.canvas_request.is_some() {
            assert!(
                std::time::Instant::now() < deadline,
                "canvas parse did not finish"
            );
            pane.poll_renders(ctx);
            std::thread::yield_now();
        }
    }
}

#[test]
fn canvas_source_loads_asynchronously_and_is_not_drawn_until_real_ui_frame() {
    let ctx = egui::Context::default();
    let mut pane = PreviewPane::default();
    pane.update_canvas_source(FIXTURE, std::path::Path::new("graph.canvas"));
    assert!(pane.has_canvas_surface());
    assert!(pane.canvas_request.is_some());
    assert!(pane.is_any_pending());
    assert!(!pane.is_all_drawn());
    assert!(pane.sections.is_empty());
    CanvasHostTest::finish(&mut pane, &ctx);
    assert!(!pane.is_any_pending());
    assert!(!pane.is_all_drawn());
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        pane.show_content(ui, None, None, None, None);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
    assert!(pane.is_all_drawn());
}

#[test]
fn replacement_source_and_abort_drop_stale_canvas_requests() {
    let ctx = egui::Context::default();
    let mut pane = PreviewPane::default();
    pane.update_canvas_source("not-json", std::path::Path::new("old.canvas"));
    let previous_generation = pane.session_generation;
    pane.update_canvas_source(FIXTURE, std::path::Path::new("new.canvas"));
    assert!(pane.session_generation > previous_generation);
    CanvasHostTest::finish(&mut pane, &ctx);
    assert_eq!(pane.md_file_path, std::path::Path::new("new.canvas"));
    assert!(pane.has_canvas_surface());
    pane.abort_renders();
    assert!(pane.canvas_request.is_none());
    assert!(!pane.has_canvas_surface());
}

#[test]
fn markdown_update_discards_canvas_and_preserves_markdown_preview() {
    let mut pane = PreviewPane::default();
    pane.update_canvas_source(FIXTURE, std::path::Path::new("graph.canvas"));
    pane.update_markdown_sections("# Heading\n\ntext", std::path::Path::new("readme.md"));
    assert!(!pane.has_canvas_surface());
    assert!(pane.canvas_request.is_none());
    assert!(!pane.sections.is_empty());
    pane.poll_renders(&egui::Context::default());
    assert!(!pane.has_canvas_surface());
}

#[test]
fn invalid_canvas_displays_a_typed_failure_without_raw_json_preview() {
    let ctx = egui::Context::default();
    let mut pane = PreviewPane::default();
    pane.update_canvas_source("not-json", std::path::Path::new("invalid.canvas"));
    CanvasHostTest::finish(&mut pane, &ctx);
    assert!(!pane.is_loading);
    assert!(pane.has_canvas_surface());
    assert!(pane.sections.is_empty());
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        pane.show(ui);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
    assert!(pane.is_all_drawn());
}

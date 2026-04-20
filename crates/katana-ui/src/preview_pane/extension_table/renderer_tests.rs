use super::*;
use egui::{Context, RawInput, Ui, pos2, vec2};

#[test]
fn test_renderer_sanity() {
    let ctx = Context::default();
    let _ = ctx.run(RawInput::default(), |ctx| {
        let rect = egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 500.0));
        let builder = egui::UiBuilder::new().max_rect(rect);
        let mut ui = Ui::new(ctx.clone(), egui::Id::new("test"), builder);

        let mut render_cell = |_ui: &mut Ui, _cache: &mut CommonMarkCache, _items: &[_]| {};
        let table_data = Table {
            header: vec![],
            rows: vec![],
        };

        let res = KatanaTableRenderer::render(
            &mut ui,
            &mut CommonMarkCache::default(),
            &CommonMarkOptions::default(),
            table_data,
            &[],
            400.0,
            &mut render_cell,
        );
        /* WHY: Verification - Resulting width MUST be within parent constraint. */
        assert!(res.rect.width() <= 400.0 + 1.0); // Allow small floating point epsilon
    });
}

#[test]
fn test_renderer_long_content_no_expand() {
    let ctx = Context::default();
    let _ = ctx.run(RawInput::default(), |ctx| {
        let rect = egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 500.0));
        let builder = egui::UiBuilder::new().max_rect(rect);
        let mut ui = Ui::new(ctx.clone(), egui::Id::new("test_long"), builder);

        let mut render_cell = |ui: &mut Ui, _cache: &mut CommonMarkCache, _items: &[_]| {
            /* Simulate a cell that is very wide (e.g., 2000px). */
            ui.allocate_space(vec2(2000.0, 20.0));
        };

        /* Give it 3 columns so the table wants to be at least 6000px wide. */
        let table_data = Table {
            header: vec![vec![], vec![], vec![]],
            rows: vec![],
        };

        let parent_max_width = 400.0;
        let _res = KatanaTableRenderer::render(
            &mut ui,
            &mut CommonMarkCache::default(),
            &CommonMarkOptions::default(),
            table_data,
            &[],
            parent_max_width,
            &mut render_cell,
        );

        /* WHY: Verification - Resulting UI min_rect MUST be within parent constraint.
        The returned response rect tracks the inner content size, so it CAN be larger.
        We exclusively check the UI's bounding box to ensure layout won't push the splitter. */
        assert!(
            ui.min_rect().width() <= parent_max_width + 1.0,
            "Layout expansion detected in UI! UI min_rect width of {}, max_width was {}",
            ui.min_rect().width(),
            parent_max_width
        );
    });
}

use crate::integration::harness_utils::flatten_clipped_shapes;
use egui_kittest::{Harness, kittest::Queryable};
use katana_core::markdown::color_preset::DiagramColorPreset;
use katana_ui::preview_pane::PreviewPane;

#[test]
fn preset_colors_applied_without_crash_in_harness() {
    /* WHY: Verify that applying syntax highlighting color presets (e.g. for code blocks)
     * doesn't cause crashes during the egui UI frame. */
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        "# Heading\n\nBody text.\n\n```rust\nfn main() {}\n```\n",
        std::path::Path::new("/tmp/test.md"),
    );

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None, None, None);
    });
    harness.run();
}

#[test]
fn markdown_text_uses_center_vertical_alignment_for_mixed_cjk_runs() {
    /* WHY: Verify that for mixed English/CJK (Japanese) paragraphs, the vertical alignment is set to Center
     * to avoid baseline jumping between different fonts. */
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        "KatanA \u{306f} AI\u{30a8}\u{30fc}\u{30b8}\u{30a7}\u{30f3}\u{30c8}\u{3068}\u{5171}\u{306b}\u{4ed5}\u{69d8}\u{99c6}\u{52d5}\u{958b}\u{767a}\u{3092}\u{884c}\u{3046}\u{6642}\u{4ee3}\u{306e}\u{305f}\u{3081}\u{306b}\u{8a2d}\u{8a08}\u{3055}\u{308c}\u{305f}\u{30c4}\u{30fc}\u{30eb}\u{3067}\u{3059}\u{3002}\n",
        std::path::Path::new("/tmp/cjk-baseline.md"),
    );

    let ctx = egui::Context::default();
    katana_ui::font_loader::SystemFontLoader::setup_fonts(
        &ctx,
        DiagramColorPreset::current(),
        None,
        None,
    );
    katana_ui::theme_bridge::ThemeBridgeOps::apply_font_family(&ctx, "Monospace");

    let output = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            pane.show_content(ui, None, None, None, None);
        });
    });

    let flat = flatten_clipped_shapes(&output.shapes);
    let text_shape = flat
        .into_iter()
        .find_map(|s| {
            if let egui::epaint::Shape::Text(text) = s {
                if text
                    .galley
                    .job
                    .text
                    .contains("AI\u{30a8}\u{30fc}\u{30b8}\u{30a7}\u{30f3}\u{30c8}")
                {
                    return Some(text);
                }
            }
            None
        })
        .expect("CJK paragraph not found");

    assert!(
        text_shape
            .galley
            .job
            .sections
            .iter()
            .all(|s| s.format.valign == egui::Align::Center),
        "mixed CJK markdown text should use center baseline alignment"
    );
}

#[test]
fn preview_markdown_uses_proportional_body_font_even_when_ui_font_family_is_monospace() {
    /* WHY: Verify font isolation: the application-wide UI font (e.g. Monospace for editor)
     * should NOT leak into the preview pane, which must always use Proportional for body text. */
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections("# Title", std::path::Path::new("/tmp/test.md"));

    let ctx = egui::Context::default();
    katana_ui::theme_bridge::ThemeBridgeOps::apply_font_family(&ctx, "Monospace");

    let output = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            pane.show_content(ui, None, None, None, None);
        });
    });

    let flat = flatten_clipped_shapes(&output.shapes);
    let text_shape = flat
        .into_iter()
        .find_map(|s| {
            if let egui::epaint::Shape::Text(text) = s {
                if text.galley.job.text.contains("Title") {
                    return Some(text);
                }
            }
            None
        })
        .expect("Title not found");
    assert_eq!(
        text_shape.galley.job.sections[0].format.font_id.family,
        egui::FontFamily::Proportional
    );
}

#[test]
fn underline_tags_render_without_crash() {
    /* WHY: Verify that HTML <u> tags are correctly handled by the Markdown renderer without causing crashes. */
    let mut pane = PreviewPane::default();

    pane.update_markdown_sections(
        "Here is some <u>underlined text</u> in the preview.",
        std::path::Path::new("/tmp/test.md"),
    );

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None, None, None);
    });
    harness.step();
    harness.run();

    let _text_node = harness.get_by_label_contains("underlined text");
}

#[test]
fn multiple_underlines_and_strikethroughs_in_same_block_render_safely() {
    /* WHY: Stability check: Ensure that nested or mixed inline formatting (underline + strikethrough)
     * is correctly processed and rendered. */
    let mut pane = PreviewPane::default();

    pane.update_markdown_sections(
        "A <u>custom underline</u> and a ~~strikethrough~~ mixed.",
        std::path::Path::new("/tmp/test.md"),
    );

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None, None, None);
    });
    harness.step();
    harness.run();

    let _text_block = harness.get_by_label_contains("custom underline");
}

#[test]
fn no_regression_inline_math_with_spaces_must_not_render_as_raw_text() {
    /* WHY: Regression test (Bug 3): Ensure that 'relaxed' inline math (e.g. `$ E = (mc^2) $` with spaces)
     * is correctly captured by the preprocessing pipeline and rendered as math, not exposed as raw text. */
    let mut pane = PreviewPane::default();
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixtures/sample.md");
    let source = std::fs::read_to_string(&path).expect("failed to read sample.md");

    pane.update_markdown_sections(&source, &path);

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None, None, None);
    });
    harness.run_steps(5);

    let raw_nodes: Vec<_> = harness
        .query_all(egui_kittest::kittest::By::default().label_contains("$ E = mc^2 $"))
        .collect();

    assert!(
        raw_nodes.is_empty(),
        "Regression (Bug 3): relaxed inline math `$ E = mc^2 $` is exposed as raw text \
         ({} node(s) with literal dollar signs found).",
        raw_nodes.len(),
    );
}

use eframe::egui;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

const PANEL_WIDTH: f32 = 1000.0;
const PANEL_HEIGHT: f32 = 8000.0;

fn load_sample_sections() -> Vec<RenderedSection> {
    static CACHE: OnceLock<Mutex<Option<Vec<RenderedSection>>>> = OnceLock::new();
    let mutex = CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = mutex.lock().unwrap();
    if guard.is_none() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixtures/sample.md");
        let source = std::fs::read_to_string(&path).expect("failed to read sample.md");
        let mut pane = PreviewPane::default();
        pane.full_render(
            &source,
            &path,
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
            false,
            2,
        );
        pane.wait_for_renders();
        *guard = Some(std::mem::take(&mut pane.sections));
    }
    guard.as_ref().unwrap().clone()
}

fn build_harness(sections: Vec<RenderedSection>) -> Harness<'static> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(PANEL_WIDTH, PANEL_HEIGHT))
        .build_ui(move |ui| {
            let mut pane = PreviewPane::default();
            pane.sections = sections.clone();
            pane.show_content(ui, None, None, None, None);
        });
    for _ in 0..5 {
        harness.step();
    }
    harness
}

/// Bug 1: Diagrams (mermaid/drawio/plantuml) must render as HTML images/error blocks in exported HTML, not raw Markdown code blocks.
///
/// The bug: when evaluating `HtmlExporter::export`, diagram blocks in `sample.ja.md`
/// are somehow bypassed or fail to render, causing them to appear as raw markdown
/// `<pre><code class="language-mermaid">...</code></pre>` in the exported HTML.
///
/// RED condition:
///   The exported HTML must contain the fallback `<div class="katana-diagram-error">`
///   or `<div class="katana-diagram">` blocks for the diagrams, NOT
///   `<code class="language-mermaid">`.
///
/// Bug fixed only when `MarkdownFenceOps::transform_diagram_blocks` correctly replaces
/// the fences with HTML tags before passing to `markdown_to_html`.
#[test]
fn bug1_html_export_must_transform_diagram_blocks_into_html_images_or_errors() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixtures/sample.ja.md");
    let source = std::fs::read_to_string(&path).expect("failed to read sample.ja.md");

    let preset = katana_core::markdown::color_preset::DiagramColorPreset::default();
    let renderer = katana_core::markdown::KatanaRenderer;
    let exported_html =
        katana_core::markdown::HtmlExporter::export(&source, &renderer, &preset, None)
            .expect("Html exporter should succeed");

    /* WHY: If `transform_diagram_blocks` failed, the raw fence remains, and `comrak`
    converts it to `<pre><code class="language-mermaid">`. This is exactly the bug. */
    println!(
        "HTML OUTPUT START=======\n{}\n======HTML OUTPUT END",
        exported_html
    );
    let has_raw_mermaid_block = exported_html.contains("language-mermaid");
    let has_raw_drawio_block = exported_html.contains("language-drawio");
    let has_raw_plantuml_block = exported_html.contains("language-plantuml");

    assert!(
        !has_raw_mermaid_block && !has_raw_drawio_block && !has_raw_plantuml_block,
        "Bug 1 (HTML export diagrams failure):\n\
         Exported HTML contains raw diagram code blocks (e.g., <code class=\"language-mermaid\">)\n\
         instead of transformed image or error divs. `MarkdownFenceOps::transform_diagram_blocks`\n\
         is failing to process diagram fences in sample.ja.md."
    );
}

/// Bug 2: Footnote definitions must appear at the very end of the document.
///
/// The bug: `egui_commonmark` renders sections independently. When the section
/// containing `[^1]` / `[^2]` definitions (§14.4) is rendered, it emits footnote
/// content there. §15 (diagrams) and the final "✅ Verification Complete" follow,
/// so footnote definitions appear MID-document instead of at the true document end.
///
/// RED condition:
///   footnote_def.y0 < last_document_text.y0
///   (footnote appears before the last visible text — i.e. NOT at the end)
///
/// Bug fixed only when footnotes are globally aggregated to the document tail.
#[test]
fn bug2_footnote_definitions_must_be_at_document_end() {
    let harness = build_harness(load_sample_sections());

    /* WHY: "there are no rendering regressions." is the very last text line in sample.md
    (inside "## ✅ Verification Complete"). Footnote definitions MUST come after this. */
    let footnote_def = harness.get_by_label_contains("First footnote content.");
    let last_doc_text = harness.get_by_label_contains("there are no rendering regressions");

    let footnote_bounds = footnote_def
        .accesskit_node()
        .raw_bounds()
        .expect("footnote definition must have layout bounds");
    let last_bounds = last_doc_text
        .accesskit_node()
        .raw_bounds()
        .expect("last document text must have layout bounds");

    assert!(
        footnote_bounds.y0 > last_bounds.y0,
        "Bug 2 (footnote not at end): footnote definition (y0={:.1}) appears before \
         the last document text (y0={:.1}).\n\
         Footnotes must be collected and rendered after ALL sections are complete.",
        footnote_bounds.y0,
        last_bounds.y0,
    );
}

/// Bug 3: Inline math with spaces `$ E = mc^2 $` must NOT appear as raw dollar text.
///
/// Katana's spec allows spaces inside `$...$` delimiters (relaxed math).
/// The bug: `$ E = mc^2 $` renders as raw text exposing the `$` delimiters.
///
/// Note: This was EN-vs-JA discrepancy. EN sample.md was updated to use
/// `$ E = mc^2 $` (with spaces) matching JA, so both should now trigger the bug.
///
/// This test is a REGRESSION GUARD — it must stay GREEN going forward.
/// If it turns RED, something in the math rendering pipeline broke again.
#[test]
fn no_regression_inline_math_with_spaces_must_not_render_as_raw_text() {
    let harness = build_harness(load_sample_sections());

    /* WHY: sample.md L339: `Mass-energy equivalence: $ E = mc^2 $`
    Katana normalizes `$ expr $` → math widget. If the `$` delimiters are visible
    in the accessibility tree, the relaxed-math pass was bypassed or broken. */
    let raw_nodes: Vec<_> = harness
        .query_all(egui_kittest::kittest::By::default().label_contains("$ E = mc^2 $"))
        .collect();

    assert!(
        raw_nodes.is_empty(),
        "Regression (Bug 3): relaxed inline math `$ E = mc^2 $` is exposed as raw text \
         ({} node(s) with literal dollar signs found).\n\
         The math pre-processing pipeline was likely broken by a recent change.",
        raw_nodes.len(),
    );
}

/// Bug 4: Icons -> Advanced Settings panel must render its table content when open.
///
/// The bug: the advanced settings bottom panel shows its heading ("Advanced Settings")
/// and close button, but the icon table (rows with icon names, vendor, color, etc.)
/// is missing — the panel body is visually empty.
///
/// RED condition:
///   After rendering with `is_open=true`, the icon table header "Icon" must be present
///   in the accessibility tree. If it is absent, the table is not being drawn.
///
/// Note: The egui `TopBottomPanel` used here is a panel-scoped widget. Injecting
/// `is_open=true` via `ui.data_mut` must happen in the same frame BEFORE the
/// `render_icons_tab_for_test` call reads it.
#[test]
fn bug4_icons_advanced_settings_panel_must_show_table_content() {
    use katana_core::ai::AiProviderRegistry;
    use katana_core::plugin::PluginRegistry;
    use katana_ui::app_state::AppState;
    use katana_ui::settings::tabs::icons::IconsTabOps;

    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );

    let mut harness = Harness::builder()
        .with_size(egui::vec2(900.0, 700.0))
        .build_ui(move |ui| {
            /* WHY: The `render_icons_tab` reads `icons_advanced_is_open` from egui temp
            storage at the START of each call. We inject `true` in the same frame closure
            so the read on the same frame sees the value. */
            ui.data_mut(|d| {
                d.insert_temp(egui::Id::new("icons_advanced_is_open"), true);
            });
            IconsTabOps::render_icons_tab_for_test(ui, &mut state);
        });
    for _ in 0..5 {
        harness.step();
    }

    /* WHY: When the panel is open, `panels.rs:56` calls `table::IconsTableOps::render`
    which emits column headers. en.json: `settings.icons.table_header_icon = "Icon"`.
    The heading "Advanced Settings" also uses the key `settings.icons.advanced_settings`.
    We verify the heading is present first (panel is open), then the table header. */
    let _heading = harness.get_by_label_contains("Advanced Settings");

    /* WHY: "Icon" appears ONLY in the table header when the panel is open.
    If the table does not render, no such node exists → test fails (RED). */
    let icon_col_headers: Vec<_> = harness
        .query_all(egui_kittest::kittest::By::default().label("Icon"))
        .collect();

    assert!(
        !icon_col_headers.is_empty(),
        "Bug 4 (Icons advanced panel empty): icon table column header not found.\n\
         The 'Advanced Settings' panel is open but `IconsTableOps::render` \
         output is absent from the accessibility tree.\n\
         Expected at least one node matching the table header icon label.",
    );
}

#[test]
fn bug5_footnote_references_must_not_render_as_raw_text() {
    let harness = build_harness(load_sample_sections());

    // In sample.ja.md, the text contains a footnote reference: "[^1]".
    // If the footnote reference is broken, the literal "[^1]" will be exposed.
    let raw_nodes: Vec<_> = harness
        .query_all(egui_kittest::kittest::By::default().label_contains("[^1]"))
        .collect();

    assert!(
        raw_nodes.is_empty(),
        "Regression (Bug 5): Footnote reference '[^1]' is exposed as literal raw text \
         ({} nodes found). \
         Footnote references must be rendered as clickable footnote links, not raw text.",
        raw_nodes.len()
    );
}

#[test]
fn bug6_footnote_bidirectional_links_exist() {
    let harness = build_harness(load_sample_sections());

    // We should be able to find the footnote link in the main text.
    // In egui_commonmark, a footnote link typically is rendered as a hoverable/clickable element.
    // The exact text rendered for footnote 1 is "1".
    //
    // And there should be a return link "↩" rendered in the footnote definition.
    let return_links: Vec<_> = harness
        .query_all(egui_kittest::kittest::By::default().label_contains("↩"))
        .collect();

    assert!(
        !return_links.is_empty(),
        "Regression (Bug 6): No return links ('↩') found for footnotes. \
        Bidirectional linking from footnote definition strictly requires these to exist."
    );
}

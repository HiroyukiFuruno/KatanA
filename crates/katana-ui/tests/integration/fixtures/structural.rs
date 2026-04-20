use super::harness::load_fixture;
use katana_ui::preview_pane::RenderedSection;

#[test]
fn fixture_en_produces_many_sections() {
    /* WHY: Verify that the standard English fixture template is correctly parsed into
     * numerous sections, ensuring broad coverage of markdown features. */
    let (pane, _, _) = load_fixture("sample.md");
    assert!(pane.sections.len() > 25);
}

#[test]
fn fixture_en_no_pending_after_render() {
    /* WHY: Ensure that all asynchronous rendering tasks (Diagrams, Math, etc.)
     * have completed and been collected into the final sections list. */
    let (pane, _, _) = load_fixture("sample.md");
    let pending = pane
        .sections
        .iter()
        .filter(|s| matches!(s, RenderedSection::Pending { .. }))
        .count();
    assert_eq!(pending, 0);
}

#[test]
fn fixture_ja_structural_integrity() {
    /* WHY: Double check that the Japanese fixture template is also correctly parsed
     * without leakage or pending sections. */
    let (pane, _, _) = load_fixture("sample.ja.md");
    assert!(pane.sections.len() > 25);
    let pending = pane
        .sections
        .iter()
        .filter(|s| matches!(s, RenderedSection::Pending { .. }))
        .count();
    assert_eq!(pending, 0);
}

#[test]
fn fixture_en_drawio_always_renders_to_image() {
    /* WHY: Verify that Draw.io blocks are correctly detected and mapped to Image sections,
     * regardless of whether other complex diagrams fail. */
    let (pane, _, _) = load_fixture("sample.md");
    let count = pane
        .sections
        .iter()
        .filter(|s| matches!(s, RenderedSection::Image { alt, .. } if alt.contains("DrawIo")))
        .count();
    assert!(count >= 2);
}

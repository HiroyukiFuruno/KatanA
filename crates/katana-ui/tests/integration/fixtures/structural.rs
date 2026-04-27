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
fn fixture_en_drawio_blocks_resolve_to_terminal_sections() {
    /* WHY: Draw.io blocks require a downloaded viewer asset in fresh environments.
     * The fixture must still prove they leave Pending state and become terminal sections. */
    let (pane, _, _) = load_fixture("sample.md");
    let count = pane
        .sections
        .iter()
        .filter(|s| match s {
            RenderedSection::Image { alt, .. } if alt.contains("DrawIo") => true,
            RenderedSection::NotInstalled { kind, .. } if kind == "Draw.io" => true,
            RenderedSection::Error { message, .. } => {
                message.contains("Could not auto detect a chrome executable")
                    || message.contains("Cannot find browser")
                    || message.contains("Failed to launch browser")
            }
            _ => false,
        })
        .count();
    assert!(count >= 2);
}

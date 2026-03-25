use katana_core::markdown::*;

#[test]
fn basic_underline_renders_to_html_or_passes_through() {
    let md = "Here is an <u>underlined</u> word.";
    let out = render_basic(md).expect("render failed");

    // pulldown_cmark passes HTML tags through when enabled.
    // The UI layer (egui_commonmark) parses '<u>' specially.
    // At the core layer, we simply verify it wasn't stripped out.
    assert!(out.html.contains("<u>underlined</u>"));
}

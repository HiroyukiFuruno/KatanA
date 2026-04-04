use katana_core::markdown::*;

#[test]
fn basic_underline_renders_to_html_or_passes_through() {
    let md = "Here is an <u>underlined</u> word.";
    let out = MarkdownRenderOps::render_basic(md).expect("render failed");

    assert!(out.html.contains("<u>underlined</u>"));
}

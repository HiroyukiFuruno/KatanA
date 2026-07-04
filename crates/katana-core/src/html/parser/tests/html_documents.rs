use crate::html::HtmlNode;

use super::parser;

#[test]
fn parses_html_document_body_without_head_content() {
    let html = r#"<!doctype html>
<html>
  <head>
    <title>Hidden title</title>
    <style>body { color: red; }</style>
    <script>alert("x")</script>
  </head>
  <body>
    <main>
      <h1>Visible title</h1>
      <p>Visible paragraph</p>
    </main>
  </body>
</html>"#;

    let nodes = parser().parse(html);

    assert!(matches!(
        &nodes[..],
        [HtmlNode::Heading { .. }, HtmlNode::Paragraph { .. },]
    ));
    assert!(!HtmlNode::render_to_html(&nodes).contains("Hidden title"));
}

#[test]
fn parses_details_with_summary_and_body() {
    let nodes = parser()
        .parse(r#"<details><summary>More</summary><p>Details <strong>body</strong></p></details>"#);

    let [HtmlNode::Details { summary, children }] = &nodes[..] else {
        panic!("expected one details node: {nodes:?}");
    };

    assert_eq!(HtmlNode::render_to_html(summary), "More");
    assert_eq!(
        HtmlNode::render_to_html(children),
        "<p>Details <strong>body</strong></p>"
    );
}

#[test]
fn parses_details_without_summary() {
    let nodes = parser().parse(r#"<details><p>Body only</p></details>"#);

    let [HtmlNode::Details { summary, children }] = &nodes[..] else {
        panic!("expected one details node: {nodes:?}");
    };

    assert!(summary.is_empty());
    assert_eq!(HtmlNode::render_to_html(children), "<p>Body only</p>");
}

#[test]
fn parses_table_headers_and_rows() {
    let nodes = parser().parse(
        r#"<table>
  <thead><tr><th>Name</th><th>State</th></tr></thead>
  <tbody><tr><td>KatanA</td><td><a href="docs.html">Ready</a></td></tr></tbody>
</table>"#,
    );

    let [HtmlNode::Table { headers, rows }] = &nodes[..] else {
        panic!("expected one table node: {nodes:?}");
    };

    assert_eq!(headers.len(), 2);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].len(), 2);
    assert_eq!(HtmlNode::render_to_html(&headers[0]), "Name");
    assert_eq!(HtmlNode::render_to_html(&rows[0][0]), "KatanA");
    assert!(HtmlNode::render_to_html(&rows[0][1]).contains("<a href="));
}

#[test]
fn parses_header_row_with_data_cells() {
    let nodes = parser().parse(r#"<table><tr><th>Name</th><td>KatanA</td></tr></table>"#);

    let [HtmlNode::Table { headers, rows }] = &nodes[..] else {
        panic!("expected one table node: {nodes:?}");
    };

    assert_eq!(headers.len(), 1);
    assert_eq!(rows.len(), 1);
    assert_eq!(HtmlNode::render_to_html(&headers[0]), "Name");
    assert_eq!(HtmlNode::render_to_html(&rows[0][0]), "KatanA");
}

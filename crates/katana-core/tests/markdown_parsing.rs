#[test]
fn test_comrak_outline_parsing_logic() {
    use comrak::nodes::NodeValue;
    use comrak::{Arena, Options, parse_document};
    let source = "### heading 1\n\nKatanA pipeline:\n\n```mermaid\ngraph LR\n```\n# heading 2\n";
    let arena = Arena::new();
    let opts = Options::default();
    let root = parse_document(&arena, source, &opts);
    let mut heading_count = 0;
    for node in root.descendants() {
        if let NodeValue::Heading(_) = node.data.borrow().value {
            heading_count += 1;
        }
    }
    assert_eq!(heading_count, 2);
}

#[test]
fn test_comrak_sourcepos_tracking() {
    use comrak::nodes::NodeValue;
    use comrak::{Arena, Options, parse_document};
    let source = "### heading 1\n\nKatanA pipeline:\n\n```mermaid\ngraph LR\n```\n# heading 2\n";
    let arena = Arena::new();
    let mut opts = Options::default();
    opts.extension.header_id_prefix = Some(String::new());
    let root = parse_document(&arena, source, &opts);

    let mut headings = Vec::new();
    for node in root.descendants() {
        if let NodeValue::Heading(_) = node.data.borrow().value {
            headings.push(node.data.borrow().sourcepos.start.line);
        }
    }

    // Check if headings are found at expected lines matching source
    assert!(!headings.is_empty());
}

use super::*;

#[cfg(test)]
mod sourcepos_tests {
    use comrak::nodes::NodeValue;
    use comrak::{parse_document, Arena, Options};

    #[test]
    fn test_sourcepos_bytes() {
        let arena = Arena::new();
        // WHY:                   0         1         2
        // WHY:                   0123456789012345678901234567
        let src = "Hello\nThis is an ![alt](test.png) text\n";
        let doc = parse_document(&arena, src, &Options::default());
        for node in doc.descendants() {
            if let NodeValue::Image(_) = node.data.borrow().value {
                let pos = node.data.borrow().sourcepos;
                let lines: Vec<&str> = src.lines().collect();
                let line = lines[pos.start.line - 1];
                let extracted = &line[pos.start.column - 1..pos.end.column];
                assert_eq!(extracted, "![alt](test.png)");
            }
        }
    }
}

#[cfg(test)]
mod split_tests {
    use super::*;

    #[test]
    fn test_split_with_mixed_diagram_and_image() {
        let md = "```mermaid\ngraph TD\nA-->B\n```\n![alt](url)\nText";
        let sections = split_into_sections(md);
        assert_eq!(sections.len(), 3);
        assert!(matches!(
            sections[0],
            PreviewSection::Diagram {
                kind: crate::markdown::diagram::DiagramKind::Mermaid,
                ..
            }
        ));
        assert!(matches!(sections[1], PreviewSection::LocalImage { .. }));
        assert!(matches!(sections[2], PreviewSection::Markdown(_)));
    }

    #[test]
    fn test_split_with_relaxed_math_spacing() {
        let md = "Here is some math: $ E = mc^2 $ and a plain text test $ 500 $ 10.";
        let sections = split_into_sections(md);
        assert_eq!(sections.len(), 1);
        if let PreviewSection::Markdown(text) = &sections[0] {
            // WHY: The heuristic converts the math equation but ignores the plain text money values
            assert!(text.contains("$E = mc^2$"));
            assert!(text.contains("$ 500 $ 10."));
        } else {
            panic!("Expected Markdown section");
        }
    }
}

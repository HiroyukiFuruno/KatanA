use super::*;

#[cfg(test)]
mod sourcepos_tests {
    use comrak::nodes::NodeValue;
    use comrak::{Arena, Options, parse_document};

    #[test]
    fn test_sourcepos_bytes() {
        let arena = Arena::new();
        /* WHY: 0         1         2 */
        /* WHY: 0123456789012345678901234567 */
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
        let sections = PreviewSectionOps::split_into_sections(md);
        assert_eq!(sections.len(), 2);
        assert!(matches!(
            sections[0],
            PreviewSection::Diagram {
                kind: crate::markdown::diagram::DiagramKind::Mermaid,
                ..
            }
        ));
        /* WHY: Without a blank line, "![alt](url)\nText" is one paragraph
        so it cannot be split into LocalImage + Markdown. */
        assert!(matches!(sections[1], PreviewSection::Markdown(_, _)));
    }

    #[test]
    fn test_split_with_relaxed_math_spacing() {
        let md = "Here is some math: $ E = mc^2 $ and a plain text test $ 500 $ 10.";
        let sections = PreviewSectionOps::split_into_sections(md);
        assert_eq!(sections.len(), 1);
        if let PreviewSection::Markdown(text, _) = &sections[0] {
            /* WHY: split_sections does not process math — text is passed as-is to downstream rendering. */
            assert!(text.contains("$ E = mc^2 $"));
            assert!(text.contains("$ 500 $ 10."));
        } else {
            panic!("Expected Markdown section");
        }
    }

    #[test]
    fn nested_mermaid_inside_markdown_fence_is_not_rendered_as_diagram() {
        /* WHY: A ```markdown fence wrapping a ```mermaid block must not cause the inner
         * mermaid to be extracted as a Diagram section. The outer fence is not a recognized
         * diagram kind, so the entire outer fence (opening + body + closing) must be consumed
         * as plain Markdown text. */
        let md = "Before\n\
                  ```markdown\n\
                  ```mermaid\n\
                  graph TD\n\
                      A[Start] --> B[End]\n\
                  ```\n\
                  ```\n\
                  After";
        let sections = PreviewSectionOps::split_into_sections(md);
        /* The whole document is one Markdown section — no Diagram sections. */
        assert_eq!(
            sections.len(),
            1,
            "Expected 1 Markdown section, got {}: {:?}",
            sections.len(),
            sections
                .iter()
                .map(|s| match s {
                    PreviewSection::Markdown(_, _) => "Markdown",
                    PreviewSection::Diagram { .. } => "Diagram",
                    PreviewSection::LocalImage { .. } => "LocalImage",
                })
                .collect::<Vec<_>>()
        );
        assert!(matches!(sections[0], PreviewSection::Markdown(_, _)));
    }

    #[test]
    fn table_followed_by_alert_stays_in_same_markdown_section() {
        /* WHY: A table immediately followed by a blockquote alert must remain in the same
         * Markdown section so that the rendering pipeline sees both elements together.
         * No diagram fences are involved; the entire document should be one Markdown section. */
        let md = "| A | B |\n\
                  |---|---|\n\
                  | 1 | 2 |\n\
                  \n\
                  > [!TIP]\n\
                  > This is a tip.";
        let sections = PreviewSectionOps::split_into_sections(md);
        assert_eq!(
            sections.len(),
            1,
            "Expected 1 Markdown section, got {}",
            sections.len()
        );
        assert!(matches!(sections[0], PreviewSection::Markdown(_, _)));
        if let PreviewSection::Markdown(text, _) = &sections[0] {
            assert!(text.contains("| A | B |"), "Table header missing");
            assert!(text.contains("> [!TIP]"), "Alert header missing");
        }
    }

    #[test]
    fn nested_mermaid_inside_four_backtick_fence_is_not_rendered_as_diagram() {
        /* WHY: CommonMark allows nesting a 3-backtick code block inside a 4-backtick fence.
         * The outer ````markdown fence must be consumed in full, so the inner ```mermaid
         * is never extracted as a Diagram section. */
        let md = "Text before\n\
                  ````markdown\n\
                  ```mermaid\n\
                  graph TD\n\
                      A --> B\n\
                  ```\n\
                  ````\n\
                  Text after";
        let sections = PreviewSectionOps::split_into_sections(md);
        assert_eq!(
            sections.len(),
            1,
            "Expected 1 Markdown section (4-backtick outer fence), got {}: {:?}",
            sections.len(),
            sections
                .iter()
                .map(|s| match s {
                    PreviewSection::Markdown(_, _) => "Markdown",
                    PreviewSection::Diagram { .. } => "Diagram",
                    PreviewSection::LocalImage { .. } => "LocalImage",
                })
                .collect::<Vec<_>>()
        );
        assert!(matches!(sections[0], PreviewSection::Markdown(_, _)));
    }

    #[test]
    fn real_mermaid_after_non_diagram_fence_is_still_rendered() {
        /* WHY: After consuming a non-diagram fence, the section splitter must continue
         * and correctly extract a subsequent standalone ```mermaid block. */
        let md = "````markdown\n\
                  ```mermaid\n\
                  graph TD\n\
                      A --> B\n\
                  ```\n\
                  ````\n\
                  \n\
                  ```mermaid\n\
                  graph TD\n\
                      C --> D\n\
                  ```";
        let sections = PreviewSectionOps::split_into_sections(md);
        let kinds: Vec<_> = sections
            .iter()
            .map(|s| match s {
                PreviewSection::Markdown(_, _) => "Markdown",
                PreviewSection::Diagram { .. } => "Diagram",
                PreviewSection::LocalImage { .. } => "LocalImage",
            })
            .collect();
        assert!(
            kinds.contains(&"Diagram"),
            "Expected at least one Diagram section after the escaped fence, got: {:?}",
            kinds
        );
    }
}

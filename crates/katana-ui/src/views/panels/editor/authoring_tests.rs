/* =========================================================================
Unit tests
========================================================================= */
#[cfg(test)]
mod tests {
    use super::*;

    fn apply(buffer: &str, sel: (usize, usize), op: MarkdownAuthoringOp) -> AuthoringTransform {
        MarkdownAuthoringOps::apply(buffer, sel.0, sel.1, op)
    }

    /* --- Bold --- */
    #[test]
    fn bold_no_selection_inserts_placeholder() {
        let t = apply("hello world", (5, 5), MarkdownAuthoringOp::Bold);
        assert_eq!(t.buffer, "hello**text** world");
        assert_eq!(&t.buffer[t.cursor_start..t.cursor_end], "text");
    }

    #[test]
    fn bold_with_selection_wraps() {
        let t = apply("hello world", (6, 11), MarkdownAuthoringOp::Bold);
        assert_eq!(t.buffer, "hello **world**");
        assert_eq!(&t.buffer[t.cursor_start..t.cursor_end], "world");
    }

    /* --- Italic --- */
    #[test]
    fn italic_wraps_selection() {
        let t = apply("hello world", (6, 11), MarkdownAuthoringOp::Italic);
        assert_eq!(t.buffer, "hello *world*");
    }

    /* --- Heading1 --- */
    #[test]
    fn heading1_prefixes_current_line() {
        let t = apply("line1\nline2\nline3", (6, 6), MarkdownAuthoringOp::Heading1);
        assert_eq!(t.buffer, "line1\n# line2\nline3");
    }

    /* --- BulletList --- */
    #[test]
    fn bullet_list_single_line_no_selection() {
        let t = apply("apple\nbanana", (0, 0), MarkdownAuthoringOp::BulletList);
        assert!(t.buffer.starts_with("- apple"));
    }

    /* --- CodeBlock --- */
    #[test]
    fn code_block_wraps_selection() {
        let t = apply(
            "hello world",
            (6, 11),
            MarkdownAuthoringOp::CodeBlock(CodeBlockKind::Text),
        );
        assert_eq!(t.buffer, "hello ```text\nworld\n```");
    }

    #[test]
    fn code_block_language_wraps_selection() {
        let transform = apply(
            "graph TD\nA-->B",
            (0, 14),
            MarkdownAuthoringOp::CodeBlock(CodeBlockKind::Mermaid),
        );

        assert_eq!(transform.buffer, "```mermaid\ngraph TD\nA-->B\n```");
    }

    #[test]
    fn code_block_language_options_include_required_entries() {
        let entries = CodeBlockKind::all()
            .iter()
            .map(|kind| kind.info_string())
            .collect::<Vec<_>>();

        assert!(entries.contains(&"text"));
        assert!(entries.contains(&"markdown"));
        assert!(entries.contains(&"bash"));
        assert!(entries.contains(&"zsh"));
        assert!(entries.contains(&"mermaid"));
        assert!(entries.contains(&"drawio"));
        assert!(entries.contains(&"plantuml"));
        assert!(entries.contains(&"rust"));
        assert!(entries.contains(&"typescript"));
    }

    #[test]
    fn code_block_kind_display_label_matches_inserted_info_string() {
        for kind in CodeBlockKind::all() {
            assert_eq!(kind.display_label(), kind.info_string());
        }
    }

    /* --- HorizontalRule --- */
    #[test]
    fn horizontal_rule_inserts_at_cursor() {
        let t = apply("hello", (5, 5), MarkdownAuthoringOp::HorizontalRule);
        assert_eq!(t.buffer, "hello\n---\n");
    }

    /* --- InsertLink with selection --- */
    #[test]
    fn insert_link_wraps_selection_as_text() {
        let t = apply("click here", (6, 10), MarkdownAuthoringOp::InsertLink);
        assert_eq!(t.buffer, "click [here](url)");
    }

    /* --- InsertTable --- */
    #[test]
    fn insert_table_appends_template() {
        let t = apply("", (0, 0), MarkdownAuthoringOp::InsertTable);
        assert!(t.buffer.contains("| Header 1 |"));
    }

    /* --- Buffer integrity: authorMarkdown never modifies text outside selection --- */
    #[test]
    fn bold_preserves_buffer_prefix_and_suffix() {
        let buf = "BEFORE hello AFTER";
        let t = apply(buf, (7, 12), MarkdownAuthoringOp::Bold);
        assert!(t.buffer.starts_with("BEFORE "));
        assert!(t.buffer.ends_with(" AFTER"));
    }
}

/* WHY: Unit tests for document search (Find) match calculation and navigation.
 * These tests serve as quality gates to prevent regressions in:
 * - Match index computation (character-based, not byte-based)
 * - Next/Prev navigation including wrap-around
 * - Edge cases (empty query, single match, multibyte, etc.)
 */

#[cfg(test)]
mod tests {
    use super::*;

    /* --- Match calculation tests --- */

    #[test]
    fn empty_query_produces_no_matches() {
        let matches = DocSearchOps::compute_matches("", "hello world");
        assert!(matches.is_empty());
    }

    #[test]
    fn simple_match_returns_correct_char_range() {
        let matches = DocSearchOps::compute_matches("world", "hello world");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], 6..11);
    }

    #[test]
    fn multiple_matches_found() {
        let matches = DocSearchOps::compute_matches("a", "abracadabra");
        assert_eq!(matches.len(), 5);
    }

    #[test]
    fn case_insensitive_matching() {
        let matches = DocSearchOps::compute_matches("hello", "Hello HELLO hello");
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn multibyte_char_offsets_are_char_based() {
        /* WHY: "\u{3042}\u{3044}\u{3046}" is 3 chars (9 bytes). "Search" at char index 3, NOT byte index 9. */
        let matches =
            DocSearchOps::compute_matches("Search", "\u{3042}\u{3044}\u{3046}Search");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], 3..9);
    }

    #[test]
    fn search_finds_matches_in_headings() {
        let buffer = "# Hello KatanA";
        let matches = DocSearchOps::compute_matches("KatanA", buffer);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn match_at_end_of_content() {
        let matches = DocSearchOps::compute_matches("end", "the end");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], 4..7);
    }

    #[test]
    fn match_at_start_of_content() {
        let matches = DocSearchOps::compute_matches("start", "start here");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], 0..5);
    }

    #[test]
    fn no_match_returns_empty() {
        let matches = DocSearchOps::compute_matches("xyz", "hello world");
        assert!(matches.is_empty());
    }

    /* --- Navigation: next --- */

    #[test]
    fn next_increments_active_index() {
        let matches = DocSearchOps::compute_matches("a", "a b a b a");
        assert_eq!(matches.len(), 3);

        let result = DocSearchOps::navigate_next(&matches, 0, "a b a b a").unwrap();
        assert_eq!(result.new_active_index, 1);

        let result = DocSearchOps::navigate_next(&matches, 1, "a b a b a").unwrap();
        assert_eq!(result.new_active_index, 2);
    }

    #[test]
    fn next_wraps_around_to_first() {
        let matches = DocSearchOps::compute_matches("x", "x y x");
        assert_eq!(matches.len(), 2);

        let result = DocSearchOps::navigate_next(&matches, 1, "x y x").unwrap();
        assert_eq!(result.new_active_index, 0);
        assert!(
            result.scroll_to_line.is_some(),
            "Wrap-around must set scroll target"
        );
    }

    #[test]
    fn next_on_empty_matches_is_none() {
        let result = DocSearchOps::navigate_next(&[], 0, "");
        assert!(result.is_none());
    }

    #[test]
    fn next_single_match_wraps_to_self() {
        let matches = DocSearchOps::compute_matches("x", "x");
        let result = DocSearchOps::navigate_next(&matches, 0, "x").unwrap();
        assert_eq!(result.new_active_index, 0);
        assert_eq!(result.scroll_to_line, Some(0));
    }

    /* --- Navigation: prev --- */

    #[test]
    fn prev_wraps_around_to_last() {
        let matches = DocSearchOps::compute_matches("x", "x y x");
        assert_eq!(matches.len(), 2);

        let result = DocSearchOps::navigate_prev(&matches, 0, "x y x").unwrap();
        assert_eq!(result.new_active_index, 1);
        assert!(
            result.scroll_to_line.is_some(),
            "Wrap-around must set scroll target"
        );
    }

    #[test]
    fn prev_decrements_active_index() {
        let matches = DocSearchOps::compute_matches("a", "a b a b a");

        let result = DocSearchOps::navigate_prev(&matches, 2, "a b a b a").unwrap();
        assert_eq!(result.new_active_index, 1);

        let result = DocSearchOps::navigate_prev(&matches, 1, "a b a b a").unwrap();
        assert_eq!(result.new_active_index, 0);
    }

    #[test]
    fn prev_on_empty_matches_is_none() {
        let result = DocSearchOps::navigate_prev(&[], 0, "");
        assert!(result.is_none());
    }

    /* --- Navigation: scroll target line --- */

    #[test]
    fn scroll_target_points_to_correct_line() {
        let buffer = "line0\nmatch on line1\nline2\nmatch on line3";
        let matches = DocSearchOps::compute_matches("match", buffer);
        assert_eq!(matches.len(), 2);

        /* WHY: First match is on line 1 */
        let result = DocSearchOps::navigate_next(&matches, 0, buffer).unwrap();
        assert_eq!(result.scroll_to_line, Some(3), "Second match on line 3");

        /* WHY: Prev from first should wrap to last (line 3) */
        let result = DocSearchOps::navigate_prev(&matches, 0, buffer).unwrap();
        assert_eq!(result.scroll_to_line, Some(3), "Last match on line 3");
    }

    #[test]
    fn scroll_target_for_first_line() {
        let buffer = "match here\nline1\nline2";
        let matches = DocSearchOps::compute_matches("match", buffer);

        let result = DocSearchOps::navigate_prev(&matches, 0, buffer).unwrap();
        assert_eq!(result.new_active_index, 0);
        assert_eq!(result.scroll_to_line, Some(0), "First line");
    }

    /* --- Markdown Awareness Tests --- */

    #[test]
    fn search_skips_link_urls() {
        let buffer = "[KatanA](https://github.com/KatanA/KatanA)";
        /* WHY: Should find only 1 "KatanA" (in the label), not 3 (label + 2 in URL). */
        let matches = DocSearchOps::compute_matches("KatanA", buffer);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], 1..7);  /* WHY: "KatanA" inside [] */
    }

    #[test]
    fn search_skips_image_src() {
        let buffer = "![KatanA Logo](katana.png)";
        /* WHY: Should find "KatanA" in alt text, but skip it in filename if it matched. */
        let matches = DocSearchOps::compute_matches("KatanA", buffer);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn search_finds_matches_in_code_blocks() {
        let buffer = "```rust\nlet x = \"katana\";\n```";
        let matches = DocSearchOps::compute_matches("katana", buffer);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn search_finds_matches_in_list_items() {
        let buffer = "- Item with katana\n- Another katana";
        let matches = DocSearchOps::compute_matches("katana", buffer);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn search_skips_html_attributes() {
        let buffer = "<span class=\"katana\">some text</span>";
        /* WHY: Should find nothing if we skip Event::Html and "katana" is only in class. */
        let matches = DocSearchOps::compute_matches("katana", buffer);
        assert_eq!(matches.len(), 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_context_suffix_removes_editor_tag() {
        assert_eq!(strip_context_suffix("primary+B[editor]"), "primary+B");
    }

    #[test]
    fn strip_context_suffix_leaves_plain_shortcut_unchanged() {
        assert_eq!(strip_context_suffix("primary+S"), "primary+S");
    }

    #[test]
    fn parse_shortcut_handles_backtick() {
        let result = ShortcutKeyOps::parse_shortcut("primary+`");
        assert!(
            result.is_some(),
            "backtick shortcut must parse successfully"
        );
    }

    #[test]
    fn parse_shortcut_ignores_unknown_key() {
        let result = ShortcutKeyOps::parse_shortcut("primary+@@@");
        assert!(result.is_none());
    }

    #[test]
    fn editor_keeps_primary_v_for_text_or_image_paste() {
        let shortcut = ShortcutKeyOps::parse_shortcut("primary+V").expect("primary+V must parse");
        assert!(ShortcutKeyOps::editor_keeps_shortcut(&shortcut));
    }

    #[test]
    fn editor_keeps_shifted_primary_b_for_native_text_entry() {
        let shortcut =
            ShortcutKeyOps::parse_shortcut("primary+shift+B").expect("primary+shift+B must parse");
        assert!(ShortcutKeyOps::editor_keeps_shortcut(&shortcut));
    }

    #[test]
    fn editor_does_not_keep_primary_s() {
        let shortcut = ShortcutKeyOps::parse_shortcut("primary+S").expect("primary+S must parse");
        assert!(!ShortcutKeyOps::editor_keeps_shortcut(&shortcut));
    }

    #[test]
    fn editor_keeps_arrow_navigation() {
        let shortcut = ShortcutKeyOps::parse_shortcut("shift+left").expect("shift+left must parse");
        assert!(ShortcutKeyOps::editor_keeps_shortcut(&shortcut));
    }
}

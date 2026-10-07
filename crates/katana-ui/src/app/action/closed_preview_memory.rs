#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ClosedPreviewTransition {
    pub removed_document_preview: bool,
    pub open_documents_empty: bool,
    pub previews_empty: bool,
}

pub(super) fn should_relieve_memory(transition: ClosedPreviewTransition) -> bool {
    transition.removed_document_preview
        && transition.open_documents_empty
        && transition.previews_empty
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transition(
        removed: bool,
        open_empty: bool,
        previews_empty: bool,
    ) -> ClosedPreviewTransition {
        ClosedPreviewTransition {
            removed_document_preview: removed,
            open_documents_empty: open_empty,
            previews_empty,
        }
    }

    #[test]
    fn only_last_document_transition_reliefs_memory() {
        for (removed, open_empty, previews_empty, expected) in [
            (false, false, false, false),
            (false, false, true, false),
            (false, true, false, false),
            (false, true, true, false),
            (true, false, false, false),
            (true, false, true, false),
            (true, true, false, false),
            (true, true, true, true),
        ] {
            assert_eq!(
                should_relieve_memory(transition(removed, open_empty, previews_empty)),
                expected
            );
        }
    }

    #[test]
    fn repeated_empty_cleanup_has_no_second_transition() {
        let first = transition(true, true, true);
        let second = transition(false, true, true);
        assert!(should_relieve_memory(first));
        assert!(!should_relieve_memory(second));
    }

    #[test]
    fn html_and_markdown_are_not_document_memory_candidates() {
        assert!(!katana_core::workspace::TreeEntry::path_is_document(
            std::path::Path::new("preview.html")
        ));
        assert!(!katana_core::workspace::TreeEntry::path_is_document(
            std::path::Path::new("notes.md")
        ));
        assert!(katana_core::workspace::TreeEntry::path_is_document(
            std::path::Path::new("report.docx")
        ));
    }
}

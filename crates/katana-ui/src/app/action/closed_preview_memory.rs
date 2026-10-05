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

#[cfg(target_os = "macos")]
mod macos {
    use std::ffi::c_void;

    unsafe extern "C" {
        fn malloc_zone_pressure_relief(zone: *mut c_void, goal: usize) -> usize;
    }

    pub(super) fn relieve() -> usize {
        /* SAFETY: NULLで登録済みzone全体を対象にし、goal 0で最大限の返却候補を要求する。
         * ポインタを保持せず、呼出し中だけlibmallocの契約に従う。 */
        unsafe { malloc_zone_pressure_relief(std::ptr::null_mut(), 0) }
    }
}

#[cfg(target_os = "macos")]
pub(super) fn relieve_closed_preview_memory() -> usize {
    macos::relieve()
}

#[cfg(not(target_os = "macos"))]
pub(super) fn relieve_closed_preview_memory() -> usize {
    /* WHY: 非macOSではlibmallocの契約を持たないため、製品動作を変えず観測候補だけ無効化する。 */
    0
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

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_pressure_relief_call_is_live_and_safe() {
        let live = vec![0x2a_u8; 32];
        let pointer = live.as_ptr();
        let length = live.len();
        let contents = live.clone();
        let _released = relieve_closed_preview_memory();
        assert_eq!(live.as_ptr(), pointer);
        assert_eq!(live.len(), length);
        assert_eq!(live, contents);
    }
}

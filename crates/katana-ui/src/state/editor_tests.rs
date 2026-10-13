use crate::state::{TocAnchorCandidate, TocCurrentAnchor, TocCurrentOrigin, TocState};

#[test]
fn editor_viewport_candidate_stable_for_0_025_seconds() {
    let mut toc = TocState::default();
    let same = TocAnchorCandidate {
        anchor_index: 1,
        toc_index: 2,
        source: TocCurrentOrigin::EditorViewport,
    };

    assert!(toc.should_apply_viewport_candidate(same.anchor_index, same.source));
    assert!(!toc.apply_viewport_candidate(same, 0.0));
    assert_eq!(toc.current, None);
    assert!(!toc.apply_viewport_candidate(same, 0.02));
    assert_eq!(toc.current, None);
    assert!(toc.apply_viewport_candidate(same, 0.026));
    assert_eq!(
        toc.current,
        Some(TocCurrentAnchor {
            anchor_index: 1,
            toc_index: 2,
            origin: TocCurrentOrigin::EditorViewport,
            generation: 1,
        })
    );
}

#[test]
fn editor_viewport_candidate_change_resets_stability_timer() {
    let mut toc = TocState::default();
    let first = TocAnchorCandidate {
        anchor_index: 1,
        toc_index: 2,
        source: TocCurrentOrigin::EditorViewport,
    };
    let second = TocAnchorCandidate {
        anchor_index: 3,
        toc_index: 4,
        source: TocCurrentOrigin::EditorViewport,
    };

    assert!(!toc.apply_viewport_candidate(first, 0.0));
    assert!(!toc.apply_viewport_candidate(first, 0.02));
    assert!(!toc.apply_viewport_candidate(second, 0.02));
    assert!(!toc.apply_viewport_candidate(second, 0.04));
    assert!(toc.apply_viewport_candidate(second, 0.046));
    assert_eq!(toc.current.map(|c| c.anchor_index), Some(3));
    assert_eq!(toc.current.map(|c| c.toc_index), Some(4));
}

#[test]
fn first_editor_viewport_candidate_after_click_applies_after_stability_wait() {
    let mut toc = TocState::default();
    assert!(!toc.apply_viewport_candidate(
        TocAnchorCandidate {
            anchor_index: 1,
            toc_index: 1,
            source: TocCurrentOrigin::EditorViewport,
        },
        0.0
    ));
    assert!(toc.update_current(2, 2, TocCurrentOrigin::TocClick));
    let candidate = TocAnchorCandidate {
        anchor_index: 3,
        toc_index: 3,
        source: TocCurrentOrigin::EditorViewport,
    };

    assert!(!toc.apply_viewport_candidate(candidate, 0.0));
    assert_eq!(
        toc.current,
        Some(TocCurrentAnchor {
            anchor_index: 2,
            toc_index: 2,
            origin: TocCurrentOrigin::TocClick,
            generation: 1,
        })
    );
    assert!(!toc.apply_viewport_candidate(candidate, 0.02));
    assert_eq!(toc.current.map(|c| c.anchor_index), Some(2));
    assert!(!toc.apply_viewport_candidate(candidate, 0.04));
    assert!(toc.apply_viewport_candidate(candidate, 0.046));
    assert_eq!(toc.last_editor_viewport_anchor_index, Some(3));
    assert_eq!(
        toc.current,
        Some(TocCurrentAnchor {
            anchor_index: 3,
            toc_index: 3,
            origin: TocCurrentOrigin::EditorViewport,
            generation: 2,
        })
    );
}

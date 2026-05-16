use crate::state::{TocAnchorCandidate, TocCurrentAnchor, TocCurrentOrigin, TocState};

#[test]
fn preview_viewport_candidate_stable_for_0_025_seconds() {
    let mut toc = TocState::default();
    let same = TocAnchorCandidate {
        anchor_index: 4,
        toc_index: 4,
        source: TocCurrentOrigin::PreviewViewport,
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
            anchor_index: 4,
            toc_index: 4,
            origin: TocCurrentOrigin::PreviewViewport,
            generation: 1,
        })
    );
}

#[test]
fn preview_hover_candidate_stable_for_0_025_seconds() {
    let mut toc = TocState::default();
    let same = TocAnchorCandidate {
        anchor_index: 2,
        toc_index: 2,
        source: TocCurrentOrigin::PreviewHover,
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
            anchor_index: 2,
            toc_index: 2,
            origin: TocCurrentOrigin::PreviewHover,
            generation: 1,
        })
    );
}

#[test]
fn first_preview_viewport_observation_after_click_is_suppressed_once_and_stable() {
    let mut toc = TocState::default();
    assert!(!toc.apply_viewport_candidate(
        TocAnchorCandidate {
            anchor_index: 1,
            toc_index: 1,
            source: TocCurrentOrigin::PreviewViewport,
        },
        0.0
    ));
    assert!(toc.update_current(2, 2, TocCurrentOrigin::TocClick));
    let first = TocAnchorCandidate {
        anchor_index: 3,
        toc_index: 3,
        source: TocCurrentOrigin::PreviewViewport,
    };
    let second = TocAnchorCandidate {
        anchor_index: 4,
        toc_index: 4,
        source: TocCurrentOrigin::PreviewViewport,
    };

    assert!(!toc.apply_viewport_candidate(first, 0.0));
    assert_eq!(
        toc.current,
        Some(TocCurrentAnchor {
            anchor_index: 2,
            toc_index: 2,
            origin: TocCurrentOrigin::TocClick,
            generation: 1,
        })
    );
    assert!(!toc.apply_viewport_candidate(second, 0.02));
    assert!(!toc.apply_viewport_candidate(second, 0.026));
    assert!(!toc.apply_viewport_candidate(second, 0.04));
    assert!(toc.apply_viewport_candidate(second, 0.046));
    assert_eq!(
        toc.current,
        Some(TocCurrentAnchor {
            anchor_index: 4,
            toc_index: 4,
            origin: TocCurrentOrigin::PreviewViewport,
            generation: 2,
        })
    );
}

#[test]
fn preview_hover_after_toc_click_updates_after_wait() {
    let mut toc = TocState::default();
    assert!(!toc.apply_viewport_candidate(
        TocAnchorCandidate {
            anchor_index: 1,
            toc_index: 1,
            source: TocCurrentOrigin::PreviewHover,
        },
        0.0
    ));
    assert!(toc.update_current(2, 2, TocCurrentOrigin::TocClick));
    let first = TocAnchorCandidate {
        anchor_index: 3,
        toc_index: 3,
        source: TocCurrentOrigin::PreviewHover,
    };

    assert!(!toc.apply_viewport_candidate(first, 0.0));
    assert_eq!(
        toc.current,
        Some(TocCurrentAnchor {
            anchor_index: 2,
            toc_index: 2,
            origin: TocCurrentOrigin::TocClick,
            generation: 1,
        })
    );
    assert!(toc.apply_viewport_candidate(first, 0.026));
    assert_eq!(
        toc.current,
        Some(TocCurrentAnchor {
            anchor_index: 3,
            toc_index: 3,
            origin: TocCurrentOrigin::PreviewHover,
            generation: 2,
        })
    );
}

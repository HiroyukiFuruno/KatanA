use crate::state::{TocAnchorCandidate, TocCurrentAnchor, TocCurrentOrigin, TocState};

#[test]
fn update_current_increments_generation_when_anchor_changes() {
    let mut toc = TocState::default();
    assert!(toc.update_current(1, 10, TocCurrentOrigin::TocClick));
    assert_eq!(toc.current.map(|c| c.generation), Some(1));

    assert!(!toc.update_current(1, 10, TocCurrentOrigin::TocClick));
    assert_eq!(toc.current.map(|c| c.generation), Some(1));

    assert!(toc.update_current(2, 10, TocCurrentOrigin::EditorViewport));
    assert_eq!(toc.current.map(|c| c.generation), Some(2));
}

#[test]
fn should_auto_scroll_only_when_generation_not_consumed() {
    let mut toc = TocState::default();
    toc.update_current(1, 10, TocCurrentOrigin::TocClick);
    assert!(toc.should_auto_scroll());
    toc.consume_auto_scroll();
    assert!(!toc.should_auto_scroll());

    toc.update_current(2, 10, TocCurrentOrigin::PreviewViewport);
    assert!(toc.should_auto_scroll());
    toc.consume_auto_scroll();
    assert!(!toc.should_auto_scroll());
}

#[test]
fn first_viewport_observation_keeps_toc_click_current() {
    let mut toc = TocState::default();
    assert!(toc.update_current(2, 2, TocCurrentOrigin::TocClick));
    let candidate = TocAnchorCandidate {
        anchor_index: 0,
        toc_index: 0,
        source: TocCurrentOrigin::PreviewViewport,
    };

    assert!(!toc.should_apply_viewport_candidate(candidate.anchor_index, candidate.source));
    assert!(!toc.apply_viewport_candidate(candidate, 0.0));
    assert!(!toc.apply_viewport_candidate(candidate, 0.02));
    assert_eq!(
        toc.current,
        Some(TocCurrentAnchor {
            anchor_index: 2,
            toc_index: 2,
            origin: TocCurrentOrigin::TocClick,
            generation: 1,
        })
    );
}

#[test]
fn first_viewport_observation_applies_after_stable_wait() {
    let mut toc = TocState::default();
    let candidate = TocAnchorCandidate {
        anchor_index: 1,
        toc_index: 3,
        source: TocCurrentOrigin::EditorViewport,
    };

    assert!(toc.should_apply_viewport_candidate(candidate.anchor_index, candidate.source));
    assert!(!toc.apply_viewport_candidate(candidate, 0.0));
    assert_eq!(toc.current, None);
    assert!(!toc.apply_viewport_candidate(candidate, 0.02));
    assert_eq!(toc.current, None);
    assert!(toc.apply_viewport_candidate(candidate, 0.026));
    assert_eq!(
        toc.current,
        Some(TocCurrentAnchor {
            anchor_index: 1,
            toc_index: 3,
            origin: TocCurrentOrigin::EditorViewport,
            generation: 1,
        })
    );
}

#[test]
fn first_hover_observation_overrides_toc_click_after_wait() {
    let mut toc = TocState::default();
    assert!(toc.update_current(2, 2, TocCurrentOrigin::TocClick));
    let candidate = TocAnchorCandidate {
        anchor_index: 1,
        toc_index: 1,
        source: TocCurrentOrigin::PreviewHover,
    };

    assert!(toc.should_apply_viewport_candidate(candidate.anchor_index, candidate.source));
    assert!(!toc.apply_viewport_candidate(candidate, 0.0));
    assert!(toc.should_apply_viewport_candidate(candidate.anchor_index, candidate.source));
    assert!(!toc.apply_viewport_candidate(candidate, 0.02));
    assert!(toc.apply_viewport_candidate(candidate, 0.026));
    assert_eq!(
        toc.current,
        Some(TocCurrentAnchor {
            anchor_index: 1,
            toc_index: 1,
            origin: TocCurrentOrigin::PreviewHover,
            generation: 2,
        })
    );
}

#[test]
fn toc_click_candidate_is_immediate() {
    let mut toc = TocState::default();
    let candidate = TocAnchorCandidate {
        anchor_index: 10,
        toc_index: 11,
        source: TocCurrentOrigin::TocClick,
    };
    assert!(toc.apply_viewport_candidate(candidate, 0.0));
    assert_eq!(
        toc.current,
        Some(TocCurrentAnchor {
            anchor_index: 10,
            toc_index: 11,
            origin: TocCurrentOrigin::TocClick,
            generation: 1,
        })
    );
}

#[test]
fn candidate_with_changed_toc_index_resets_stability_timer() {
    let mut toc = TocState::default();
    let first = TocAnchorCandidate {
        anchor_index: 1,
        toc_index: 1,
        source: TocCurrentOrigin::PreviewViewport,
    };
    let second = TocAnchorCandidate {
        anchor_index: 1,
        toc_index: 2,
        source: TocCurrentOrigin::PreviewViewport,
    };

    assert!(!toc.apply_viewport_candidate(first, 0.0));
    assert!(!toc.apply_viewport_candidate(second, 0.02));
    assert!(!toc.apply_viewport_candidate(second, 0.04));
    assert!(toc.apply_viewport_candidate(second, 0.046));
    assert_eq!(toc.current.map(|c| c.toc_index), Some(2));
}

#[test]
fn reset_for_document_change_clears_viewport_state() {
    let mut toc = TocState {
        current: Some(TocCurrentAnchor {
            anchor_index: 1,
            toc_index: 1,
            origin: TocCurrentOrigin::TocClick,
            generation: 3,
        }),
        auto_scroll_consumed_generation: 3,
        last_editor_viewport_anchor_index: Some(2),
        last_preview_viewport_anchor_index: Some(3),
        pending_candidate: None,
        pending_candidate_first_observed_at: None,
        suppress_next_editor_or_preview_observation: false,
        suppressed_editor_or_preview_viewport_candidate_once: false,
    };
    toc.reset_for_document_change();
    assert_eq!(toc.current, None);
    assert_eq!(toc.auto_scroll_consumed_generation, 0);
    assert_eq!(toc.last_editor_viewport_anchor_index, None);
    assert_eq!(toc.last_preview_viewport_anchor_index, None);
}

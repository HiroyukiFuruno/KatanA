pub(crate) const TOC_CANDIDATE_STABLE_SECONDS: f64 = 0.025;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TocState {
    pub current: Option<TocCurrentAnchor>,
    pub auto_scroll_consumed_generation: u64,
    pub last_editor_viewport_anchor_index: Option<usize>,
    pub last_preview_viewport_anchor_index: Option<usize>,
    pub pending_candidate: Option<TocAnchorCandidate>,
    pub pending_candidate_first_observed_at: Option<f64>,
    pub suppress_next_editor_or_preview_observation: bool,
    pub suppressed_editor_or_preview_viewport_candidate_once: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TocCurrentAnchor {
    pub anchor_index: usize,
    pub toc_index: usize,
    pub origin: TocCurrentOrigin,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TocCurrentOrigin {
    TocClick,
    EditorViewport,
    PreviewViewport,
    PreviewHover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TocAnchorCandidate {
    pub anchor_index: usize,
    pub toc_index: usize,
    pub source: TocCurrentOrigin,
}

impl TocState {
    pub fn reset_for_document_change(&mut self) {
        *self = Self::default();
    }

    fn clear_pending_candidate(&mut self) {
        self.pending_candidate = None;
        self.pending_candidate_first_observed_at = None;
    }

    fn record_pending_candidate(&mut self, candidate: TocAnchorCandidate, now: f64) {
        self.pending_candidate = Some(candidate);
        self.pending_candidate_first_observed_at = Some(now);
    }

    fn is_candidate_stable(&mut self, candidate: TocAnchorCandidate, now: f64) -> bool {
        let Some(previous) = self.pending_candidate else {
            self.record_pending_candidate(candidate, now);
            return false;
        };

        if previous == candidate {
            let first_observed_at = self.pending_candidate_first_observed_at.unwrap_or(now);
            if now - first_observed_at >= TOC_CANDIDATE_STABLE_SECONDS {
                self.clear_pending_candidate();
                return true;
            }
            return false;
        }

        self.record_pending_candidate(candidate, now);
        false
    }

    pub fn update_current(
        &mut self,
        anchor_index: usize,
        toc_index: usize,
        source: TocCurrentOrigin,
    ) -> bool {
        let generation = match self.current {
            Some(current) if current.anchor_index == anchor_index => current.generation,
            Some(current) => current.generation + 1,
            None => 1,
        };
        let updated = self
            .current
            .is_none_or(|current| current.anchor_index != anchor_index);
        self.current = Some(TocCurrentAnchor {
            anchor_index,
            toc_index,
            origin: source,
            generation,
        });
        if source == TocCurrentOrigin::TocClick {
            /* WHY: A TOC click must not be overwritten by the first viewport sample. */
            self.suppress_next_editor_or_preview_observation = true;
            self.suppressed_editor_or_preview_viewport_candidate_once = false;
            self.clear_pending_candidate();
        }
        updated
    }

    pub fn should_auto_scroll(&self) -> bool {
        self.current
            .is_some_and(|current| current.generation > self.auto_scroll_consumed_generation)
    }

    pub fn consume_auto_scroll(&mut self) {
        if let Some(current) = self.current {
            self.auto_scroll_consumed_generation = current.generation;
        }
    }

    pub fn should_apply_viewport_candidate(
        &self,
        anchor_index: usize,
        source: TocCurrentOrigin,
    ) -> bool {
        let previous_observed = match source {
            TocCurrentOrigin::EditorViewport => self.last_editor_viewport_anchor_index,
            TocCurrentOrigin::PreviewViewport | TocCurrentOrigin::PreviewHover => {
                self.last_preview_viewport_anchor_index
            }
            TocCurrentOrigin::TocClick => return true,
        };
        if self.suppress_next_editor_or_preview_observation
            && !matches!(source, TocCurrentOrigin::PreviewHover)
        {
            if self.suppressed_editor_or_preview_viewport_candidate_once {
                return true;
            }
            return false;
        }
        match previous_observed {
            Some(previous_anchor) => previous_anchor != anchor_index,
            None => match source {
                TocCurrentOrigin::PreviewHover => true,
                _ => self
                    .current
                    .is_none_or(|current| current.origin != TocCurrentOrigin::TocClick),
            },
        }
    }

    pub fn apply_viewport_candidate(&mut self, candidate: TocAnchorCandidate, now: f64) -> bool {
        if candidate.source == TocCurrentOrigin::TocClick {
            return self.update_current(
                candidate.anchor_index,
                candidate.toc_index,
                candidate.source,
            );
        }

        if self.suppress_next_editor_or_preview_observation
            && !self.suppressed_editor_or_preview_viewport_candidate_once
            && !matches!(candidate.source, TocCurrentOrigin::PreviewHover)
        {
            self.suppressed_editor_or_preview_viewport_candidate_once = true;
            self.clear_pending_candidate();
            return false;
        }

        if !self.should_apply_viewport_candidate(candidate.anchor_index, candidate.source) {
            return false;
        }

        if !self.is_candidate_stable(candidate, now) {
            return false;
        }

        if self.suppress_next_editor_or_preview_observation
            && !matches!(candidate.source, TocCurrentOrigin::PreviewHover)
        {
            self.suppressed_editor_or_preview_viewport_candidate_once = false;
            self.suppress_next_editor_or_preview_observation = false;
        }

        match candidate.source {
            TocCurrentOrigin::EditorViewport => {
                self.last_editor_viewport_anchor_index = Some(candidate.anchor_index)
            }
            TocCurrentOrigin::PreviewViewport | TocCurrentOrigin::PreviewHover => {
                self.last_preview_viewport_anchor_index = Some(candidate.anchor_index)
            }
            TocCurrentOrigin::TocClick => {}
        }

        self.update_current(
            candidate.anchor_index,
            candidate.toc_index,
            candidate.source,
        )
    }
}

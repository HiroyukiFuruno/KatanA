use std::collections::BTreeSet;

use katana_document_viewer::{
    SpreadsheetFilterCommand, SpreadsheetFilterCriterion, SpreadsheetFilterEvent,
    SpreadsheetFrameMetadata,
};

#[derive(Debug, Default)]
pub(in crate::preview_pane::document_surface) struct SpreadsheetFilterUiState {
    pub(super) metadata: Option<SpreadsheetFrameMetadata>,
    pub(super) requested: Option<(usize, usize)>,
    pub(super) candidate_values: Vec<String>,
    pub(super) selected_values: BTreeSet<String>,
    pub(super) include_blank: bool,
    pub(super) include_non_blank: bool,
    pub(super) loading: bool,
    pub(super) candidate_truncated: bool,
    has_existing_filter: bool,
    pub(super) has_unsupported_criterion: bool,
}

impl SpreadsheetFilterUiState {
    pub(in crate::preview_pane::document_surface) fn update_metadata(
        &mut self,
        metadata: Option<SpreadsheetFrameMetadata>,
        event: Option<&SpreadsheetFilterEvent>,
    ) {
        let sheet_changed = self.metadata.as_ref().map(|current| current.sheet_index)
            != metadata.as_ref().map(|current| current.sheet_index);
        self.metadata = metadata;
        if sheet_changed {
            self.reset_request();
        }
        if let Some(event) = event {
            self.apply_event(event);
        }
    }

    fn apply_event(&mut self, event: &SpreadsheetFilterEvent) {
        match event {
            SpreadsheetFilterEvent::Candidates {
                sheet_index,
                column,
                values,
                truncated,
            } => self.apply_candidates(*sheet_index, *column, values, *truncated),
            SpreadsheetFilterEvent::VisibilityChanged { sheet_index, .. } => {
                self.apply_visibility_change(*sheet_index);
            }
        }
    }

    fn apply_candidates(
        &mut self,
        sheet_index: usize,
        column: usize,
        values: &[String],
        truncated: bool,
    ) {
        let matches_request = self.requested == Some((sheet_index, column))
            && self
                .metadata
                .as_ref()
                .is_some_and(|current| current.sheet_index == sheet_index);
        if !matches_request {
            return;
        }
        let contains_blank = values.iter().any(String::is_empty);
        self.candidate_values = values
            .iter()
            .filter(|value| !value.is_empty())
            .cloned()
            .collect();
        self.candidate_truncated = truncated;
        if !self.has_existing_filter {
            self.selected_values = self.candidate_values.iter().cloned().collect();
            self.include_blank = contains_blank;
        } else if self.include_non_blank {
            self.selected_values
                .extend(self.candidate_values.iter().cloned());
        }
        self.loading = false;
    }

    fn apply_visibility_change(&mut self, sheet_index: usize) {
        if self
            .metadata
            .as_ref()
            .is_some_and(|current| current.sheet_index == sheet_index)
            && let Some((_, column)) = self.requested
        {
            self.load_selection(column);
        }
    }

    pub(super) fn request(&mut self, sheet_index: usize, column: usize) {
        self.requested = Some((sheet_index, column));
        self.candidate_values.clear();
        self.loading = true;
        self.candidate_truncated = false;
        self.load_selection(column);
    }

    fn reset_request(&mut self) {
        self.requested = None;
        self.candidate_values.clear();
        self.selected_values.clear();
        self.include_blank = false;
        self.include_non_blank = false;
        self.loading = false;
        self.candidate_truncated = false;
        self.has_existing_filter = false;
        self.has_unsupported_criterion = false;
    }

    fn load_selection(&mut self, column: usize) {
        self.selected_values.clear();
        self.include_blank = false;
        self.include_non_blank = false;
        self.has_existing_filter = false;
        self.has_unsupported_criterion = false;
        let criteria = self.criteria_for_column(column);
        self.apply_criteria(&criteria);
    }

    fn criteria_for_column(&self, column: usize) -> Vec<SpreadsheetFilterCriterion> {
        self.metadata
            .as_ref()
            .and_then(|metadata| metadata.auto_filter.as_ref())
            .and_then(|filter| filter.columns.iter().find(|item| item.column == column))
            .map(|item| item.criteria.clone())
            .unwrap_or_default()
    }

    fn apply_criteria(&mut self, criteria: &[SpreadsheetFilterCriterion]) {
        self.has_existing_filter = !criteria.is_empty();
        for criterion in criteria {
            match criterion {
                SpreadsheetFilterCriterion::Values(values) => self.apply_values(values),
                SpreadsheetFilterCriterion::Blank => self.include_blank = true,
                SpreadsheetFilterCriterion::NonBlank => self.include_non_blank = true,
                SpreadsheetFilterCriterion::Unsupported(_) => {
                    self.has_unsupported_criterion = true;
                }
            }
        }
    }

    fn apply_values(&mut self, values: &[String]) {
        for value in values {
            if value.is_empty() {
                self.include_blank = true;
            } else {
                self.selected_values.insert(value.clone());
            }
        }
    }
}

pub(in crate::preview_pane::document_surface) fn apply_command(
    state: &SpreadsheetFilterUiState,
    sheet_index: usize,
    column: usize,
) -> Option<SpreadsheetFilterCommand> {
    if state.candidate_truncated || state.has_unsupported_criterion {
        return None;
    }
    Some(SpreadsheetFilterCommand::ApplyValues {
        sheet_index,
        column,
        values: selected_filter_values(state),
    })
}

pub(in crate::preview_pane::document_surface) fn selected_filter_values(
    state: &SpreadsheetFilterUiState,
) -> Vec<String> {
    let mut values: Vec<String> = state
        .selected_values
        .iter()
        .filter(|value| !value.is_empty())
        .cloned()
        .collect();
    if state.include_blank {
        values.push(String::new());
    }
    values
}

mod compaction;
mod fix_application;
mod model;
mod state;
mod types;

pub(crate) use compaction::DiffCompactionOps;
pub(crate) use fix_application::DiagnosticFixApplicationOps;
pub(crate) use model::DiffModelOps;
pub(crate) use state::{DiffReviewDecision, DiffReviewFile, DiffReviewState};
pub(crate) use types::{
    DiffCell, DiffLine, DiffLineKind, FileDiffModel, InlineDiffRow, SplitDiffLine, SplitDiffRow,
};

#[cfg(test)]
mod fix_application_tests;
#[cfg(test)]
mod model_tests;
#[cfg(test)]
mod state_tests;

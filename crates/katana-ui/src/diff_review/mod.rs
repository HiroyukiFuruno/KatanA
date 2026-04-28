mod compaction;
mod fix_application;
mod highlight;
mod model;
mod split_model;
mod split_pairing;
mod state;
mod types;

pub(crate) use compaction::DiffCompactionOps;
pub(crate) use fix_application::DiagnosticFixApplicationOps;
pub(crate) use model::DiffModelOps;
pub(crate) use state::{DiffReviewDecision, DiffReviewFile, DiffReviewState};
pub(crate) use types::{
    DiffCell, DiffLine, DiffLineKind, FileDiffModel, InlineDiffRow, SplitDiffLine, SplitDiffRow,
    TextRange, UnchangedBlock,
};

#[cfg(test)]
mod fix_application_tests;
#[cfg(test)]
mod model_coverage_tests;
#[cfg(test)]
mod model_pairing_tests;
#[cfg(test)]
mod model_tests;
#[cfg(test)]
mod state_tests;

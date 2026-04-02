#![deny(warnings, clippy::all)]
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_or_default,
    clippy::wildcard_imports,
    clippy::match_wild_err_arm,
    clippy::let_and_return,
    clippy::manual_ok_err,
    clippy::cognitive_complexity,
    clippy::type_complexity
)]

pub mod ai;
pub mod document;
pub mod emoji;
pub mod html;
pub mod markdown;
pub mod plugin;
pub mod preview;
pub mod search;
pub mod update;
pub mod workspace;

pub use document::Document;
pub use workspace::Workspace;

/* WHY: Refactored search tabs module entry point to maintain a clean structure and manage architectural complexity via specialized sub-modules. */

pub mod filename_tab;
pub mod history;
pub mod md_tab;
pub mod utils;

pub(super) use filename_tab::FilenameTabOps;
pub(super) use md_tab::MdTabOps;

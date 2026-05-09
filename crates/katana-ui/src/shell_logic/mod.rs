pub mod logic;
mod title;
#[cfg(test)]
mod title_tests;
pub mod utils;

pub use utils::ShellUtils;
pub struct ShellLogicOps;

pub use crate::app_state::AppAction;

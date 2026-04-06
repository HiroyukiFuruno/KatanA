pub mod defaults;
pub mod defaults_impls;
pub mod impls;
pub mod migration;
pub mod repository;
pub mod service;
pub mod types;

pub use defaults::{DEFAULT_IGNORED_DIRECTORIES, DEFAULT_MAX_DEPTH, SettingsDefaultOps};
pub use repository::{InMemoryRepository, JsonFileRepository, SettingsRepository};
pub use service::SettingsService;
pub use types::*;

#[cfg(test)]
mod tests;

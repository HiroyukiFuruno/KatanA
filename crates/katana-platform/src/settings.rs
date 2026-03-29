//! Application settings persistence.
//!
//! Settings are loaded from and saved to a JSON file via `JsonFileRepository`.
//! For tests, `InMemoryRepository` provides a no-op backend.
//!
//! ## Module structure
//!
//! | Module | Responsibility |
//! |---|---|
//! | `types` | All struct, enum, and constant type definitions |
//! | `defaults` | Serde default functions + Default impls |
//! | `impls` | `AppSettings` method implementations |
//! | `repository` | `SettingsRepository` trait + JSON/InMemory implementations |
//! | `service` | `SettingsService` (business logic) |
//! | `migration/` | Schema migration (trait + versioned strategies) |

pub mod defaults;
pub mod impls;
pub mod migration;
pub mod repository;
pub mod service;
pub mod types;

// Public API re-exports to preserve `use crate::settings::*` compatibility.
pub use defaults::default_true;
pub use repository::{InMemoryRepository, JsonFileRepository, SettingsRepository};
pub use service::SettingsService;
pub use types::*;

#[cfg(test)]
mod tests;

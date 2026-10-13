use super::adapter::DiagramBackendAdapter;
use super::katana_backend;
use super::types::DiagramBackendLanguage;

/// Factory for diagram backend adapters.
pub struct DiagramBackendFactory;

impl DiagramBackendFactory {
    pub fn create(language: DiagramBackendLanguage) -> Box<dyn DiagramBackendAdapter> {
        katana_backend::create_backend(language)
    }
}

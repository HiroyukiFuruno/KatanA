use super::{AiCapabilities, AiError, AiModel, AiRequest, AiResponse};

pub trait AiProvider: Send + Sync {
    fn id(&self) -> &str;

    fn display_name(&self) -> &str;

    fn execute(&self, request: &AiRequest) -> Result<AiResponse, AiError>;

    fn is_available(&self) -> bool;

    fn capabilities(&self) -> AiCapabilities {
        AiCapabilities::default()
    }

    fn list_models(&self) -> Result<Vec<AiModel>, AiError> {
        Ok(Vec::new())
    }
}

#[derive(Default)]
pub struct AiProviderRegistry {
    providers: Vec<Box<dyn AiProvider>>,
    active_id: Option<String>,
}

impl AiProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: Box<dyn AiProvider>) {
        let id = provider.id().to_string();
        if let Some(index) = self
            .providers
            .iter()
            .position(|provider| provider.id() == id)
        {
            self.providers[index] = provider;
        } else {
            self.providers.push(provider);
        }
    }

    pub fn set_active(&mut self, id: &str) -> bool {
        if self.providers.iter().any(|provider| provider.id() == id) {
            self.active_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    pub fn execute(&self, request: &AiRequest) -> Result<AiResponse, AiError> {
        let provider = self.active_provider()?;
        if !provider.is_available() {
            return Err(AiError::NotConfigured);
        }
        provider.execute(request)
    }

    pub fn has_active_provider(&self) -> bool {
        self.active_provider()
            .map(|provider| provider.is_available())
            .unwrap_or(false)
    }

    pub fn active_capabilities(&self) -> Result<AiCapabilities, AiError> {
        self.active_provider().map(AiProvider::capabilities)
    }

    pub fn list_models(&self) -> Result<Vec<AiModel>, AiError> {
        self.active_provider()?.list_models()
    }

    fn active_provider(&self) -> Result<&dyn AiProvider, AiError> {
        let id = self.active_id.as_deref().ok_or(AiError::NotConfigured)?;
        self.providers
            .iter()
            .find(|provider| provider.id() == id)
            .map(|provider| provider.as_ref())
            .ok_or(AiError::NotConfigured)
    }
}

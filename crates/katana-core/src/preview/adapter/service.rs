use super::{PreviewAdapterResult, PreviewInput};

pub trait PreviewAdapter: Send + Sync {
    fn render(&self, input: &PreviewInput) -> PreviewAdapterResult;
}

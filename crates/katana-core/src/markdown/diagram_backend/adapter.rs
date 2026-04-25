use super::DiagramBackendVersion;
use super::{DiagramBackendId, DiagramBackendInput, DiagramBackendRenderResult};

pub trait DiagramBackendAdapter: Send + Sync {
    fn id(&self) -> &DiagramBackendId;
    fn version(&self) -> &DiagramBackendVersion;
    fn render(&self, input: &DiagramBackendInput) -> DiagramBackendRenderResult;
}

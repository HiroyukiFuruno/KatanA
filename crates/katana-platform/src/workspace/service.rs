use super::repository::GlobalWorkspaceRepository;
use super::types::GlobalWorkspaceState;

pub struct GlobalWorkspaceService {
    state: GlobalWorkspaceState,
    repository: Box<dyn GlobalWorkspaceRepository>,
}

impl GlobalWorkspaceService {
    pub fn new(repository: Box<dyn GlobalWorkspaceRepository>) -> Self {
        let state = repository.load();
        Self { state, repository }
    }

    pub fn state(&self) -> &GlobalWorkspaceState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut GlobalWorkspaceState {
        &mut self.state
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn save(&self) -> anyhow::Result<()> {
        self.repository.save(&self.state)
    }

    pub fn reload(&mut self) {
        /* WHY: Re-read from disk to get the latest state, e.g. when opening history or workspace panel */
        self.state = self.repository.load();
    }

    pub fn is_ephemeral(&self) -> bool {
        self.repository.is_ephemeral()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::repository::InMemoryWorkspaceRepository;

    #[test]
    fn test_reload_refreshes_state_from_repository() {
        let initial = GlobalWorkspaceState {
            persisted: vec!["/initial".to_string()],
            histories: vec![],
        };
        let repo = InMemoryWorkspaceRepository::new(initial);
        let mut svc = GlobalWorkspaceService::new(Box::new(repo));

        /* WHY: Mutate state directly without saving, then reload should revert to repo state */
        svc.state_mut().persisted.push("/local_only".to_string());
        assert_eq!(svc.state().persisted.len(), 2);

        svc.reload();
        /* WHY: After reload, the service state should be the saved repository state again */
        assert_eq!(svc.state().persisted, vec!["/initial"]);
    }

    #[test]
    fn test_is_ephemeral_delegates_to_repository() {
        let repo = InMemoryWorkspaceRepository::default();
        let svc = GlobalWorkspaceService::new(Box::new(repo));

        assert!(svc.is_ephemeral());
    }
}

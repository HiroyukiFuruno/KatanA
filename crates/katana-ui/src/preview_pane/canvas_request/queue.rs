use std::collections::VecDeque;
use std::sync::Weak;

use super::state::RequestState;

#[derive(Default)]
pub(super) struct QueueState {
    pub pending: VecDeque<Weak<RequestState>>,
    pub active: Weak<RequestState>,
}

impl QueueState {
    pub(super) fn drain(&mut self) -> Vec<Weak<RequestState>> {
        let mut requests: Vec<_> = self.pending.drain(..).collect();
        requests.push(std::mem::take(&mut self.active));
        requests
    }
}

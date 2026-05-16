use super::types::PreviewPane;

impl PreviewPane {
    pub fn abort_renders(&mut self) {
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.is_loading = false;
        self.render_rx = None;
    }
}

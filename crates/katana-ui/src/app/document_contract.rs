pub(crate) trait DocumentOps {
    fn handle_select_document(&mut self, path: std::path::PathBuf, activate: bool);
    fn force_close_document(&mut self, idx: usize);
    fn handle_update_buffer(&mut self, content: String);
    fn handle_save_document(&mut self);
}

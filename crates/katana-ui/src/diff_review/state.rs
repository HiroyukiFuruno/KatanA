use std::path::{Path, PathBuf};

use katana_platform::DiffViewMode;

use super::{DiffModelOps, FileDiffModel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffReviewDecision {
    Pending,
    Applied,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffReviewFile {
    pub(crate) path: PathBuf,
    pub(crate) before: String,
    pub(crate) after: String,
    pub(crate) model: FileDiffModel,
    pub(crate) decision: DiffReviewDecision,
}

impl DiffReviewFile {
    pub(crate) fn new(path: PathBuf, before: String, after: String) -> Self {
        let model = DiffModelOps::build(&before, &after);
        Self {
            path,
            before,
            after,
            model,
            decision: DiffReviewDecision::Pending,
        }
    }

    pub(crate) fn display_name(&self, workspace_root: Option<&Path>) -> String {
        if let Some(root) = workspace_root
            && let Ok(relative) = self.path.strip_prefix(root)
        {
            return relative.display().to_string();
        }
        self.path.display().to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffReviewState {
    pub(crate) files: Vec<DiffReviewFile>,
    pub(crate) current_index: usize,
    pub(crate) mode: DiffViewMode,
    pub(crate) is_fullscreen: bool,
    pub(crate) restore_path: Option<PathBuf>,
    pub(crate) workspace_root: Option<PathBuf>,
}

impl DiffReviewState {
    pub(crate) fn new(
        files: Vec<DiffReviewFile>,
        mode: DiffViewMode,
        restore_path: Option<PathBuf>,
    ) -> Self {
        Self {
            files,
            current_index: 0,
            mode,
            is_fullscreen: false,
            restore_path,
            workspace_root: None,
        }
    }

    pub(crate) fn with_workspace_root(mut self, workspace_root: Option<PathBuf>) -> Self {
        self.workspace_root = workspace_root;
        self
    }

    pub(crate) fn current_file(&self) -> Option<&DiffReviewFile> {
        self.files.get(self.current_index)
    }

    pub(crate) fn current_file_mut(&mut self) -> Option<&mut DiffReviewFile> {
        self.files.get_mut(self.current_index)
    }

    pub(crate) fn current_file_display_name(&self) -> String {
        self.current_file()
            .map(|file| file.display_name(self.workspace_root.as_deref()))
            .unwrap_or_default()
    }

    pub(crate) fn move_previous(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
    }

    pub(crate) fn move_next(&mut self) {
        if self.current_index + 1 < self.files.len() {
            self.current_index += 1;
        }
    }

    pub(crate) fn can_move_previous(&self) -> bool {
        self.current_index > 0
    }

    pub(crate) fn can_move_next(&self) -> bool {
        self.current_index + 1 < self.files.len()
    }

    pub(crate) fn mark_current(&mut self, decision: DiffReviewDecision) {
        if let Some(file) = self.current_file_mut() {
            file.decision = decision;
        }
        if let Some(index) = self.next_pending_index() {
            self.current_index = index;
        }
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.files
            .iter()
            .all(|file| file.decision != DiffReviewDecision::Pending)
    }

    pub(crate) fn reject_all_pending(&mut self) {
        for file in &mut self.files {
            if file.decision == DiffReviewDecision::Pending {
                file.decision = DiffReviewDecision::Rejected;
            }
        }
    }

    fn next_pending_index(&self) -> Option<usize> {
        let next_start = self.current_index.saturating_add(1);
        self.files
            .iter()
            .enumerate()
            .skip(next_start)
            .find(|(_, file)| file.decision == DiffReviewDecision::Pending)
            .map(|(index, _)| index)
            .or_else(|| {
                self.files
                    .iter()
                    .enumerate()
                    .take(self.current_index)
                    .find(|(_, file)| file.decision == DiffReviewDecision::Pending)
                    .map(|(index, _)| index)
            })
    }
}

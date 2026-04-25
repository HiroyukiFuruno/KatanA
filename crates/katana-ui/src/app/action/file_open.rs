use crate::app::DocumentOps;
use crate::app::WorkspaceOps;
use crate::app_state::{AppAction, StatusType};
use crate::shell::KatanaApp;
use std::path::{Path, PathBuf};

pub(crate) struct FileOpenOps;

impl FileOpenOps {
    const DRAWIO_EXTENSIONS: &'static [&'static str] = &["drawio", "drowio"];

    pub(crate) fn supported_extensions(app: &KatanaApp) -> Vec<String> {
        let mut extensions = app
            .state
            .config
            .settings
            .settings()
            .workspace
            .visible_extensions
            .clone();
        Self::append_extensions(
            &mut extensions,
            katana_core::workspace::TreeEntry::image_extensions(),
        );
        Self::append_extensions(&mut extensions, Self::DRAWIO_EXTENSIONS);
        extensions
    }

    pub(crate) fn dialog_extensions(app: &KatanaApp) -> Vec<String> {
        Self::supported_extensions(app)
            .into_iter()
            .filter(|extension| !extension.is_empty())
            .collect()
    }

    pub(crate) fn open_in_current_workspace(app: &mut KatanaApp, path: PathBuf) {
        if !Self::is_openable_file(app, &path) {
            Self::show_invalid_file(app, &path);
            return;
        }
        if app.state.workspace.data.is_none() {
            Self::open_as_temporary_workspace(app, path);
            return;
        }
        app.handle_select_document(path, true);
    }

    pub(crate) fn open_as_temporary_workspace(app: &mut KatanaApp, path: PathBuf) {
        if !Self::is_openable_file(app, &path) {
            Self::show_invalid_file(app, &path);
            return;
        }
        let Some(parent_dir) = path.parent().map(Path::to_path_buf) else {
            Self::show_invalid_file(app, &path);
            return;
        };
        app.pending_workspace_file_open = Some(path);
        app.state
            .workspace
            .temporary_roots
            .insert(parent_dir.clone());
        app.handle_open_explorer(parent_dir);
    }

    pub(crate) fn open_dropped_files(app: &mut KatanaApp, paths: Vec<PathBuf>) {
        let files = Self::openable_files(app, paths);
        if files.is_empty() {
            return;
        }
        if app.state.workspace.data.is_none() {
            if let Some(first_path) = files.first() {
                Self::open_as_temporary_workspace(app, first_path.clone());
            }
            for path in files.into_iter().skip(1) {
                app.pending_document_loads.push_back(path);
            }
            return;
        }
        app.handle_action_open_multiple(files);
    }

    pub(crate) fn dropped_file_paths(ctx: &egui::Context) -> Vec<PathBuf> {
        ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect()
        })
    }

    pub(crate) fn dropped_openable_file_paths(
        app: &KatanaApp,
        ctx: &egui::Context,
    ) -> Vec<PathBuf> {
        Self::openable_files(app, Self::dropped_file_paths(ctx))
    }

    pub(crate) fn is_openable_file(app: &KatanaApp, path: &Path) -> bool {
        path.is_file() && Self::has_supported_extension(app, path)
    }

    fn has_supported_extension(app: &KatanaApp, path: &Path) -> bool {
        match path.extension().and_then(|it| it.to_str()) {
            Some(extension) => Self::is_supported_extension(app, extension),
            None => Self::supported_extensions(app)
                .iter()
                .any(|extension| extension.is_empty()),
        }
    }

    fn is_supported_extension(app: &KatanaApp, extension: &str) -> bool {
        Self::supported_extensions(app)
            .iter()
            .any(|supported| supported.eq_ignore_ascii_case(extension))
    }

    fn openable_files(app: &KatanaApp, paths: Vec<PathBuf>) -> Vec<PathBuf> {
        paths
            .into_iter()
            .filter(|path| Self::is_openable_file(app, path))
            .collect()
    }

    fn append_extensions(extensions: &mut Vec<String>, additions: &[&str]) {
        for addition in additions {
            if !extensions
                .iter()
                .any(|extension| extension.eq_ignore_ascii_case(addition))
            {
                extensions.push((*addition).to_string());
            }
        }
    }

    fn show_invalid_file(app: &mut KatanaApp, path: &Path) {
        let error = path.display().to_string();
        let message = crate::i18n::I18nOps::tf(
            &crate::i18n::I18nOps::get().status.cannot_open_file,
            &[("error", error.as_str())],
        );
        app.state.layout.status_message = Some((message, StatusType::Warning));
    }
}

impl KatanaApp {
    pub(super) fn handle_action_pick_open_file(&mut self, action: AppAction) {
        if crate::shell_ui::ShellUiOps::is_headless() {
            self.pending_dialog_action = Some(action);
            self.file_dialog.pick_file();
            return;
        }
        let result = crate::shell_ui::ShellUiOps::open_file_dialog_result(
            &FileOpenOps::dialog_extensions(self),
        );
        self.handle_pick_open_file_result(result, action);
    }

    pub(crate) fn handle_pick_open_file_result(
        &mut self,
        result: crate::shell_ui::NativeDialogResult<PathBuf>,
        action: AppAction,
    ) {
        match result {
            crate::shell_ui::NativeDialogResult::Picked(path) => match action {
                AppAction::PickOpenFileInNewWorkspace => {
                    FileOpenOps::open_as_temporary_workspace(self, path);
                }
                AppAction::PickOpenFileInCurrentWorkspace => {
                    FileOpenOps::open_in_current_workspace(self, path);
                }
                _ => {}
            },
            crate::shell_ui::NativeDialogResult::Cancelled => {}
            crate::shell_ui::NativeDialogResult::Unavailable => {
                self.pending_dialog_action = Some(action);
                self.file_dialog.pick_file();
            }
        }
    }
}

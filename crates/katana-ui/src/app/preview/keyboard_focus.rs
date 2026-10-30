use crate::app_state::AppAction;
use eframe::egui;
use katana_core::document_source::BinaryDocumentFormat;
use std::path::{Path, PathBuf};

const REQUEST_ID: &str = "katana.document-page-focus-request";

#[derive(Clone)]
struct Request {
    owner_path: PathBuf,
    focus_owner: Option<egui::Id>,
}

pub(crate) struct PageNavigationFocusOps;

impl PageNavigationFocusOps {
    pub(crate) fn selection_path(action: &AppAction) -> Option<PathBuf> {
        match action {
            AppAction::SelectDocument(path) => Some(path.clone()),
            AppAction::OpenMultipleDocuments(paths) => paths.first().cloned(),
            _ => None,
        }
    }

    pub(crate) fn finish_action(
        ctx: &egui::Context,
        selected_document: Option<PathBuf>,
        opens_file: bool,
        previous_active_path: Option<PathBuf>,
        active_path: Option<PathBuf>,
        pending_workspace_file_open: Option<&Path>,
    ) {
        if let Some(path) = selected_document {
            if active_path.as_ref() == Some(&path) {
                Self::arm_for_selection(ctx, &path);
            } else {
                Self::clear(ctx);
            }
        } else if opens_file {
            let activated_path = pending_workspace_file_open.or_else(|| {
                active_path
                    .as_deref()
                    .filter(|path| Some(*path) != previous_active_path.as_deref())
            });
            if let Some(path) = activated_path {
                Self::arm_for_selection(ctx, path);
            }
        } else if previous_active_path != active_path
            && let Some(path) = active_path.as_deref()
        {
            Self::arm_for_selection(ctx, path);
        }
    }

    pub(crate) fn finish_file_dialog(
        ctx: &egui::Context,
        selected_path: &Path,
        active_path: Option<&Path>,
        pending_path: Option<&Path>,
    ) {
        let activated = active_path == Some(selected_path) || pending_path == Some(selected_path);
        if activated {
            Self::arm_for_selection(ctx, selected_path);
        } else {
            Self::clear(ctx);
        }
    }

    pub(crate) fn arm_for_selection(ctx: &egui::Context, path: &Path) {
        let Some(format) = BinaryDocumentFormat::from_path(path) else {
            Self::clear(ctx);
            return;
        };
        if !matches!(
            format,
            BinaryDocumentFormat::Pdf | BinaryDocumentFormat::Pptx
        ) {
            Self::clear(ctx);
            return;
        }
        let focus_owner = ctx.memory(|memory| memory.focused());
        ctx.data_mut(|data| {
            data.insert_temp(
                egui::Id::new(REQUEST_ID),
                Request {
                    owner_path: path.to_path_buf(),
                    focus_owner,
                },
            )
        });
    }

    pub(crate) fn clear(ctx: &egui::Context) {
        ctx.data_mut(|data| data.remove::<Request>(egui::Id::new(REQUEST_ID)));
    }

    pub(crate) fn focus_if_activated(
        ctx: &egui::Context,
        response: &egui::Response,
        owner_path: &Path,
    ) -> Option<bool> {
        let request = ctx.data_mut(|data| data.get_temp::<Request>(egui::Id::new(REQUEST_ID)));
        let request = request.filter(|request| request.owner_path == owner_path)?;
        Self::clear(ctx);
        let current_owner = ctx.memory(|memory| memory.focused());
        /* WHY: 読込中に旧widgetが消えた場合は、別の入力先への移動と区別する。 */
        if current_owner.is_some() && current_owner != request.focus_owner {
            return Some(false);
        }
        response.request_focus();
        Some(true)
    }
}

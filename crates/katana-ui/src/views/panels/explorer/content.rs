use super::types::ExplorerLogicOps;
use crate::app_state::AppAction;
use crate::shell_ui::TreeRenderContext;
use eframe::egui;
use std::sync::atomic::{AtomicU64, Ordering};

const SLOW_EXPLORER_FRAME_MICROS: u128 = 8_000;
const EXPLORER_TRACE_SAMPLE_INTERVAL: u64 = 120;
static EXPLORER_FRAME_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) struct ExplorerContent<'a> {
    pub workspace: &'a mut crate::app_state::WorkspaceState,
    pub search: &'a mut crate::app_state::SearchState,
    pub histories: &'a [String],
    pub active_path: Option<&'a std::path::Path>,
    pub tab_groups: &'a [crate::state::document::TabGroup],
    pub action: &'a mut AppAction,
    pub show_vertical_line: bool,
}

impl<'a> ExplorerContent<'a> {
    pub fn new(
        workspace: &'a mut crate::app_state::WorkspaceState,
        search: &'a mut crate::app_state::SearchState,
        histories: &'a [String],
        active_path: Option<&'a std::path::Path>,
        tab_groups: &'a [crate::state::document::TabGroup],
        action: &'a mut AppAction,
        show_vertical_line: bool,
    ) -> Self {
        Self {
            workspace,
            search,
            histories,
            active_path,
            tab_groups,
            action,
            show_vertical_line,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        if self.workspace.data.is_some() {
            self.show_active_workspace(ui);
        } else {
            crate::views::panels::explorer::empty::EmptyWorkspaceView::new(
                self.histories,
                self.action,
            )
            .show(ui);
        }
    }

    fn show_active_workspace(self, ui: &mut egui::Ui) {
        let frame_started = std::time::Instant::now();
        let (workspace, search, active_path, action) =
            (self.workspace, self.search, self.active_path, self.action);
        ExplorerLogicOps::update_tree_expansion(workspace);
        let ws = workspace.data.as_ref().expect("workspace must be active");
        ExplorerLogicOps::update_search_filter_cache(search, &ws.root, &ws.tree, ws.revision());
        let is_flat_view = workspace.is_flat_view(&ws.root);
        let filter = search
            .filter_cache
            .as_ref()
            .map(|(params, paths)| (params, paths));
        let projection_started = std::time::Instant::now();
        let rebuilt = workspace.explorer_projection.refresh(
            ws,
            &workspace.expanded_directories,
            filter,
            is_flat_view,
        );
        let projection_elapsed = projection_started.elapsed();
        let filter_set = filter.map(|(_, paths)| paths);
        let mut ctx = TreeRenderContext {
            action,
            depth: 0,
            active_path,
            filter_set,
            expanded_directories: &mut workspace.expanded_directories,
            disable_context_menu: false,
            is_flat_view,
            ws_root: Some(&ws.root),
            tab_groups: Some(self.tab_groups),
            show_vertical_line: self.show_vertical_line,
        };
        let visible_rows = Self::show_virtualized_tree(
            ui,
            &ws.tree,
            workspace.explorer_projection.rows(),
            &mut ctx,
            &ws.root,
        );
        let frame_elapsed = frame_started.elapsed();
        let sequence = EXPLORER_FRAME_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        if should_trace_explorer_frame(rebuilt, frame_elapsed.as_micros(), sequence) {
            crate::debug_log::DebugLog::write(
                "explorer_frame",
                format_args!(
                    "workspace_revision={} total_rows={} visible_rows={} projection_rebuilt={} projection_bytes={} projection_us={} frame_us={}",
                    ws.revision(),
                    workspace.explorer_projection.rows().len(),
                    visible_rows,
                    rebuilt,
                    workspace.explorer_projection.estimated_heap_bytes(),
                    projection_elapsed.as_micros(),
                    frame_elapsed.as_micros(),
                ),
            );
        }
    }

    #[allow(dead_code)]
    pub(crate) fn format_workspace_markdown_action(ws_root: &std::path::Path) -> AppAction {
        AppAction::FormatWorkspaceMarkdown(ws_root.to_path_buf())
    }

    #[allow(dead_code)]
    pub(crate) fn new_workspace_root_file_action(ws_root: &std::path::Path) -> AppAction {
        AppAction::RequestNewFile(ws_root.to_path_buf())
    }

    #[allow(dead_code)]
    pub(crate) fn new_workspace_root_directory_action(ws_root: &std::path::Path) -> AppAction {
        AppAction::RequestNewDirectory(ws_root.to_path_buf())
    }
}

const fn should_trace_explorer_frame(rebuilt: bool, elapsed_micros: u128, sequence: u64) -> bool {
    rebuilt
        || elapsed_micros >= SLOW_EXPLORER_FRAME_MICROS
        || sequence.is_multiple_of(EXPLORER_TRACE_SAMPLE_INTERVAL)
}

#[cfg(test)]
mod trace_tests {
    use super::{EXPLORER_TRACE_SAMPLE_INTERVAL, should_trace_explorer_frame};

    #[test]
    fn explorer_trace_keeps_rebuilds_slow_frames_and_periodic_samples() {
        assert!(should_trace_explorer_frame(true, 1, 1));
        assert!(should_trace_explorer_frame(false, 8_000, 1));
        assert!(should_trace_explorer_frame(
            false,
            1,
            EXPLORER_TRACE_SAMPLE_INTERVAL
        ));
        assert!(!should_trace_explorer_frame(false, 1, 1));
    }
}

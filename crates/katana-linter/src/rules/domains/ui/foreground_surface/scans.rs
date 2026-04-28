use crate::Violation;
use std::path::{Path, PathBuf};

pub(super) struct ForegroundSurfaceScanOps;

impl ForegroundSurfaceScanOps {
    pub(super) fn lint(workspace_root: &Path, violations: &mut Vec<Violation>) {
        let ui_src = workspace_root.join("crates/katana-ui/src");
        let mut files = Vec::new();
        Self::collect_rust_files(&ui_src, &mut files);
        Self::lint_foreground_area_contract(&files, violations);
        Self::lint_direct_window_contract(&files, violations);
    }

    fn lint_foreground_area_contract(files: &[PathBuf], violations: &mut Vec<Violation>) {
        for file in files {
            let Ok(source) = std::fs::read_to_string(file) else {
                continue;
            };
            if !Self::has_foreground_area(&source) || Self::is_guarded_foreground_area(&source) {
                continue;
            }

            violations.push(Violation {
                file: file.clone(),
                line: Self::line_number_of(&source, "egui::Order::Foreground"),
                column: 1,
                message: "Foreground egui::Area must consume pointer and hover events through InteractionFacade or an explicit full-screen blocker.".to_string(),
            });
        }
    }

    fn lint_direct_window_contract(files: &[PathBuf], violations: &mut Vec<Violation>) {
        for file in files {
            let Ok(source) = std::fs::read_to_string(file) else {
                continue;
            };
            if !source.contains("egui::Window::new") || Self::is_known_window_surface(file) {
                continue;
            }

            violations.push(Violation {
                file: file.clone(),
                line: Self::line_number_of(&source, "egui::Window::new"),
                column: 1,
                message: "New modal/popup window surfaces must use crate::widgets::Modal or be registered in foreground-surface lint with InteractionFacade coverage.".to_string(),
            });
        }
    }

    fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::collect_rust_files(&path, files);
                continue;
            }
            if path.extension().and_then(|it| it.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }

    fn has_foreground_area(source: &str) -> bool {
        source.contains("egui::Area::new") && source.contains("egui::Order::Foreground")
    }

    fn is_guarded_foreground_area(source: &str) -> bool {
        source.contains("InteractionFacade::consume_rect")
            || source.contains("allocate_exact_size(screen.size(), egui::Sense::click_and_drag())")
            || source.contains("allocate_rect(ctx.screen_rect(), egui::Sense::all())")
            || source.contains("drag_ghost")
            || source.contains("rail_ghost")
    }

    fn line_number_of(source: &str, token: &str) -> usize {
        source
            .lines()
            .position(|line| line.contains(token))
            .map_or(1, |index| index + 1)
    }

    fn is_known_window_surface(file: &Path) -> bool {
        let normalized = file.to_string_lossy();
        Self::known_window_surface_suffixes()
            .iter()
            .any(|suffix| normalized.ends_with(suffix))
    }

    fn known_window_surface_suffixes() -> &'static [&'static str] {
        &[
            "crates/katana-ui/src/widgets/modal/ui.rs",
            "crates/katana-ui/src/settings/ui.rs",
            "crates/katana-ui/src/views/modals/file_ops_rename_delete.rs",
            "crates/katana-ui/src/views/modals/file_ops_move.rs",
            "crates/katana-ui/src/views/modals/file_ops.rs",
            "crates/katana-ui/src/views/modals/search.rs",
            "crates/katana-ui/src/views/modals/meta_info.rs",
            "crates/katana-ui/src/views/modals/workspace_toggle.rs",
            "crates/katana-ui/src/views/modals/command_palette.rs",
            "crates/katana-ui/src/views/modals/about.rs",
            "crates/katana-ui/src/settings/tabs/icons/popups.rs",
            "crates/katana-ui/src/settings/tabs/linter/preset_dialog.rs",
            "crates/katana-ui/src/settings/tabs/workspace_modal.rs",
            "crates/katana-ui/src/settings/tabs/theme_editor/modal.rs",
            "crates/katana-ui/src/settings/tabs/behavior/performance.rs",
        ]
    }
}

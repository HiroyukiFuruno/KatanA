use crate::Violation;
use std::path::{Path, PathBuf};

pub struct ForegroundSurfaceOps;

impl ForegroundSurfaceOps {
    pub fn lint(workspace_root: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();
        let shell_path = workspace_root.join("crates/katana-ui/src/shell/mod.rs");
        let shell_ui_path = workspace_root.join("crates/katana-ui/src/shell_ui/mod.rs");
        let central_content_path =
            workspace_root.join("crates/katana-ui/src/views/app_frame/central_content.rs");
        let slideshow_settings_path =
            workspace_root.join("crates/katana-ui/src/preview_pane/slideshow/settings.rs");
        let slideshow_modal_path =
            workspace_root.join("crates/katana-ui/src/preview_pane/slideshow/modal.rs");

        let Ok(shell_source) = std::fs::read_to_string(&shell_path) else {
            return violations;
        };
        let Ok(shell_ui_source) = std::fs::read_to_string(&shell_ui_path) else {
            return violations;
        };
        let Ok(central_content_source) = std::fs::read_to_string(&central_content_path) else {
            return violations;
        };
        let Ok(slideshow_source) = std::fs::read_to_string(&slideshow_modal_path) else {
            return violations;
        };
        let Ok(slideshow_settings_source) = std::fs::read_to_string(&slideshow_settings_path)
        else {
            return violations;
        };

        Self::require_contains(
            &shell_path,
            &shell_source,
            "show_slideshow",
            "Foreground surface detection must include slideshow mode.",
            &mut violations,
        );
        Self::require_contains(
            &shell_path,
            &shell_source,
            "any_popup_open",
            "Foreground surface detection must include egui popup/context-menu state.",
            &mut violations,
        );
        Self::require_contains(
            &shell_ui_path,
            &shell_ui_source,
            "InteractionFacade::begin_frame",
            "InteractionFacade must refresh global hover blockers at the beginning of each frame.",
            &mut violations,
        );
        Self::require_contains(
            &central_content_path,
            &central_content_source,
            "layout.show_slideshow",
            "Preview side panels must not render while slideshow mode is active.",
            &mut violations,
        );
        Self::require_contains(
            &slideshow_modal_path,
            &slideshow_source,
            "click_and_drag",
            "Slideshow overlay blocker must consume pointer events.",
            &mut violations,
        );
        Self::require_contains(
            &slideshow_settings_path,
            &slideshow_settings_source,
            "InteractionFacade::consume_rect",
            "Slideshow settings sidebar must use InteractionFacade to consume pointer events over the panel and tab.",
            &mut violations,
        );
        Self::require_contains(
            &slideshow_modal_path,
            &slideshow_source,
            "InteractionFacade::scope",
            "Slideshow content must be rendered through InteractionFacade.",
            &mut violations,
        );
        Self::lint_foreground_area_contract(workspace_root, &mut violations);

        violations
    }

    fn lint_foreground_area_contract(workspace_root: &Path, violations: &mut Vec<Violation>) {
        let ui_src = workspace_root.join("crates/katana-ui/src");
        let mut files = Vec::new();
        Self::collect_rust_files(&ui_src, &mut files);

        for file in files {
            let Ok(source) = std::fs::read_to_string(&file) else {
                continue;
            };
            if !Self::has_foreground_area(&source) || Self::is_guarded_foreground_area(&source) {
                continue;
            }

            violations.push(Violation {
                file,
                line: Self::line_number_of(&source, "egui::Order::Foreground"),
                column: 1,
                message: "Foreground egui::Area must consume pointer and hover events through InteractionFacade or an explicit full-screen blocker.".to_string(),
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

    fn require_contains(
        file: &Path,
        source: &str,
        token: &str,
        message: &str,
        violations: &mut Vec<Violation>,
    ) {
        if source.contains(token) {
            return;
        }

        violations.push(Violation {
            file: file.to_path_buf(),
            line: 1,
            column: 1,
            message: message.to_string(),
        });
    }
}

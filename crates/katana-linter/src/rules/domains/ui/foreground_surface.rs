use crate::Violation;
use std::path::Path;

mod scans;

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
        let modal_widget_path = workspace_root.join("crates/katana-ui/src/widgets/modal/ui.rs");

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
        let Ok(modal_widget_source) = std::fs::read_to_string(&modal_widget_path) else {
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
            &shell_path,
            &shell_source,
            "move_modal",
            "Foreground surface detection must include move modal state.",
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
        Self::require_contains(
            &modal_widget_path,
            &modal_widget_source,
            "InteractionFacade::register_hover_blocker",
            "Katana Modal must register its window rect with InteractionFacade.",
            &mut violations,
        );
        scans::ForegroundSurfaceScanOps::lint(workspace_root, &mut violations);

        violations
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

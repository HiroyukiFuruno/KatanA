use crate::Violation;
use std::path::Path;

pub struct ForegroundSurfaceOps;

impl ForegroundSurfaceOps {
    pub fn lint(workspace_root: &Path) -> Vec<Violation> {
        let mut violations = Vec::new();
        let shell_path = workspace_root.join("crates/katana-ui/src/shell/mod.rs");
        let central_content_path =
            workspace_root.join("crates/katana-ui/src/views/app_frame/central_content.rs");
        let slideshow_modal_path =
            workspace_root.join("crates/katana-ui/src/preview_pane/slideshow/modal.rs");

        let Ok(shell_source) = std::fs::read_to_string(&shell_path) else {
            return violations;
        };
        let Ok(central_content_source) = std::fs::read_to_string(&central_content_path) else {
            return violations;
        };
        let Ok(slideshow_source) = std::fs::read_to_string(&slideshow_modal_path) else {
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

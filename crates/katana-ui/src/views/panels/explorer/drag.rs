use crate::icon::IconSize;
use crate::shell::TREE_FONT_SIZE;
use eframe::egui;

pub(crate) struct ExplorerDragUi;

impl ExplorerDragUi {
    const GHOST_MAX_WIDTH: f32 = 280.0;
    const GHOST_ITEM_SPACING: f32 = 6.0;
    const HINT_PADDING_X: f32 = 8.0;
    const HINT_PADDING_Y: f32 = 4.0;
    const HINT_OFFSET_X: f32 = 8.0;
    const HINT_OFFSET_Y: f32 = 2.0;
    const HINT_BG_GAMMA: f32 = 0.92;
    const HINT_CORNER_RADIUS: f32 = 4.0;

    pub(crate) fn is_pointer_over_rect(ui: &egui::Ui, rect: egui::Rect) -> bool {
        ui.input(|i| i.pointer.hover_pos().is_some_and(|pos| rect.contains(pos)))
    }

    pub(crate) fn render_drag_ghost(
        ui: &mut egui::Ui,
        source_path: &std::path::Path,
        is_directory: bool,
        source_rect: egui::Rect,
        ws_root: Option<&std::path::Path>,
        icon: crate::Icon,
    ) {
        let Some(pointer_pos) = ui.ctx().pointer_interact_pos() else {
            return;
        };

        let press_origin = ui
            .input(|i| i.pointer.press_origin())
            .unwrap_or(pointer_pos);
        let drag_offset = pointer_pos - press_origin;
        let ghost_rect = source_rect.translate(drag_offset);

        let drag_id = crate::shell_logic::ShellLogicOps::hash_str(&source_path.to_string_lossy());
        let text = Self::relative_display_path(source_path, ws_root, is_directory);

        egui::Area::new(egui::Id::new("explorer_drag_ghost").with(drag_id))
            .fixed_pos(ghost_rect.min)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                ui.set_max_width(Self::GHOST_MAX_WIDTH);
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        ui.spacing_mut().item_spacing.x = Self::GHOST_ITEM_SPACING;
                        ui.add(icon.ui_image(ui, IconSize::Small));
                        ui.label(
                            egui::RichText::new(text)
                                .size(TREE_FONT_SIZE)
                                .color(ui.visuals().weak_text_color()),
                        );
                    })
                    .show(ui);
            });
    }

    pub(crate) fn render_drop_target_hint(
        ui: &mut egui::Ui,
        target_rect: egui::Rect,
        target_path: &std::path::Path,
        ws_root: Option<&std::path::Path>,
        is_directory: bool,
    ) {
        let text = format!(
            "Move to {}",
            Self::relative_display_path(target_path, ws_root, is_directory),
        );
        let font = egui::FontId::proportional(TREE_FONT_SIZE);
        let hint_color = ui.visuals().selection.stroke.color;
        let galley = ui
            .painter()
            .layout_no_wrap(text.clone(), font.clone(), hint_color);
        let padding = egui::vec2(Self::HINT_PADDING_X, Self::HINT_PADDING_Y);
        let hint_pos =
            target_rect.left_top() + egui::vec2(Self::HINT_OFFSET_X, Self::HINT_OFFSET_Y);
        let hint_rect = egui::Rect::from_min_size(hint_pos, galley.size() + padding * 2.0);
        let bg = ui
            .visuals()
            .selection
            .bg_fill
            .gamma_multiply(Self::HINT_BG_GAMMA);
        ui.painter()
            .rect_filled(hint_rect, Self::HINT_CORNER_RADIUS, bg);
        ui.painter().text(
            hint_pos + egui::vec2(padding.x, padding.y / 2.0),
            egui::Align2::LEFT_TOP,
            text,
            font,
            hint_color,
        );
    }

    pub(crate) fn resolve_drop_target_dir(
        source_path: &std::path::Path,
        hovered_path: &std::path::Path,
        hovered_is_directory: bool,
    ) -> Option<std::path::PathBuf> {
        let target_dir = if hovered_is_directory {
            hovered_path
        } else {
            hovered_path.parent()?
        };

        if source_path == target_dir {
            return None;
        }

        if source_path.parent() == Some(target_dir) {
            return None;
        }

        if target_dir.starts_with(source_path) {
            return None;
        }

        Some(target_dir.to_path_buf())
    }

    pub(crate) fn resolve_drop_target_dir_for_empty_tree_area(
        source_path: &std::path::Path,
        ws_root: &std::path::Path,
    ) -> Option<std::path::PathBuf> {
        if source_path == ws_root {
            return None;
        }

        let parent = source_path.parent()?;
        if parent == ws_root {
            return None;
        }

        Some(parent.to_path_buf())
    }

    pub(crate) fn relative_display_path(
        path: &std::path::Path,
        ws_root: Option<&std::path::Path>,
        is_directory: bool,
    ) -> String {
        let mut display = crate::shell_logic::ShellLogicOps::relative_full_path(path, ws_root);

        if display.is_empty() {
            display = ws_root
                .and_then(|root| root.file_name())
                .and_then(std::ffi::OsStr::to_str)
                .map(str::to_string)
                .unwrap_or_else(|| "workspace".to_string());
        }

        if is_directory && !display.ends_with(std::path::MAIN_SEPARATOR) {
            display.push(std::path::MAIN_SEPARATOR);
        }

        display
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn relative_display_path_uses_workspace_relative_form_for_file() {
        let root = Path::new("workspace_root");
        let source = root.join("docs").join("readme.md");
        let expected = source
            .strip_prefix(root)
            .map_or_else(|_| "".into(), |v| v.display().to_string());
        assert_eq!(
            super::ExplorerDragUi::relative_display_path(&source, Some(root), false),
            expected
        );
    }

    #[test]
    fn relative_display_path_appends_slash_for_directories() {
        let root = Path::new("workspace_root");
        let source = root.join("notes");
        assert_eq!(
            super::ExplorerDragUi::relative_display_path(&source, Some(root), true),
            format!("notes{}", std::path::MAIN_SEPARATOR)
        );
    }

    #[test]
    fn relative_display_path_falls_back_to_workspace_name_when_root_is_target() {
        let root = Path::new("workspace_root");
        assert_eq!(
            super::ExplorerDragUi::relative_display_path(root, Some(root), true),
            format!("workspace_root{}", std::path::MAIN_SEPARATOR)
        );
    }

    #[test]
    fn relative_display_path_keeps_absolute_without_root() {
        let path = Path::new("tmp/file.md");
        assert_eq!(
            super::ExplorerDragUi::relative_display_path(path, None, false),
            "tmp/file.md"
        );
    }

    #[test]
    fn relative_display_path_does_not_generate_dot_prefix() {
        let root = Path::new("workspace_root");
        let source = root.join("docs").join("readme.md");
        let rel = super::ExplorerDragUi::relative_display_path(&source, Some(root), false);
        assert!(rel == "docs/readme.md");
    }

    #[test]
    fn resolve_drop_target_dir_for_file_to_sibling_file_parent() {
        let source = Path::new("workspace_root").join("notes").join("old.md");
        let hovered = Path::new("workspace_root").join("notes").join("new.md");

        assert!(super::ExplorerDragUi::resolve_drop_target_dir(&source, &hovered, false).is_none());
    }

    #[test]
    fn resolve_drop_target_dir_for_file_to_sibling_directory() {
        let source = Path::new("workspace_root").join("notes").join("old.md");
        let hovered = Path::new("workspace_root").join("docs");

        assert_eq!(
            super::ExplorerDragUi::resolve_drop_target_dir(&source, &hovered, true),
            Some(hovered)
        );
    }

    #[test]
    fn resolve_drop_target_dir_for_directory_into_its_parent_is_none() {
        let source = Path::new("workspace_root").join("notes");
        let hovered = Path::new("workspace_root");

        assert!(super::ExplorerDragUi::resolve_drop_target_dir(&source, &hovered, true).is_none());
    }

    #[test]
    fn resolve_drop_target_dir_for_empty_tree_area_uses_parent_directory() {
        let root = Path::new("workspace_root");
        let source = root.join("docs").join("note.md");

        assert_eq!(
            super::ExplorerDragUi::resolve_drop_target_dir_for_empty_tree_area(&source, root),
            Some(root.join("docs"))
        );
    }

    #[test]
    fn resolve_drop_target_dir_for_empty_tree_area_hides_root_level_target() {
        let root = Path::new("workspace_root");
        let source = root.join("notes.md");

        assert!(
            super::ExplorerDragUi::resolve_drop_target_dir_for_empty_tree_area(&source, root)
                .is_none()
        );
    }
}

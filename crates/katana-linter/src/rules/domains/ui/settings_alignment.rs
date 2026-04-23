use crate::Violation;

pub struct SettingsAlignmentOps;

impl SettingsAlignmentOps {
    pub fn check_settings_alignment(workspace_root: &std::path::Path) -> Vec<Violation> {
        let mut errors = Vec::new();

        let properties_path =
            workspace_root.join("crates/katana-ui/src/settings/tabs/linter/properties.rs");
        let Ok(content_props) = std::fs::read_to_string(&properties_path) else {
            return errors;
        };

        let mut in_string_array = false;
        for (i, line) in content_props.lines().enumerate() {
            if line.contains("fn render_string_array_property") {
                in_string_array = true;
            } else if line.contains("fn render_singleline_property") {
                in_string_array = false;
            }

            if in_string_array && line.contains("AlignCenter::new") {
                errors.push(Violation {
                    file: properties_path.clone(),
                    line: i + 1,
                    column: 1,
                    message: "Do not use `AlignCenter` for array properties, as it causes a layout breakdown (nested expanding constraints). Use `Accordion` instead.".to_string(),
                });
            }

            if line.contains("LabeledToggle") {
                errors.push(Violation {
                    file: properties_path.clone(),
                    line: i + 1,
                    column: 1,
                    message: "Do not use `LabeledToggle` in properties.rs. Use `AlignCenter::new().interactive(true)` with `ToggleOps::switch` to ensure perfect right-alignment and hover effects.".to_string(),
                });
            }
        }

        let layout_path = workspace_root.join("crates/katana-ui/src/settings/tabs/layout.rs");
        let Ok(content_layout) = std::fs::read_to_string(&layout_path) else {
            return errors;
        };

        for (i, line) in content_layout.lines().enumerate() {
            if line.contains("egui::Layout::top_down(egui::Align::Min)") {
                errors.push(Violation {
                    file: layout_path.clone(),
                    line: i + 1,
                    column: 1,
                    message: "Do not use `egui::Align::Min` for top_down layouts in settings, as it breaks right-alignment of list items. Use `egui::Align::Max`.".to_string(),
                });
            }
        }

        errors
    }
}

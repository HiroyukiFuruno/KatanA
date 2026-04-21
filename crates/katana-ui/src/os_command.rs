use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsCommand {
    pub mac: String,
    pub windows: String,
    pub linux: String,
}

impl OsCommand {
    pub fn display(&self) -> &str {
        #[cfg(target_os = "macos")]
        {
            &self.mac
        }
        #[cfg(target_os = "windows")]
        {
            &self.windows
        }
        #[cfg(target_os = "linux")]
        {
            &self.linux
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            &self.linux
        }
    }
}

pub struct ShortcutFacade;

pub type OsCommandOps = ShortcutFacade;

impl ShortcutFacade {
    fn get_dictionary() -> std::collections::HashMap<String, OsCommand> {
        let json = include_str!("../resources/os_commands.json");
        serde_json::from_str(json).unwrap_or_default()
    }

    pub fn get(key: &str) -> String {
        let dictionary = Self::get_dictionary();
        if let Some(cmd) = dictionary.get(key) {
            cmd.display().to_string()
        } else {
            key.to_string()
        }
    }

    pub fn replace_in_text(text: &str) -> String {
        let dictionary = Self::get_dictionary();
        let mut result = text.to_string();
        for (k, v) in dictionary {
            let placeholder = format!("{{{{os_cmd:{}}}}}", k);
            result = result.replace(&placeholder, v.display());
        }
        result
    }

    /* WHY: Renders a string with `{{os_cmd:KEY}}` using the ShortcutWidget for keys.
    This version wraps everything in a horizontal_wrapped layout, suitable for standalone blocks. */
    pub fn render_mixed(ui: &mut egui::Ui, text: &str) {
        ui.horizontal_wrapped(|ui| {
            Self::render_inline(ui, text);
        });
    }

    /* WHY: Renders placeholders directly into the current layout. Suitable for use inside
    existing text rendering pipelines (like egui_commonmark) where the layout is already managed.
    Returns true if it handled at least one shortcut placeholder, otherwise false. */
    pub fn render_inline(ui: &mut egui::Ui, text: &str) -> bool {
        if !text.contains("{{os_cmd:") {
            return false;
        }

        let mut remaining = text;
        let dictionary = Self::get_dictionary();
        let mut handled = false;

        /* WHY: Use a scope to allow local spacing adjustments without affecting the rest of the UI.
        We want the text surrounding the badges to be tight (0 spacing), while badges themselves
        maintain their internal KEY_CAP_SEP. */
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            const OS_CMD_PREFIX: &str = "{{os_cmd:";
            while let Some(start_idx) = remaining.find(OS_CMD_PREFIX) {
                let prefix = &remaining[..start_idx];
                if !prefix.is_empty() {
                    ui.label(prefix);
                }

                let after_prefix = &remaining[start_idx + OS_CMD_PREFIX.len()..];
                if let Some(end_idx) = after_prefix.find("}}") {
                    let key = &after_prefix[..end_idx];
                    let raw_shortcut = if let Some(cmd) = dictionary.get(key) {
                        cmd.display().to_string()
                    } else {
                        key.to_string()
                    };

                    if raw_shortcut.is_empty() {
                        let i18n = crate::i18n::I18nOps::get();
                        ui.label(egui::RichText::new(&i18n.settings.shortcuts.none).weak());
                    } else {
                        /* WHY: ShortcutWidget handles its own internal item_spacing.
                        We use a nested scope/ui so it doesn't permanently change our 0.0 spacing. */
                        crate::widgets::ShortcutWidget::new(&raw_shortcut).ui(ui);
                    }

                    remaining = &after_prefix[end_idx + 2..];
                    handled = true;
                } else {
                    ui.label(OS_CMD_PREFIX);
                    remaining = after_prefix;
                }
            }

            if !remaining.is_empty() {
                ui.label(remaining);
            }
        });

        handled
    }

    /* WHY: Renders a shortcut string in a fixed-width row layout, with the label on the left
    and the shortcut badge right-aligned, mirroring the appearance in the Shortcut Settings panel. */
    pub fn render_shortcut_row(ui: &mut egui::Ui, text: &str) {
        let regex = regex::Regex::new(r"\{\{os_cmd:([^}]+)\}\}").unwrap();

        let mut key = String::new();
        let mut label_text = text.to_string();

        if let Some(cap) = regex.captures(text) {
            key = cap.get(1).unwrap().as_str().to_string();
            let matched = cap.get(0).unwrap().as_str();
            label_text = label_text.replace(matched, "");
            /* WHY: Clean up leading colons or hyphens and whitespace */
            label_text = label_text
                .trim_start_matches(|c: char| c == ':' || c == '-' || c.is_whitespace())
                .to_string();
            label_text = label_text
                .trim_end_matches(|c: char| c == ':' || c == '-' || c.is_whitespace())
                .to_string();
        }

        let raw_shortcut = if !key.is_empty() {
            let dictionary = Self::get_dictionary();
            if let Some(cmd) = dictionary.get(&key) {
                cmd.display().to_string()
            } else {
                key.clone()
            }
        } else {
            String::new()
        };

        const MIN_ROW_WIDTH: f32 = 200.0;
        let available_w = ui.available_width().max(MIN_ROW_WIDTH);

        ui.allocate_ui_with_layout(
            egui::vec2(available_w, 0.0), // 0.0 means natural height shrink wrap
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(label_text);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !raw_shortcut.is_empty() {
                        crate::widgets::ShortcutWidget::new(&raw_shortcut).ui(ui);
                    } else {
                        let i18n = crate::i18n::I18nOps::get();
                        ui.label(egui::RichText::new(&i18n.settings.shortcuts.none).weak());
                    }
                });
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_command_ops_get() {
        /* WHY: verify it doesn't return the raw key for a known command */
        let result = OsCommandOps::get("save_document");
        assert_ne!(result, "save_document");

        /* WHY: Unkown key returns the key itself */
        let missing = OsCommandOps::get("missing_key_123");
        assert_eq!(missing, "missing_key_123");
    }

    #[test]
    fn test_replace_in_text() {
        let result = OsCommandOps::replace_in_text("Press {{os_cmd:save_document}} to save.");
        assert!(!result.contains("{{os_cmd:save_document}}"));
        assert!(result.starts_with("Press "));
        assert!(result.ends_with(" to save."));
    }

    #[test]
    fn test_render_shortcut_row() {
        let mut harness = egui_kittest::Harness::builder()
            .with_size(eframe::egui::vec2(400.0, 300.0))
            .build(|ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    OsCommandOps::render_shortcut_row(ui, "Label: {{os_cmd:save_document}}");
                    OsCommandOps::render_shortcut_row(ui, "Missing {{os_cmd:unknown_123}}");
                    OsCommandOps::render_shortcut_row(ui, "No OS Cmd");
                    OsCommandOps::render_shortcut_row(ui, "Empty shortcut: {{os_cmd:}}");
                });
            });
        harness.run();
    }

    #[test]
    fn test_render() {
        let mut harness = egui_kittest::Harness::builder()
            .with_size(eframe::egui::vec2(400.0, 300.0))
            .build(|ctx| {
                eframe::egui::CentralPanel::default().show(ctx, |ui| {
                    OsCommandOps::render_mixed(ui, "Press {{os_cmd:save_document}} to save.");
                    OsCommandOps::render_mixed(ui, "Missing {{os_cmd:unknown}}.");
                    OsCommandOps::render_mixed(ui, "No shortcut here.");
                    OsCommandOps::render_mixed(ui, "Empty {{os_cmd:}}");
                    OsCommandOps::render_mixed(ui, "{{os_cmd:just_shortcut}}");
                    OsCommandOps::render_mixed(ui, "Unclosed {{os_cmd:unclosed_template");
                });
            });
        harness.run();
    }
}

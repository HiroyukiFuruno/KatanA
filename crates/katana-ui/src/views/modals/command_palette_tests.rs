#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::command_palette::CommandPaletteResultKind;

    fn action_result(action: AppAction) -> CommandPaletteResult {
        CommandPaletteResult {
            id: "test".to_string(),
            label: "Test".to_string(),
            secondary_label: None,
            shortcut: None,
            score: 1.0,
            kind: CommandPaletteResultKind::Action,
            execute_payload: CommandPaletteExecutePayload::DispatchAppAction(action),
        }
    }

    #[test]
    fn normal_palette_allows_image_file_command() {
        assert!(command_result_visible_in_normal_palette(&action_result(
            AppAction::IngestImageFile
        )));
    }

    #[test]
    fn normal_palette_allows_clipboard_image_command() {
        assert!(command_result_visible_in_normal_palette(&action_result(
            AppAction::IngestClipboardImage
        )));
    }

    #[test]
    fn normal_palette_hides_unrelated_commands() {
        assert!(!command_result_visible_in_normal_palette(&action_result(
            AppAction::ToggleSettings
        )));
    }
}

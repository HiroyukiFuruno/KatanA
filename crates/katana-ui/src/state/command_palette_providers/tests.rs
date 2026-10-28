use super::{AppCommandProvider, DocumentOperationSettings, ViewMode};
use crate::app_state::AppAction;
use crate::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteProvider, CommandPaletteResultKind,
};

#[test]
fn app_command_provider_returns_system_actions() {
    /* WHY: 文書操作設定を公開せず、候補の契約を同じcrate内で検証する。 */
    let provider = AppCommandProvider {
        operation_settings: DocumentOperationSettings::evaluate(None, None),
        current_mode: ViewMode::PreviewOnly,
    };

    let results = provider.search("", None, None);
    assert!(!results.is_empty());
    assert_eq!(results[0].kind, CommandPaletteResultKind::RecentOrCommon);

    let results = provider.search("Settings", None, None);
    assert!(!results.is_empty());
    assert_eq!(results[0].kind, CommandPaletteResultKind::Action);
    assert_eq!(results[0].label, "Settings");

    assert!(matches!(
        &results[0].execute_payload,
        CommandPaletteExecutePayload::DispatchAppAction(AppAction::ToggleSettings)
    ));
}

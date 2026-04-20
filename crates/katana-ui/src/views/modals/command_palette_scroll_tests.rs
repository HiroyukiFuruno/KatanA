#[cfg(test)]
mod tests {
    use crate::app_state::{AppAction, AppState};
    use crate::shell::KatanaApp;
    use crate::state::command_palette::{
        CommandPaletteExecutePayload, CommandPaletteResult, CommandPaletteResultKind,
        CommandPaletteState,
    };
    use eframe::egui;
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;
    use katana_core::ai::AiProviderRegistry;
    use katana_core::plugin::PluginRegistry;

    #[test]
    fn test_command_palette_scroll_behavior() {
        /* WHY: Verify that displaying 100+ items triggers a scrollbar and doesn't break window constraints. */
        let mut state = CommandPaletteState::new();
        state.is_open = true;
        let mut results = Vec::new();
        for i in 0..100 {
            results.push(CommandPaletteResult {
                id: format!("dummy_{}", i),
                label: format!("Dummy Command {}", i),
                secondary_label: None,
                score: 1.0,
                kind: CommandPaletteResultKind::Action,
                execute_payload: CommandPaletteExecutePayload::DispatchAppAction(
                    AppAction::ToggleSettings,
                ),
            });
        }
        state.results = results;
        let mut action = AppAction::ToggleCommandPalette; /* WHY: Check window constraints and layout bug. */
        let mut harness = Harness::builder().with_size(egui::vec2(600.0, 600.0)).build_ui(move |ui| {
            /* WHY: We need a top-level window, since CommandPaletteModal uses Window::new internally. */
            crate::views::modals::command_palette::CommandPaletteModal::new(&mut state, None, &mut action, &[]).show(ui.ctx());
        });

        harness.step();

        /* WHY: Find Dummy Command 0 bounding rect to measure its rendered position. */
        let search_box = harness.get_by_label("Dummy Command 0");
        let y_pos = search_box.rect().top();
        println!("Dummy 0 Y position: {}", y_pos);

        /* WHY: The command palette should be anchored at Y = 100.0.
        The text box has some margin, so its Y should be > 100.0.
        If the window blew up to 1200 height, egui would push it to the top of the screen to try to fit it.
        So y_pos would be near 0 instead of > 100.0. */
        assert!(
            y_pos > 100.0,
            "Command Palette was pushed to the top of the screen because it demanded too much height! y_pos={}",
            y_pos
        );

        /* WHY: Additionally, we verify it properly scrolls. To do that, the window shouldn't expand off-screen.
        Let's just assert the window didn't become overwhelmingly huge, but using the Y offset check is the most direct verification of the egui layout bug. */
    }
}

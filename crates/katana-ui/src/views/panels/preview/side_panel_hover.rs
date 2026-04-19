use super::side_panel_types::{HoverDelay, PendingPanel};
use super::side_panels::{HOVER_SWITCH_DELAY, PreviewSidePanels};
use eframe::egui;

impl<'a> PreviewSidePanels<'a> {
    /// Centralized hover logic with delay to prevent accidental panel switches.
    ///
    /// WHY: Always wait for the hover delay to elapse before opening or switching
    /// a popup. This prevents annoying immediate flashes of popups when moving the
    /// mouse across the sidebar, ensuring the user deliberately rested on the button.
    pub(super) fn handle_popup_hover(
        &mut self,
        ui: &mut egui::Ui,
        export_hovered: bool,
        story_hovered: bool,
        tools_hovered: bool,
    ) {
        let hover_id = egui::Id::new("preview_side_panel_hover_delay");
        let mut delay: HoverDelay = ui.ctx().data(|d| d.get_temp(hover_id).unwrap_or_default());

        let hovered_panel = if export_hovered {
            PendingPanel::Export
        } else if story_hovered {
            PendingPanel::Story
        } else if tools_hovered {
            PendingPanel::Tools
        } else {
            PendingPanel::None
        };

        let any_open = self.app.state.layout.show_export_panel
            || self.app.state.layout.show_story_panel
            || self.app.state.layout.show_tools_panel;

        if !any_open && hovered_panel != PendingPanel::None {
            self.activate_panel(hovered_panel);
            delay = HoverDelay::default();
        } else {
            self.transition_hover_delay(ui, &mut delay, hovered_panel);
        }

        ui.ctx().data_mut(|d| d.insert_temp(hover_id, delay));
    }

    fn transition_hover_delay(
        &mut self,
        ui: &egui::Ui,
        delay: &mut HoverDelay,
        hovered_panel: PendingPanel,
    ) {
        /* WHY: If mouse is not on any button, reset the delay to cancel any pending activation. */
        if hovered_panel == PendingPanel::None {
            *delay = HoverDelay::default();
            return;
        }

        let already_showing = match hovered_panel {
            PendingPanel::Export => self.app.state.layout.show_export_panel,
            PendingPanel::Story => self.app.state.layout.show_story_panel,
            PendingPanel::Tools => self.app.state.layout.show_tools_panel,
            PendingPanel::None => false,
        };

        /* WHY: If the panel they are hovering is ALREADY showing, nothing to switch to. */
        if already_showing {
            *delay = HoverDelay::default();
            return;
        }

        let now = ui.input(|i| i.time);

        if delay.pending == hovered_panel {
            /* WHY: We have been hovering this button. Has enough time passed? */
            if now - delay.start_time >= HOVER_SWITCH_DELAY {
                self.activate_panel(hovered_panel);
                *delay = HoverDelay::default();
            } else {
                /* WHY: Not enough time passed yet, request a repaint for when it will be due. */
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_secs_f64(
                        HOVER_SWITCH_DELAY - (now - delay.start_time),
                    ));
            }
        } else {
            /* WHY: Just started hovering a different button. Mark the start time. */
            *delay = HoverDelay {
                pending: hovered_panel,
                start_time: now,
            };
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs_f64(HOVER_SWITCH_DELAY));
        }
    }

    /// Activate the given popup panel and deactivate all others.
    fn activate_panel(&mut self, panel: PendingPanel) {
        self.app.state.layout.show_export_panel = panel == PendingPanel::Export;
        self.app.state.layout.show_story_panel = panel == PendingPanel::Story;
        self.app.state.layout.show_tools_panel = panel == PendingPanel::Tools;
        self.app.state.layout.show_toc = false;
    }
}

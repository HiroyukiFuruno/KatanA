pub use super::side_panel_types::*;
use eframe::egui;

pub(super) const PREVIEW_SIDE_BAR_WIDTH: f32 = 32.0;
pub(super) const PREVIEW_SIDE_BAR_MARGIN: f32 = 2.0;
pub(super) const PREVIEW_SIDE_BAR_SPACING: f32 = 4.0;
pub(super) const LIGHT_MODE_ICON_BG: u8 = 245;
pub const PANEL_WIDTH: f32 = 260.0;
pub const SELECTABLE_H: f32 = 20.0;
pub(super) const PANEL_HEAD_SPACE: f32 = 8.0;
pub(super) const PANEL_ITEM_SPACE: f32 = 4.0;
pub(super) const PANEL_HOVER_MARGIN: f32 = 12.0;
pub(super) const PANEL_ANIM_SPEED: f32 = 0.15;
pub(super) const POPUP_ROUNDING: f32 = 8.0;
pub(super) const POPUP_PADDING: i8 = 0;
pub(super) const POPUP_SHADOW_ALPHA: u8 = 48;
pub(super) const POPUP_GAP: f32 = 2.0;

/* WHY: Prevents accidental panel switches while the pointer crosses buttons. */
pub(super) const HOVER_SWITCH_DELAY: f64 = 0.25;

impl<'a> PreviewSidePanels<'a> {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let (
            toc_available,
            export_available,
            story_available,
            tools_available,
            slideshow_available,
        ) = {
            let path = self
                .app
                .state
                .active_document()
                .map(|document| document.path.as_path());
            (
                super::PreviewMenuAvailability::for_path(path, super::PreviewMenu::Toc),
                super::PreviewMenuAvailability::for_path(path, super::PreviewMenu::Export),
                super::PreviewMenuAvailability::for_path(path, super::PreviewMenu::Story),
                super::PreviewMenuAvailability::for_path(path, super::PreviewMenu::Tools),
                super::PreviewMenuAvailability::for_path(path, super::PreviewMenu::Slideshow),
            )
        };
        if !toc_available {
            self.app.state.layout.show_toc = false;
        }
        if !export_available {
            self.app.state.layout.show_export_panel = false;
        }
        if !story_available {
            self.app.state.layout.show_story_panel = false;
        }
        if !tools_available {
            self.app.state.layout.show_tools_panel = false;
        }
        if !slideshow_available {
            if self.app.state.layout.show_slideshow
                && !self.app.state.layout.was_os_fullscreen_before_slideshow
            {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            self.app.state.layout.show_slideshow = false;
        }
        self.render_sidebar(ui);
        self.render_export(ui);
        self.render_story(ui);
        self.render_tools(ui);
        self.render_toc(ui);
    }
}

#[cfg(test)]
#[path = "side_panels/slideshow_tests.rs"]
mod slideshow_tests;

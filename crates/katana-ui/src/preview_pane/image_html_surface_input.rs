use super::{HtmlBrowserSurface, frame_position, frame_scroll_delta};
use eframe::egui::{self, Vec2};
use katana_document_viewer::browser_session::HtmlBrowserInput;

impl HtmlBrowserSurface {
    pub(super) fn forward_input(
        &mut self,
        ui: &egui::Ui,
        rect: egui::Rect,
        response: &egui::Response,
    ) {
        if self.frame.is_none() {
            return;
        }

        let pointer_position = ui.input(|input| input.pointer.hover_pos());
        self.forward_focus(ui, response);
        self.forward_pointer_move(rect, response, pointer_position);
        self.forward_pointer_down(rect, ui, response, pointer_position);
        self.forward_pointer_up(rect, ui, response, pointer_position);
        self.forward_scroll(ui, rect, response);
        self.forward_keyboard_events(ui);
    }

    fn forward_pointer_move(
        &mut self,
        rect: egui::Rect,
        response: &egui::Response,
        pointer_position: Option<egui::Pos2>,
    ) {
        let Some(position) = pointer_position else {
            return;
        };
        if !response.hovered() || self.last_pointer_position == Some(position) {
            return;
        }

        let Some(viewport) = self.frame.as_ref().map(|frame| frame.viewport) else {
            return;
        };
        self.last_pointer_position = Some(position);
        let position = frame_position(rect, position, viewport);
        self.dispatch(HtmlBrowserInput::PointerMove {
            x: position.x,
            y: position.y,
        });
    }

    fn forward_pointer_down(
        &mut self,
        rect: egui::Rect,
        ui: &egui::Ui,
        response: &egui::Response,
        pointer_position: Option<egui::Pos2>,
    ) {
        let Some(position) = pointer_position else {
            return;
        };
        if !response.hovered() || !ui.input(|input| input.pointer.primary_pressed()) {
            return;
        }

        let Some(viewport) = self.frame.as_ref().map(|frame| frame.viewport) else {
            return;
        };
        let position = frame_position(rect, position, viewport);
        self.primary_pointer_pressed = true;
        self.dispatch(HtmlBrowserInput::PointerDown {
            x: position.x,
            y: position.y,
            button: 0,
        });
    }

    fn forward_pointer_up(
        &mut self,
        rect: egui::Rect,
        ui: &egui::Ui,
        response: &egui::Response,
        pointer_position: Option<egui::Pos2>,
    ) {
        if !ui.input(|input| input.pointer.primary_released()) {
            return;
        }

        let Some(viewport) = self.frame.as_ref().map(|frame| frame.viewport) else {
            return;
        };
        let Some(input) = self.take_pointer_release(rect, pointer_position, viewport) else {
            return;
        };
        self.dispatch(input);
        if response.hovered() {
            self.apply_focus_change(Some(true));
        }
    }

    fn take_pointer_release(
        &mut self,
        rect: egui::Rect,
        pointer_position: Option<egui::Pos2>,
        viewport: katana_document_viewer::browser_session::HtmlBrowserViewport,
    ) -> Option<HtmlBrowserInput> {
        if !self.primary_pointer_pressed {
            return None;
        }
        self.primary_pointer_pressed = false;
        let position = pointer_position
            .or(self.last_pointer_position)
            .map(|position| frame_position(rect, position, viewport))
            .unwrap_or(egui::pos2(-1.0, -1.0));
        Some(HtmlBrowserInput::PointerUp {
            x: position.x,
            y: position.y,
            button: 0,
        })
    }

    fn forward_focus(&mut self, ui: &egui::Ui, response: &egui::Response) {
        let primary_pressed = ui.input(|input| input.pointer.primary_pressed());
        let focus = (self.focused && primary_pressed && !response.hovered()).then_some(false);
        self.apply_focus_change(focus);
    }

    fn apply_focus_change(&mut self, focus: Option<bool>) {
        let Some(focused) = focus else {
            return;
        };
        self.focused = focused;
        self.dispatch(HtmlBrowserInput::Focus { focused });
    }

    fn forward_scroll(&mut self, ui: &egui::Ui, rect: egui::Rect, response: &egui::Response) {
        if !response.hovered() {
            return;
        }

        let delta = ui.input(|input| {
            if has_nonzero_wheel_event(&input.raw.events) {
                input.smooth_scroll_delta
            } else {
                Vec2::ZERO
            }
        });
        if delta == Vec2::ZERO {
            return;
        }

        let Some(viewport) = self.frame.as_ref().map(|frame| frame.viewport) else {
            return;
        };
        let delta = browser_scroll_delta(rect, delta, viewport);
        self.dispatch(HtmlBrowserInput::Scroll {
            delta_x: delta.x,
            delta_y: delta.y,
        });
    }

    pub(super) fn dispatch(&mut self, input: HtmlBrowserInput) {
        let Some(adapter) = &self.adapter else {
            return;
        };
        match adapter.dispatch_input(input) {
            Ok(()) => self.await_frame(),
            Err(error) => self.record_adapter_error("input", None, error),
        }
    }
}

fn browser_scroll_delta(
    rect: egui::Rect,
    ui_delta: Vec2,
    viewport: katana_document_viewer::browser_session::HtmlBrowserViewport,
) -> Vec2 {
    frame_scroll_delta(rect, -ui_delta, viewport)
}

fn has_nonzero_wheel_event(events: &[egui::Event]) -> bool {
    events.iter().any(|event| {
        matches!(
            event,
            egui::Event::MouseWheel { delta, .. } if *delta != Vec2::ZERO
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preview_pane::image_html_surface::HtmlBrowserSurface;

    #[test]
    fn keyboard_input_requests_polling_for_the_resulting_browser_frame() -> Result<(), String> {
        let source = katana_document_viewer::browser_session::HtmlBrowserSource::new(
            "<input autofocus>",
            "https://example.test/index.html",
        )
        .map_err(|error| error.to_string())?;
        let mut surface = HtmlBrowserSurface::start(source);
        surface.frame_update_deadline = None;

        surface.dispatch(HtmlBrowserInput::KeyDown {
            key: "Enter".to_string(),
        });

        assert!(surface.frame_update_deadline.is_some());
        Ok(())
    }

    #[test]
    fn pointer_release_is_forwarded_outside_the_surface_and_clears_capture() {
        let mut surface = HtmlBrowserSurface::failed("test".to_string());
        surface.primary_pointer_pressed = true;
        surface.last_pointer_position = Some(egui::pos2(20.0, 30.0));
        let viewport =
            katana_document_viewer::browser_session::HtmlBrowserViewport::new(200, 100, 2.0)
                .unwrap();
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(100.0, 50.0));

        assert_eq!(
            surface.take_pointer_release(rect, None, viewport),
            Some(HtmlBrowserInput::PointerUp {
                x: 20.0,
                y: 20.0,
                button: 0,
            })
        );
        assert!(!surface.primary_pointer_pressed);
        assert_eq!(surface.take_pointer_release(rect, None, viewport), None);
    }

    #[test]
    fn focus_change_clears_browser_focus_without_starting_a_fallback() {
        let mut surface = HtmlBrowserSurface::failed("test".to_string());
        surface.focused = true;

        surface.apply_focus_change(Some(false));
        surface.apply_focus_change(None);

        assert!(!surface.focused);
        assert!(surface.adapter.is_none());
    }

    #[test]
    fn downward_egui_wheel_delta_maps_to_positive_browser_scroll() {
        let viewport =
            katana_document_viewer::browser_session::HtmlBrowserViewport::new(200, 100, 2.0)
                .unwrap();
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(100.0, 50.0));

        assert_eq!(
            browser_scroll_delta(rect, egui::vec2(0.0, -5.0), viewport),
            egui::vec2(0.0, 10.0)
        );
    }

    #[test]
    fn scroll_forwarding_ignores_smoothing_tail_and_zero_end_phase() {
        let move_event = egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, -5.0),
            modifiers: egui::Modifiers::NONE,
            phase: egui::TouchPhase::Move,
        };
        let end_event = egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: Vec2::ZERO,
            modifiers: egui::Modifiers::NONE,
            phase: egui::TouchPhase::End,
        };

        assert!(has_nonzero_wheel_event(&[move_event]));
        assert!(!has_nonzero_wheel_event(&[end_event]));
        assert!(!has_nonzero_wheel_event(&[]));
    }
}

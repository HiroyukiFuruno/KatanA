/// A custom generic toggle switch widget.
///
/// Designed to visually represent boolean states as an iOS-style switch
/// instead of a traditional checkbox.
///
/// # Example
///
/// ```ignore
/// use katana_ui::widgets::toggle_switch;
///
/// let mut is_enabled = true;
/// if toggle_switch(ui, &mut is_enabled).clicked() {
///     // Handle state change
/// }
/// ```
pub fn toggle_switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
    });

    paint_toggle_switch(ui, rect, &response, *on);

    response
}

/// Extracted painting logic for reuse without click sensing
pub(crate) fn paint_toggle_switch(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    response: &egui::Response,
    on: bool,
) {
    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, on);
        let visuals = ui.style().interact_selectable(response, on);
        let rect = rect.expand(visuals.expansion);
        const TOGGLE_RADIUS_RATIO: f32 = 0.5;
        let radius = TOGGLE_RADIUS_RATIO * rect.height();
        ui.painter().rect(
            rect,
            radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        const TOGGLE_CIRCLE_RATIO: f32 = 0.75;
        ui.painter().circle(
            center,
            TOGGLE_CIRCLE_RATIO * radius,
            visuals.bg_fill,
            visuals.fg_stroke,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_switch_clicked() {
        let mut on = false;
        let ctx = egui::Context::default();

        let mut rect = egui::Rect::NOTHING;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rect = toggle_switch(ui, &mut on).rect;
            });
        });

        assert!(!on);

        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::PointerMoved(rect.center()));
        input.events.push(egui::Event::PointerButton {
            pos: rect.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        });
        input.events.push(egui::Event::PointerButton {
            pos: rect.center(),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        });

        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = toggle_switch(ui, &mut on);
            });
        });

        assert!(on);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TogglePosition {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToggleAlignment {
    /// Attach the toggle to the text with a specific margin.
    Attached(f32),
    /// Push the toggle and text to opposite ends of the available width.
    SpaceBetween,
}

/// A labeled toggle switch that ensures perfect vertical centering between text and the toggle.
/// It supports flexible alignment and positioning (e.g. left vs right).
pub struct LabeledToggle<'a> {
    text: egui::WidgetText,
    on: &'a mut bool,
    position: TogglePosition,
    alignment: ToggleAlignment,
}

impl<'a> LabeledToggle<'a> {
    pub fn new(text: impl Into<egui::WidgetText>, on: &'a mut bool) -> Self {
        Self {
            text: text.into(),
            on,
            position: TogglePosition::Right,
            alignment: ToggleAlignment::SpaceBetween,
        }
    }

    pub fn position(mut self, position: TogglePosition) -> Self {
        self.position = position;
        self
    }

    pub fn alignment(mut self, alignment: ToggleAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl<'a> egui::Widget for LabeledToggle<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut list_item = crate::widgets::ListItem::new().interactive(true);

        match self.alignment {
            ToggleAlignment::Attached(margin) => {
                list_item = list_item.spacing(margin);
            }
            ToggleAlignment::SpaceBetween => (),
        }

        let toggle_rect = std::rc::Rc::new(std::cell::Cell::new(egui::Rect::NOTHING));
        let rect_clone = toggle_rect.clone();

        let toggle_node: Box<dyn FnOnce(&mut egui::Ui) -> egui::Response + '_> =
            Box::new(move |ui: &mut egui::Ui| {
                let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
                let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
                rect_clone.set(rect);
                resp
            });

        let text_node: Box<dyn FnOnce(&mut egui::Ui) -> egui::Response + '_> =
            Box::new(|ui: &mut egui::Ui| ui.label(self.text));

        match (self.position, self.alignment) {
            (TogglePosition::Right, ToggleAlignment::SpaceBetween) => {
                list_item = list_item.left(text_node).right(toggle_node);
            }
            (TogglePosition::Left, ToggleAlignment::SpaceBetween) => {
                list_item = list_item.left(toggle_node).right(text_node);
            }
            (TogglePosition::Right, ToggleAlignment::Attached(_)) => {
                list_item = list_item.left(text_node).left(toggle_node);
            }
            (TogglePosition::Left, ToggleAlignment::Attached(_)) => {
                list_item = list_item.left(toggle_node).left(text_node);
            }
        }

        let mut row_resp = list_item.show(ui);

        if row_resp.clicked() {
            *self.on = !*self.on;
            row_resp.mark_changed();
        }

        row_resp.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *self.on, "")
        });

        paint_toggle_switch(ui, toggle_rect.get(), &row_resp, *self.on);

        row_resp
    }
}

#[cfg(test)]
mod labeled_toggle_tests {
    use super::*;
    use egui::Context;

    #[test]
    fn test_labeled_toggle_alignments() {
        let ctx = Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut on = false;

                // SpaceBetween Right
                let _ = ui.add(
                    LabeledToggle::new("Toggle Space Right", &mut on)
                        .position(TogglePosition::Right)
                        .alignment(ToggleAlignment::SpaceBetween),
                );

                // SpaceBetween Left
                let _ = ui.add(
                    LabeledToggle::new("Toggle Space Left", &mut on)
                        .position(TogglePosition::Left)
                        .alignment(ToggleAlignment::SpaceBetween),
                );

                // Attached Right
                let _ = ui.add(
                    LabeledToggle::new("Toggle Attached Right", &mut on)
                        .position(TogglePosition::Right)
                        .alignment(ToggleAlignment::Attached(8.0)),
                );

                // Attached Left
                let _ = ui.add(
                    LabeledToggle::new("Toggle Attached Left", &mut on)
                        .position(TogglePosition::Left)
                        .alignment(ToggleAlignment::Attached(8.0)),
                );

                // Click interaction
                let mut on_click = false;
                let _response = ui.add(LabeledToggle::new("Clickable", &mut on_click));
            });
        });
    }

    #[test]
    fn test_labeled_toggle_click() {
        let mut on = false;
        let ctx = Context::default();

        let mut rect = egui::Rect::NOTHING;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rect = ui.add(LabeledToggle::new("Clickable", &mut on)).rect;
            });
        });

        assert!(!on);

        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::PointerMoved(rect.center()));
        input.events.push(egui::Event::PointerButton {
            pos: rect.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        input.events.push(egui::Event::PointerButton {
            pos: rect.center(),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        });

        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _response = ui.add(LabeledToggle::new("Clickable", &mut on));
            });
        });

        assert!(on);
    }
}

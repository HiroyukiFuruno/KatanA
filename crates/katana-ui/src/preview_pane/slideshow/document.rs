use super::super::fullscreen::{
    SLIDESHOW_CONTROL_FADE_DELAY, SLIDESHOW_CONTROL_FADE_DURATION, SLIDESHOW_OPACITY_MAX,
    SLIDESHOW_OPACITY_MIN,
};
use super::controls::SlideshowControlsOps;
use super::settings::SlideshowSettingsOps;
use eframe::egui;

pub(super) fn render(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    layout: &mut crate::state::layout::LayoutState,
    pane: &mut crate::preview_pane::PreviewPane,
    blocker_rect: egui::Rect,
    content_rect: egui::Rect,
) {
    let (active, item_count) = pane.document_slideshow_state().unwrap_or((0, 0));
    layout.slideshow_page = active;
    let key_navigation = dispatch_key_navigation(ui, pane, item_count);

    ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
        pane.show_document_slideshow(ui);
    });
    let opacity = control_opacity(ctx, layout, ctx.input(|i| i.time));
    let page_delta = SlideshowControlsOps::render_slideshow_controls(
        ctx,
        ui,
        layout,
        blocker_rect,
        item_count.saturating_sub(1),
        opacity,
    );
    if !key_navigation && page_delta < 0 {
        pane.document_slideshow_step(false);
    } else if !key_navigation && page_delta > 0 {
        pane.document_slideshow_step(true);
    }
    if opacity > SLIDESHOW_OPACITY_MIN {
        SlideshowControlsOps::render_slideshow_close_button(ctx, ui, layout, blocker_rect, opacity);
        SlideshowSettingsOps::render_slideshow_settings_sidebar(
            ctx,
            ui,
            layout,
            blocker_rect,
            opacity,
        );
    }
}

fn dispatch_key_navigation(
    ui: &egui::Ui,
    pane: &mut crate::preview_pane::PreviewPane,
    item_count: usize,
) -> bool {
    let input = ui.input(|i| {
        (
            i.key_pressed(egui::Key::ArrowRight)
                || i.key_pressed(egui::Key::PageDown)
                || i.key_pressed(egui::Key::Space),
            i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::PageUp),
            i.key_pressed(egui::Key::Home),
            i.key_pressed(egui::Key::End),
        )
    });
    match input {
        (true, _, _, _) => pane.document_slideshow_step(true),
        (false, true, _, _) => pane.document_slideshow_step(false),
        (false, false, true, _) => pane.document_slideshow_jump_to(0),
        (false, false, false, true) => {
            pane.document_slideshow_jump_to(item_count.saturating_sub(1));
        }
        _ => {}
    }
    input.0 || input.1 || input.2 || input.3
}

fn control_opacity(
    ctx: &egui::Context,
    layout: &mut crate::state::layout::LayoutState,
    now: f64,
) -> f32 {
    let active = ctx.input(|i| {
        i.pointer.velocity() != egui::Vec2::ZERO || i.pointer.any_pressed() || !i.events.is_empty()
    });
    if active {
        layout.slideshow_last_active_time = now;
    }
    let idle = now - layout.slideshow_last_active_time;
    if idle <= SLIDESHOW_CONTROL_FADE_DELAY {
        return SLIDESHOW_OPACITY_MAX;
    }
    let progress = ((idle - SLIDESHOW_CONTROL_FADE_DELAY) as f32) / SLIDESHOW_CONTROL_FADE_DURATION;
    let opacity =
        (SLIDESHOW_OPACITY_MAX - progress).clamp(SLIDESHOW_OPACITY_MIN, SLIDESHOW_OPACITY_MAX);
    if opacity > SLIDESHOW_OPACITY_MIN && opacity < SLIDESHOW_OPACITY_MAX {
        ctx.request_repaint();
    }
    opacity
}

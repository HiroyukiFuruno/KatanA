use super::{ImageLogicOps, RasterizedInteraction};
use crate::preview_pane::ViewerState;
use eframe::egui;
use egui_kittest::{Harness, kittest::Queryable};
use katana_core::markdown::svg_rasterize::RasterizedSvg;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy)]
struct LayoutBounds {
    image: egui::Rect,
    selection: egui::Rect,
    following_cursor_y: f32,
    following_widget: egui::Rect,
}

#[test]
fn rasterized_wide_short_image_controls_are_below_image_and_inside_selection() {
    assert_rasterized_layout(egui::vec2(800.0, 480.0), egui::vec2(640.0, 24.0), true);
}

#[test]
fn rasterized_narrow_image_controls_are_below_image_and_inside_selection() {
    assert_rasterized_layout(egui::vec2(800.0, 480.0), egui::vec2(60.0, 160.0), true);
}

#[test]
fn rasterized_controls_stay_below_image_in_narrow_viewport() {
    assert_rasterized_layout(egui::vec2(160.0, 480.0), egui::vec2(640.0, 24.0), true);
}

#[test]
fn rasterized_hidden_controls_do_not_reserve_control_space() {
    assert_rasterized_layout(egui::vec2(800.0, 480.0), egui::vec2(640.0, 24.0), false);
}

fn assert_rasterized_layout(viewport: egui::Vec2, display_size: egui::Vec2, show_controls: bool) {
    let _guard = crate::test_render_env::RenderEnvLock::lock();
    let recorded = Arc::new(Mutex::new(None));
    let output = Arc::clone(&recorded);
    let image = RasterizedSvg::new(1, 1, display_size.x, display_size.y, vec![255, 0, 0, 255]);
    let mut state = ViewerState::default();
    let harness = Harness::builder().with_size(viewport).build_ui(move |ui| {
        let mut image_bounds = None;
        let interaction = if show_controls {
            RasterizedInteraction::Visible {
                state: Some(&mut state),
                fullscreen_request: None,
            }
        } else {
            RasterizedInteraction::Hidden {
                state: Some(&mut state),
            }
        };
        let selection = ImageLogicOps::show_rasterized(
            ui,
            &image,
            "Mermaid diagram",
            0,
            interaction,
            |_, rect, _| image_bounds = Some(rect),
        );
        let following_cursor_y = ui.cursor().top();
        let following_widget = ui.separator().rect;
        *output.lock().expect("layout bounds lock") = Some(LayoutBounds {
            image: image_bounds.expect("raster image allocation"),
            selection,
            following_cursor_y,
            following_widget,
        });
    });
    let bounds = recorded
        .lock()
        .expect("layout bounds lock")
        .expect("rasterized caller rendered");
    let following = bounds.following_widget;
    assert!(
        bounds.following_cursor_y >= bounds.selection.bottom(),
        "cursor y={} must follow selection {:?}",
        bounds.following_cursor_y,
        bounds.selection
    );
    assert!(
        following.top() >= bounds.selection.bottom(),
        "following content {following:?} must start after selection {:?}, image {:?}",
        bounds.selection,
        bounds.image
    );

    if !show_controls {
        assert!(
            harness
                .query_by_role(egui::accesskit::Role::Button)
                .is_none()
        );
        assert_eq!(bounds.selection, bounds.image);
        assert_eq!(bounds.image.height(), display_size.y);
        return;
    }

    let messages = crate::i18n::I18nOps::get();
    let controls = &messages.preview.diagram_controller;
    let fullscreen = harness.get_by_label(&controls.fullscreen).rect();
    assert!(bounds.image.contains_rect(fullscreen));
    assert!((bounds.image.right() - fullscreen.right() - 8.0).abs() <= 1.0);
    assert!((fullscreen.top() - bounds.image.top() - 8.0).abs() <= 1.0);
    let scaled_height = display_size.y * (bounds.image.width() / display_size.x).min(1.0);
    assert_eq!(
        bounds.image.height(),
        scaled_height.max(crate::diagram_controller::DiagramControllerOps::minimum_image_height())
    );

    let mut control_bounds = egui::Rect::NOTHING;
    /* WHY: 実際の呼び出し元が作るボタンの領域を調べ、補助関数だけの検証で重なりを見逃さない。 */
    for label in [
        &controls.pan_up,
        &controls.zoom_in,
        &controls.pan_left,
        &controls.reset,
        &controls.pan_right,
        &controls.trackpad_help,
        &controls.pan_down,
        &controls.zoom_out,
    ] {
        let button = harness.get_by_label(label).rect();
        assert!(
            button.top() >= bounds.image.bottom(),
            "control {label}: {button:?}, image {:?}",
            bounds.image
        );
        assert_eq!(bounds.image.intersect(button).area(), 0.0);
        assert!(bounds.selection.contains_rect(button));
        control_bounds = control_bounds.union(button);
    }
    assert!((control_bounds.right() - fullscreen.right()).abs() <= 1.0);
    assert!(bounds.selection.bottom() > bounds.image.bottom());
}

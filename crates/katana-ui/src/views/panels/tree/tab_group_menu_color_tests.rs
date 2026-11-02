use super::TabGroupMenu;
use crate::app_state::AppAction;
use crate::shell_ui::TreeRenderContext;
use crate::state::document::TabGroup;
use eframe::egui::{self, Color32, Shape};
use egui_kittest::kittest::Queryable;

const TEST_VIEWPORT: egui::Vec2 = egui::vec2(640.0, 480.0);

fn color(hex: &str) -> Color32 {
    Color32::from_hex(hex).expect("valid fixture color")
}

fn draw_menu(ui: &mut egui::Ui, action: &mut AppAction, groups: &[TabGroup]) {
    let mut expanded = std::collections::HashSet::new();
    let mut ctx = TreeRenderContext {
        action,
        depth: 0,
        active_path: None,
        filter_set: None,
        expanded_directories: &mut expanded,
        disable_context_menu: false,
        is_flat_view: false,
        ws_root: None,
        tab_groups: Some(groups),
        show_vertical_line: false,
    };
    TabGroupMenu::render(
        ui,
        std::path::Path::new("fixture.md"),
        &mut ctx,
        false,
        None,
    );
}

fn open_menu(groups: Vec<TabGroup>) -> egui_kittest::Harness<'static, AppAction> {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(TEST_VIEWPORT)
        .build_ui_state(
            move |ui, action: &mut AppAction| {
                crate::widgets::MenuButtonOps::show(ui, "Explorer", |ui| {
                    draw_menu(ui, action, &groups)
                });
            },
            AppAction::None,
        );
    harness.get_by_label("Explorer").click();
    harness.step();
    harness.step();
    harness
}

fn group(id: &str, name: &str, hex: &str) -> TabGroup {
    TabGroup {
        id: id.into(),
        name: name.into(),
        color_hex: hex.into(),
        collapsed: false,
        members: vec![],
    }
}

fn blue_center(shape: &Shape) -> Option<egui::Pos2> {
    match shape {
        Shape::Circle(circle) if circle.fill == color("#0000FF") => Some(circle.center),
        Shape::Vec(shapes) => shapes.iter().find_map(blue_center),
        _ => None,
    }
}

fn click_point(harness: &mut egui_kittest::Harness<'_, AppAction>, center: egui::Pos2) {
    for pressed in [true, false] {
        harness
            .input_mut()
            .events
            .push(egui::Event::PointerMoved(center));
        harness.input_mut().events.push(egui::Event::PointerButton {
            pos: center,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
    }
}

#[test]
fn explorer_menu_draws_and_selects_the_empty_group_color() {
    let mut harness = open_menu(vec![group("blue", "", "#0000FF")]);
    let center = harness
        .output()
        .shapes
        .iter()
        .find_map(|shape| {
            blue_center(&shape.shape).filter(|center| shape.clip_rect.contains(*center))
        })
        .expect("visible explorer group color");
    let right = row_right(&harness, center);
    click_point(&mut harness, right);
    assert!(
        matches!(harness.state(), AppAction::AddTabToGroup { group_id, member } if group_id == "blue" && member == std::path::Path::new("fixture.md"))
    );
}

fn row_right(harness: &egui_kittest::Harness<'_, AppAction>, dot: egui::Pos2) -> egui::Pos2 {
    let row = harness
        .get_all_by_role(egui::accesskit::Role::Button)
        .map(|node| node.rect())
        .filter(|rect| rect.contains(dot))
        .min_by(|left, right| left.area().total_cmp(&right.area()))
        .expect("group row containing color marker");
    egui::pos2(row.right() - 2.0, row.center().y)
}

#[test]
fn explorer_excludes_demo_and_preserves_existing_membership_rows() {
    let mut existing = group("member", "Member", "#0000FF");
    existing.members.push("fixture.md".into());
    let mut harness = open_menu(vec![group(TabGroup::DEMO_ID, "Demo", "#FF0000"), existing]);
    assert!(harness.query_by_label("Demo").is_none());
    let center = harness
        .output()
        .shapes
        .iter()
        .find_map(|shape| {
            blue_center(&shape.shape).filter(|center| shape.clip_rect.contains(*center))
        })
        .expect("visible existing membership");
    let right = row_right(&harness, center);
    click_point(&mut harness, right);
    assert!(
        matches!(harness.state(), AppAction::AddTabToGroup { group_id, .. } if group_id == "member")
    );
}

#[test]
fn explorer_short_and_long_names_select_the_right_edge() {
    for name in ["A", "A much longer group name"] {
        let mut harness = open_menu(vec![
            group("blue", name, "#0000FF"),
            group("other", "Other", "#FF0000"),
        ]);
        let center = harness
            .output()
            .shapes
            .iter()
            .find_map(|shape| {
                blue_center(&shape.shape).filter(|center| shape.clip_rect.contains(*center))
            })
            .expect("visible group marker");
        let right = row_right(&harness, center);
        click_point(&mut harness, right);
        assert!(
            matches!(harness.state(), AppAction::AddTabToGroup { group_id, .. } if group_id == "blue")
        );
    }
}

use super::TabContextMenu;
use crate::state::document::TabGroup;
use eframe::egui::{self, Color32, Shape};
use egui_kittest::kittest::Queryable;

const TEST_VIEWPORT: egui::Vec2 = egui::vec2(800.0, 600.0);

fn color(hex: &str) -> Color32 {
    Color32::from_hex(hex).expect("valid fixture color")
}

fn group(id: &str, name: &str, color: &str) -> TabGroup {
    TabGroup {
        id: id.into(),
        name: name.into(),
        color_hex: color.into(),
        collapsed: false,
        members: vec![],
    }
}

fn circle_center(shape: &Shape, color: Color32) -> Option<egui::Pos2> {
    match shape {
        Shape::Circle(circle) if circle.fill == color => Some(circle.center),
        Shape::Vec(shapes) => shapes.iter().find_map(|shape| circle_center(shape, color)),
        _ => None,
    }
}

fn visible_dot(
    harness: &egui_kittest::Harness<'_, Option<crate::app_state::AppAction>>,
    hex: &str,
) -> egui::Pos2 {
    harness
        .output()
        .shapes
        .iter()
        .find_map(|shape| {
            circle_center(&shape.shape, color(hex))
                .filter(|center| shape.clip_rect.contains(*center))
        })
        .expect("visible group marker")
}

fn click_point(
    harness: &mut egui_kittest::Harness<'_, Option<crate::app_state::AppAction>>,
    pos: egui::Pos2,
) {
    for pressed in [true, false] {
        harness
            .input_mut()
            .events
            .push(egui::Event::PointerMoved(pos));
        harness.input_mut().events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
    }
}

fn row_right(
    harness: &egui_kittest::Harness<'_, Option<crate::app_state::AppAction>>,
    dot: egui::Pos2,
) -> egui::Pos2 {
    let row = harness
        .get_all_by_role(egui::accesskit::Role::Button)
        .map(|node| node.rect())
        .filter(|rect| rect.contains(dot))
        .min_by(|left, right| left.area().total_cmp(&right.area()))
        .expect("group row containing color marker");
    egui::pos2(row.right() - 2.0, row.center().y)
}

fn open_menu(
    groups: Vec<TabGroup>,
) -> egui_kittest::Harness<'static, Option<crate::app_state::AppAction>> {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(TEST_VIEWPORT)
        .build_ui_state(
            move |ui, action: &mut Option<crate::app_state::AppAction>| {
                crate::widgets::MenuButtonOps::show(ui, "Context", |ui| {
                    let doc = katana_core::document::Document::new_empty("fixture.md");
                    TabContextMenu {
                        idx: 0,
                        doc: &doc,
                        tab_groups: &groups,
                        recently_closed_tabs_empty: true,
                    }
                    .show(ui, action);
                });
            },
            None,
        );
    harness.get_by_label("Context").click();
    harness.step();
    let submenu = format!("{} ⏵", crate::i18n::I18nOps::get().tab.add_to_group);
    harness.get_by_label(&submenu).click();
    harness.step();
    harness
}

#[test]
fn tab_group_submenu_shows_distinct_color_for_named_and_empty_groups() {
    let groups = vec![group("one", "", "#FF0000"), group("two", "Same", "#00FF00")];
    let harness = open_menu(groups);
    for color in [color("#FF0000"), color("#00FF00")] {
        assert!(
            harness
                .output()
                .shapes
                .iter()
                .any(|shape| circle_center(&shape.shape, color).is_some()),
            "missing group color {color:?}"
        );
    }
}

#[test]
fn colored_empty_group_row_selects_its_id_and_excludes_membership() {
    let mut existing = group("existing", "Same", "#FFFF00");
    existing.members.push("fixture.md".into());
    let groups = vec![
        group("empty_red", "", "#FF0000"),
        group("empty_blue", "", "#0000FF"),
        group("same_green", "Same", "#00FF00"),
        group("same_purple", "Same", "#FF00FF"),
        existing,
    ];
    let mut harness = open_menu(groups);
    assert_colors(&harness);
    let center = visible_dot(&harness, "#0000FF");
    let right = row_right(&harness, center);
    click_point(&mut harness, right);
    assert!(
        matches!(harness.state(), Some(crate::app_state::AppAction::AddTabToGroup { group_id, member }) if group_id == "empty_blue" && member == std::path::Path::new("fixture.md"))
    );
}

fn assert_colors(harness: &egui_kittest::Harness<'_, Option<crate::app_state::AppAction>>) {
    for color in [
        color("#FF0000"),
        color("#0000FF"),
        color("#00FF00"),
        color("#FF00FF"),
    ] {
        assert!(
            harness
                .output()
                .shapes
                .iter()
                .any(|shape| circle_center(&shape.shape, color).is_some())
        );
    }
    assert!(
        !harness
            .output()
            .shapes
            .iter()
            .any(|shape| circle_center(&shape.shape, color("#FFFF00")).is_some())
    );
}

#[test]
fn same_named_group_color_targets_the_selected_group() {
    let mut harness = open_menu(vec![
        group("green", "Same", "#00FF00"),
        group("purple", "Same", "#FF00FF"),
    ]);
    let center = visible_dot(&harness, "#FF00FF");
    let right = row_right(&harness, center);
    click_point(&mut harness, right);
    assert!(
        matches!(harness.state(), Some(crate::app_state::AppAction::AddTabToGroup { group_id, .. }) if group_id == "purple")
    );
}

#[test]
fn short_and_long_group_rows_select_from_the_right_edge() {
    for name in ["A", "A much longer group name"] {
        let mut harness = open_menu(vec![
            group("target", name, "#123456"),
            group("other", "Other", "#654321"),
        ]);
        let center = visible_dot(&harness, "#123456");
        let right = row_right(&harness, center);
        click_point(&mut harness, right);
        assert!(
            matches!(harness.state(), Some(crate::app_state::AppAction::AddTabToGroup { group_id, .. }) if group_id == "target")
        );
    }
}

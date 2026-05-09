use crate::integration::harness_utils::{
    flatten_clipped_shapes, fresh_temp_dir, setup_harness, wait_for_workspace_load,
};
use crate::tab_bar_visual_helpers::{
    document_tab_border, leading_icon_rect_for_label, matching_border_rect,
    maybe_trailing_control_rect_for_label, open_document_names, topmost_label_rect,
    trailing_control_rect_for_label, workspace_name, workspace_tab_border_rects,
};
use eframe::egui;
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;

const TAB_VISIBLE_RIGHT_LIMIT: f32 = 1150.0;

#[test]
fn workspace_tab_parent_borders_share_the_same_vertical_center() {
    let mut harness = setup_harness();
    harness.step();

    let first_workspace = fresh_temp_dir("katana_workspace_tab_center_a");
    let second_workspace = fresh_temp_dir("katana_workspace_tab_center_b");
    std::fs::write(first_workspace.join("a.md"), "# A").unwrap();
    std::fs::write(second_workspace.join("b.md"), "# B").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(first_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(second_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness.run_steps(10);

    let _ = topmost_label_rect(&harness, &workspace_name(&first_workspace));
    let _ = topmost_label_rect(&harness, &workspace_name(&second_workspace));

    let mut borders = workspace_tab_border_rects(&harness);
    borders.sort_by(|left, right| left.left().total_cmp(&right.left()));
    assert!(
        borders.len() >= 2,
        "workspace tab borders not found: {borders:?}"
    );

    let delta = (borders[0].center().y - borders[1].center().y).abs();

    assert!(
        delta <= 0.5,
        "workspace tab parent borders must share vertical center: {:?} vs {:?}",
        borders[0],
        borders[1]
    );
}

#[test]
fn workspace_tab_icon_and_title_are_adjacent() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_workspace_tab_icon_gap");
    std::fs::write(workspace.join("a.md"), "# A").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness.run_steps(10);

    let title_rect = topmost_label_rect(&harness, &workspace_name(&workspace));
    let icon_rect = leading_icon_rect_for_label(&harness, &workspace_name(&workspace));
    let gap = title_rect.left() - icon_rect.right();

    assert!(
        (3.0..=6.0).contains(&gap),
        "workspace tab icon and title must use the intended 4px gap, got {gap}: {icon_rect:?} vs {title_rect:?}"
    );
}

#[test]
fn workspace_tab_close_stays_at_right_edge() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_workspace_tab_close_right");
    std::fs::write(workspace.join("a.md"), "# A").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness.run_steps(10);

    let title_rect = topmost_label_rect(&harness, &workspace_name(&workspace));
    let border = workspace_tab_border_rects(&harness)
        .into_iter()
        .find(|rect| rect.contains(title_rect.center()))
        .expect("workspace tab border for title");
    harness.hover_at(border.center());
    harness.run_steps(2);

    let title_rect = topmost_label_rect(&harness, &workspace_name(&workspace));
    let close_rect = trailing_control_rect_for_label(&harness, &workspace_name(&workspace));
    let border = workspace_tab_border_rects(&harness)
        .into_iter()
        .find(|rect| rect.contains(title_rect.center()))
        .expect("workspace tab border for title");
    let right_gap = border.right() - close_rect.right();
    let title_close_gap = close_rect.left() - title_rect.right();

    assert!(
        (0.0..=2.0).contains(&right_gap),
        "workspace close control must stay at the right edge, got {right_gap}: {border:?} vs {close_rect:?}"
    );
    assert!(
        title_close_gap >= 8.0,
        "workspace close control must not be part of centered title group, got {title_close_gap}: {title_rect:?} vs {close_rect:?}"
    );
}

#[test]
fn workspace_tab_close_button_closes_hovered_tab() {
    let mut harness = setup_harness();
    harness.step();

    let first_workspace = fresh_temp_dir("katana_workspace_tab_close_click_a");
    let second_workspace = fresh_temp_dir("katana_workspace_tab_close_click_b");
    std::fs::write(first_workspace.join("a.md"), "# A").unwrap();
    std::fs::write(second_workspace.join("b.md"), "# B").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(first_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(second_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness.run_steps(10);

    let first_name = workspace_name(&first_workspace);
    let first_title = topmost_label_rect(&harness, &first_name);
    let first_border = workspace_tab_border_rects(&harness)
        .into_iter()
        .find(|rect| rect.contains(first_title.center()))
        .expect("first workspace tab border");
    harness.hover_at(first_border.center());
    harness.run_steps(3);
    click_trailing_control_for_label(&mut harness, &first_name);
    harness.run_steps(10);

    let first_path = first_workspace.display().to_string();
    let open_tabs = &harness
        .state_mut()
        .app_state_mut()
        .global_workspace
        .state()
        .open_workspace_tabs;
    assert!(
        !open_tabs.contains(&first_path),
        "workspace tab close button must remove the hovered workspace tab: {open_tabs:?}"
    );
}

#[test]
fn workspace_tabs_can_be_reordered_by_dragging() {
    let mut harness = setup_harness();
    harness.step();

    let first_workspace = fresh_temp_dir("katana_workspace_tab_drag_a");
    let second_workspace = fresh_temp_dir("katana_workspace_tab_drag_b");
    let third_workspace = fresh_temp_dir("katana_workspace_tab_drag_c");
    std::fs::write(first_workspace.join("a.md"), "# A").unwrap();
    std::fs::write(second_workspace.join("b.md"), "# B").unwrap();
    std::fs::write(third_workspace.join("c.md"), "# C").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(first_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(second_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(third_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness.run_steps(10);

    let first_tab = workspace_tab_border_for_label(&harness, &workspace_name(&first_workspace));
    let last_tab = workspace_tab_border_for_label(&harness, &workspace_name(&third_workspace));
    let target = egui::pos2(last_tab.right() + 24.0, last_tab.center().y);

    harness.hover_at(first_tab.center());
    harness.step();
    harness.drag_at(first_tab.center());
    harness.step();
    harness.hover_at(target);
    harness.run_steps(3);
    harness.drop_at(target);
    harness.run_steps(10);

    let first_path = first_workspace.display().to_string();
    let second_path = second_workspace.display().to_string();
    let third_path = third_workspace.display().to_string();
    let open_tabs = harness
        .state_mut()
        .app_state_mut()
        .global_workspace
        .state()
        .open_workspace_tabs
        .clone();
    assert_eq!(open_tabs, [second_path, third_path, first_path]);
}

#[test]
fn workspace_tab_drag_ghost_stays_on_workspace_tab_row() {
    let mut harness = setup_harness();
    harness.step();

    let first_workspace = fresh_temp_dir("katana_workspace_tab_ghost_a");
    let second_workspace = fresh_temp_dir("katana_workspace_tab_ghost_b");
    let third_workspace = fresh_temp_dir("katana_workspace_tab_ghost_c");
    std::fs::write(first_workspace.join("a.md"), "# A").unwrap();
    std::fs::write(second_workspace.join("b.md"), "# B").unwrap();
    std::fs::write(third_workspace.join("c.md"), "# C").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(first_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(second_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(third_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness.run_steps(10);

    let first_name = workspace_name(&first_workspace);
    let first_tab = workspace_tab_border_for_label(&harness, &first_name);
    let last_tab = workspace_tab_border_for_label(&harness, &workspace_name(&third_workspace));
    let target = egui::pos2(last_tab.right() + 24.0, last_tab.center().y);

    harness.hover_at(first_tab.center());
    harness.step();
    harness.drag_at(first_tab.center());
    harness.step();
    harness.hover_at(target);
    harness.run_steps(3);

    let label_rects: Vec<_> = harness
        .query_all_by_label(&first_name)
        .map(|node| node.rect())
        .collect();
    assert!(
        label_rects.len() >= 2,
        "workspace drag must render a ghost label as document tabs do: {label_rects:?}"
    );
    assert!(
        label_rects
            .iter()
            .all(|rect| rect.top() >= first_tab.top() - 1.0),
        "workspace drag ghost must stay on the workspace tab row, not at viewport origin: row={first_tab:?}, labels={label_rects:?}"
    );
}

#[test]
fn workspace_tab_drag_shows_drop_indicator() {
    let mut harness = setup_harness();
    harness.step();

    let first_workspace = fresh_temp_dir("katana_workspace_tab_indicator_a");
    let second_workspace = fresh_temp_dir("katana_workspace_tab_indicator_b");
    let third_workspace = fresh_temp_dir("katana_workspace_tab_indicator_c");
    std::fs::write(first_workspace.join("a.md"), "# A").unwrap();
    std::fs::write(second_workspace.join("b.md"), "# B").unwrap();
    std::fs::write(third_workspace.join("c.md"), "# C").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(first_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(second_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(third_workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness.run_steps(10);

    let first_tab = workspace_tab_border_for_label(&harness, &workspace_name(&first_workspace));
    let last_tab = workspace_tab_border_for_label(&harness, &workspace_name(&third_workspace));
    let target = egui::pos2(last_tab.right() + 24.0, last_tab.center().y);

    harness.hover_at(first_tab.center());
    harness.step();
    harness.drag_at(first_tab.center());
    harness.step();
    harness.hover_at(target);
    harness.run_steps(3);

    assert!(
        workspace_drop_indicator_visible(&harness, first_tab),
        "workspace tab drag must show a document-tab-style drop indicator on the tab row"
    );
}

#[test]
fn workspace_tab_close_hover_does_not_paint_close_button_frame() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_workspace_tab_close_frame");
    std::fs::write(workspace.join("a.md"), "# A").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness.run_steps(10);

    let workspace_label = workspace_name(&workspace);
    let border = workspace_tab_border_for_label(&harness, &workspace_label);
    harness.hover_at(border.center());
    harness.run_steps(3);
    let close_rect = trailing_control_rect_for_label(&harness, &workspace_label);
    harness.hover_at(close_rect.center());
    harness.run_steps(3);

    let close_button_frame = flatten_clipped_shapes(&harness.output().shapes)
        .into_iter()
        .any(|shape| match shape {
            egui::epaint::Shape::Rect(rect_shape) => {
                rect_shape.rect.contains(close_rect.center())
                    && rect_shape.rect.width() <= close_rect.width() + 4.0
                    && (rect_shape.fill.a() > 0 || rect_shape.stroke.width > 0.0)
            }
            _ => false,
        });

    assert!(
        !close_button_frame,
        "workspace tab close hover must not paint a close-button-local frame: {close_rect:?}"
    );
}

#[test]
fn workspace_tab_icon_and_title_share_vertical_center() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_workspace_tab_icon_center");
    std::fs::write(workspace.join("a.md"), "# A").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness.run_steps(10);

    let title_rect = topmost_label_rect(&harness, &workspace_name(&workspace));
    let icon_rect = leading_icon_rect_for_label(&harness, &workspace_name(&workspace));
    let offset = icon_rect.center().y - title_rect.center().y;

    assert!(
        (0.5..=1.5).contains(&offset),
        "workspace tab icon must be visually centered 1px below the title center: {icon_rect:?} vs {title_rect:?}"
    );
}

#[test]
fn document_tabs_start_immediately_below_workspace_tabs() {
    let mut harness = setup_harness();
    harness.step();
    harness.state_mut().app_state_mut().layout.show_explorer = false;

    let workspace = fresh_temp_dir("katana_workspace_tab_no_scrollbar_gutter");
    let readme = workspace.join("README.md");
    std::fs::write(&readme, "# README").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![readme]));
    harness.run_steps(20);

    let workspace_title_rect = topmost_label_rect(&harness, &workspace_name(&workspace));
    let workspace_border = workspace_tab_border_rects(&harness)
        .into_iter()
        .find(|rect| rect.contains(workspace_title_rect.center()))
        .expect("workspace tab border for title");
    let document_border = document_tab_border(&harness, "README.md");
    let gap = document_border.top() - workspace_border.bottom();

    assert!(
        (-1.0..=12.0).contains(&gap),
        "workspace tab row must not reserve hidden horizontal scrollbar gutter, got vertical gap {gap}: {workspace_border:?} -> {document_border:?}"
    );
}

#[test]
fn pinned_explorer_keeps_document_tabs_in_content_area() {
    let mut harness = setup_harness();
    harness.step();
    harness.state_mut().app_state_mut().layout.show_explorer = true;

    let workspace = fresh_temp_dir("katana_pinned_explorer_document_tab_offset");
    let readme = workspace.join("README.md");
    std::fs::write(&readme, "# README").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![readme]));
    harness.run_steps(20);

    let document_border = document_tab_border(&harness, "README.md");

    assert!(
        document_border.left() >= 200.0,
        "pinned explorer must reserve left content area before document tabs, got {document_border:?}"
    );
}

#[test]
fn breadcrumb_first_segment_keeps_left_padding() {
    let mut harness = setup_harness();
    harness.step();
    harness.state_mut().app_state_mut().layout.show_explorer = false;

    let workspace = fresh_temp_dir("katana_breadcrumb_left_padding");
    let nested_dir = workspace.join("docs").join("nested");
    std::fs::create_dir_all(&nested_dir).unwrap();
    let readme = nested_dir.join("README.md");
    std::fs::write(&readme, "# README").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme));
    harness.run_steps(20);

    let document_border = document_tab_border(&harness, "README.md");
    let first_segment = topmost_label_rect(&harness, "docs");
    let left_padding = first_segment.left() - document_border.left();

    assert!(
        left_padding >= 6.0,
        "breadcrumb first segment must not be flush with the content edge, got {left_padding}: {document_border:?} vs {first_segment:?}"
    );
}

#[test]
fn breadcrumb_hover_does_not_paint_button_background() {
    let mut harness = setup_harness();
    harness.step();
    harness.state_mut().app_state_mut().layout.show_explorer = false;

    let workspace = fresh_temp_dir("katana_breadcrumb_no_bg");
    let nested_dir = workspace.join("docs").join("nested");
    std::fs::create_dir_all(&nested_dir).unwrap();
    let readme = nested_dir.join("README.md");
    std::fs::write(&readme, "# README").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme));
    harness.run_steps(20);

    let first_segment = topmost_label_rect(&harness, "docs");
    harness.hover_at(first_segment.center());
    harness.run_steps(3);
    let first_segment = topmost_label_rect(&harness, "docs");
    let tight_background = flatten_clipped_shapes(&harness.output().shapes)
        .into_iter()
        .any(|shape| match shape {
            egui::epaint::Shape::Rect(rect_shape) => {
                rect_shape.fill.a() > 0
                    && rect_shape.rect.contains(first_segment.center())
                    && rect_shape.rect.width() <= first_segment.width() + 24.0
                    && rect_shape.rect.height() <= first_segment.height() + 16.0
            }
            _ => false,
        });

    assert!(
        !tight_background,
        "breadcrumb segment must not paint a button background on hover: {first_segment:?}"
    );
}

#[test]
fn document_tab_border_wraps_title_and_close_without_extra_width() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_document_tab_border");
    let readme = workspace.join("README.md");
    let sample = workspace.join("sample.md");
    std::fs::write(&readme, "# README").unwrap();
    std::fs::write(&sample, "# Sample").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![readme, sample]));
    harness.run_steps(20);

    hover_document_tab(&mut harness, "README.md");
    let title_rect = topmost_label_rect(&harness, "README.md");
    let close_rect = trailing_control_rect_for_label(&harness, "README.md");
    let border = matching_border_rect(&harness, title_rect)
        .unwrap_or_else(|| panic!("document tab parent border not found for {title_rect:?}"));

    let left_padding = title_rect.left() - border.left();
    let right_overhang = border.right() - close_rect.right();
    assert!(
        (6.0..=10.0).contains(&left_padding),
        "document tab border must start at the title padding only, got {left_padding}: {border:?} vs {title_rect:?}"
    );
    assert!(
        (0.0..=2.0).contains(&right_overhang),
        "document tab border must end at the close control without extra width, got {right_overhang}: {border:?} vs {close_rect:?}"
    );
}

#[test]
fn document_tab_close_appears_on_hover_and_expands_width() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_document_tab_hover_close");
    let readme = workspace.join("README.md");
    std::fs::write(&readme, "# README").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![readme]));
    harness.run_steps(20);

    let compact_border = document_tab_border(&harness, "README.md");
    let compact_title_rect = topmost_label_rect(&harness, "README.md");
    let compact_title_right_padding = compact_border.right() - compact_title_rect.right();
    assert!(
        maybe_trailing_control_rect_for_label(&harness, "README.md").is_none(),
        "document tab close control must be hidden until hover"
    );
    assert!(
        (7.5..=8.5).contains(&compact_title_right_padding),
        "document tab title must keep symmetric right padding before hover, got {compact_title_right_padding}: {compact_border:?} vs {compact_title_rect:?}"
    );

    let expanded_border = hover_document_tab(&mut harness, "README.md");
    let close_rect = trailing_control_rect_for_label(&harness, "README.md");
    let width_delta = expanded_border.width() - compact_border.width();
    let right_margin = expanded_border.right() - close_rect.right();

    assert!(
        width_delta >= 16.0,
        "document tab must expand when hover reveals close control, got {width_delta}: {compact_border:?} -> {expanded_border:?}"
    );
    assert!(
        (1.5..=2.5).contains(&right_margin),
        "document tab close control must keep the symmetric 2px right margin, got {right_margin}: {expanded_border:?} vs {close_rect:?}"
    );
}

#[test]
fn document_tab_height_stays_stable_when_close_appears() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_document_tab_hover_height");
    let readme = workspace.join("README.md");
    std::fs::write(&readme, "# README").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![readme]));
    harness.run_steps(20);

    let compact_border = document_tab_border(&harness, "README.md");
    let expanded_border = hover_document_tab(&mut harness, "README.md");
    let height_delta = (expanded_border.height() - compact_border.height()).abs();

    assert!(
        height_delta <= 0.5,
        "document tab hover must not expand vertical size, got {height_delta}: {compact_border:?} -> {expanded_border:?}"
    );
}

#[test]
fn document_tab_height_stays_stable_when_close_itself_is_hovered() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_document_tab_close_hover_height");
    let readme = workspace.join("README.md");
    let sample = workspace.join("sample.md");
    std::fs::write(&readme, "# README").unwrap();
    std::fs::write(&sample, "# Sample").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![readme, sample]));
    harness.run_steps(20);

    let compact_readme_border = document_tab_border(&harness, "README.md");
    let compact_sample_border = document_tab_border(&harness, "sample.md");
    let compact_row_top = compact_readme_border.top().min(compact_sample_border.top());
    let compact_row_bottom = compact_readme_border
        .bottom()
        .max(compact_sample_border.bottom());

    hover_document_tab(&mut harness, "README.md");
    let close_rect = trailing_control_rect_for_label(&harness, "README.md");
    harness.hover_at(close_rect.center());
    harness.run_steps(3);

    let hovered_readme_border = document_tab_border(&harness, "README.md");
    let hovered_sample_border = document_tab_border(&harness, "sample.md");
    let hovered_row_top = hovered_readme_border.top().min(hovered_sample_border.top());
    let hovered_row_bottom = hovered_readme_border
        .bottom()
        .max(hovered_sample_border.bottom());
    let row_height_delta =
        ((hovered_row_bottom - hovered_row_top) - (compact_row_bottom - compact_row_top)).abs();
    let readme_height_delta =
        (hovered_readme_border.height() - compact_readme_border.height()).abs();
    let sample_height_delta =
        (hovered_sample_border.height() - compact_sample_border.height()).abs();

    assert!(
        row_height_delta <= 0.5,
        "document tab row height must stay stable when the close icon itself is hovered, got {row_height_delta}: row {compact_row_top}..{compact_row_bottom} -> {hovered_row_top}..{hovered_row_bottom}"
    );
    assert!(
        readme_height_delta <= 0.5,
        "hovered document tab height must stay stable when its close icon is hovered, got {readme_height_delta}: {compact_readme_border:?} -> {hovered_readme_border:?}"
    );
    assert!(
        sample_height_delta <= 0.5,
        "neighbor document tab height must stay stable when another tab close icon is hovered, got {sample_height_delta}: {compact_sample_border:?} -> {hovered_sample_border:?}"
    );
}

#[test]
fn document_tab_close_hover_does_not_paint_full_height_button_frame() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_document_tab_close_frame_height");
    let readme = workspace.join("README.md");
    std::fs::write(&readme, "# README").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![readme]));
    harness.run_steps(20);

    hover_document_tab(&mut harness, "README.md");
    let close_rect = trailing_control_rect_for_label(&harness, "README.md");
    let oversized_button_frame = flatten_clipped_shapes(&harness.output().shapes)
        .into_iter()
        .any(|shape| match shape {
            egui::epaint::Shape::Rect(rect_shape) => {
                rect_shape.rect.contains(close_rect.center())
                    && (rect_shape.fill.a() > 0 || rect_shape.stroke.width > 0.0)
                    && rect_shape.rect.width() <= close_rect.width() + 4.0
                    && rect_shape.rect.height() > 16.0
            }
            _ => false,
        });

    assert!(
        !oversized_button_frame,
        "document tab close hover must not paint a full-height button frame: {close_rect:?}"
    );
}

#[test]
fn document_tab_close_button_closes_hovered_tab() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_document_tab_close_click");
    let readme = workspace.join("README.md");
    let sample = workspace.join("sample.md");
    std::fs::write(&readme, "# README").unwrap();
    std::fs::write(&sample, "# Sample").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![
            readme.clone(),
            sample,
        ]));
    harness.run_steps(20);

    hover_document_tab(&mut harness, "README.md");
    click_trailing_control_for_label(&mut harness, "README.md");
    harness.run_steps(10);

    let document_names = open_document_names(&mut harness);
    assert_eq!(
        document_names,
        ["sample.md"],
        "document tab close button must remove the hovered document tab"
    );
}

#[test]
fn document_tab_content_uses_readable_padding() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_document_tab_padding");
    let readme = workspace.join("README.md");
    std::fs::write(&readme, "# README").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![readme]));
    harness.run_steps(20);

    hover_document_tab(&mut harness, "README.md");
    let title_rect = topmost_label_rect(&harness, "README.md");
    let close_rect = trailing_control_rect_for_label(&harness, "README.md");
    let border = document_tab_border(&harness, "README.md");
    let left_padding = title_rect.left() - border.left();
    let title_close_gap = close_rect.left() - title_rect.right();
    let right_padding = border.right() - close_rect.right();

    assert!(
        close_rect.width() <= 28.0,
        "document tab close control must use compact icon width, got {}: {close_rect:?}",
        close_rect.width()
    );
    assert!(
        (6.0..=10.0).contains(&left_padding),
        "document tab title must keep readable left padding, got {left_padding}: {border:?} vs {title_rect:?}"
    );
    assert!(
        (3.5..=14.0).contains(&title_close_gap),
        "document tab title and close icon must not touch or drift apart, got {title_close_gap}: {title_rect:?} vs {close_rect:?}"
    );
    assert!(
        (1.5..=2.5).contains(&right_padding),
        "document tab close control must keep the symmetric 2px right margin, got {right_padding}: {border:?} vs {close_rect:?}"
    );
}

#[test]
fn short_document_tab_title_is_not_truncated() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_document_tab_short_title");
    let readme = workspace.join("README.md");
    std::fs::write(&readme, "# README").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![readme]));
    harness.run_steps(20);

    let title_rect = topmost_label_rect(&harness, "README.md");
    let border = document_tab_border(&harness, "README.md");

    assert!(
        title_rect.width() >= 70.0,
        "short document tab title must not be ellipsized: {title_rect:?}"
    );
    assert!(
        border.width() <= 150.0,
        "short document tab must keep the original compact width: {border:?}"
    );
}

#[test]
fn document_tab_title_and_close_share_vertical_center() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_document_tab_content_center");
    let readme = workspace.join("README.md");
    std::fs::write(&readme, "# README").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![readme]));
    harness.run_steps(20);

    hover_document_tab(&mut harness, "README.md");
    let title_rect = topmost_label_rect(&harness, "README.md");
    let close_rect = trailing_control_rect_for_label(&harness, "README.md");
    let border = document_tab_border(&harness, "README.md");
    let title_delta = (title_rect.center().y - border.center().y).abs();
    let close_delta = (close_rect.center().y - border.center().y).abs();

    assert!(
        title_delta <= 0.5,
        "document tab title must share vertical center with parent border: {border:?} vs {title_rect:?}"
    );
    assert!(
        close_delta <= 0.5,
        "document tab close icon must share vertical center with parent border: {border:?} vs {close_rect:?}"
    );
}

#[test]
fn document_tab_parent_borders_are_adjacent_without_gap() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_document_tab_adjacency");
    let readme = workspace.join("README.md");
    let sample = workspace.join("sample.md");
    std::fs::write(&readme, "# README").unwrap();
    std::fs::write(&sample, "# Sample").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![readme, sample]));
    harness.run_steps(20);

    let readme_border = document_tab_border(&harness, "README.md");
    let sample_border = document_tab_border(&harness, "sample.md");
    let delta = (readme_border.right() - sample_border.left()).abs();

    assert!(
        delta <= 0.5,
        "document tab parent borders must be directly adjacent without visual gap: {readme_border:?} vs {sample_border:?}"
    );
}

#[test]
fn document_tab_parent_borders_share_the_same_vertical_center() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_document_tab_center");
    let readme = workspace.join("README.md");
    let sample = workspace.join("sample.md");
    std::fs::write(&readme, "# README").unwrap();
    std::fs::write(&sample, "# Sample").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![readme, sample]));
    harness.run_steps(20);

    let readme_border = document_tab_border(&harness, "README.md");
    let sample_border = document_tab_border(&harness, "sample.md");
    let delta = (readme_border.center().y - sample_border.center().y).abs();

    assert!(
        delta <= 0.5,
        "document tab parent borders must share vertical center: {readme_border:?} vs {sample_border:?}"
    );
}

#[test]
fn document_tabs_can_still_be_reordered_by_dragging() {
    let mut harness = setup_harness();
    harness.step();

    let workspace = fresh_temp_dir("katana_document_tab_drag");
    let a = workspace.join("a.md");
    let b = workspace.join("b.md");
    let c = workspace.join("c.md");
    std::fs::write(&a, "# A").unwrap();
    std::fs::write(&b, "# B").unwrap();
    std::fs::write(&c, "# C").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![a, b, c]));
    harness.run_steps(20);

    let first_tab = document_tab_border(&harness, "a.md");
    let last_tab = document_tab_border(&harness, "c.md");
    let target = egui::pos2(last_tab.right() + 24.0, last_tab.center().y);

    harness.hover_at(first_tab.center());
    harness.step();
    harness.drag_at(first_tab.center());
    harness.step();
    harness.hover_at(target);
    harness.run_steps(3);
    harness.drop_at(target);
    harness.run_steps(10);

    let order = open_document_names(&mut harness);
    assert_eq!(order, ["b.md", "c.md", "a.md"]);
}

#[test]
fn shortcut_prev_from_first_scrolls_wrapped_last_tab_into_view() {
    let mut harness = setup_harness();
    harness.step();

    let document_names = open_many_document_tabs(&mut harness, "katana_prev_wrap_scroll", 14);
    harness.state_mut().trigger_action(AppAction::SelectPrevTab);
    harness.run_steps(20);

    let active_idx = harness.state_mut().app_state_mut().document.active_doc_idx;
    assert_eq!(active_idx, Some(document_names.len() - 1));
    assert_document_tab_visible(&harness, document_names.last().unwrap());
}

#[test]
fn shortcut_next_from_last_returns_scroll_to_first_tab_and_keeps_following() {
    let mut harness = setup_harness();
    harness.step();

    let document_names = open_many_document_tabs(&mut harness, "katana_next_wrap_scroll", 14);
    let last_path = {
        let state = harness.state_mut().app_state_mut();
        state
            .document
            .open_documents
            .last()
            .expect("last document")
            .path
            .clone()
    };
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(last_path));
    harness.run_steps(20);
    assert_document_tab_visible(&harness, document_names.last().unwrap());

    harness.state_mut().trigger_action(AppAction::SelectNextTab);
    harness.run_steps(20);
    assert_eq!(
        harness.state_mut().app_state_mut().document.active_doc_idx,
        Some(0)
    );
    assert_document_tab_visible(&harness, &document_names[0]);

    harness.state_mut().trigger_action(AppAction::SelectNextTab);
    harness.run_steps(20);
    assert_eq!(
        harness.state_mut().app_state_mut().document.active_doc_idx,
        Some(1)
    );
    assert_document_tab_visible(&harness, &document_names[1]);
}

fn hover_document_tab(
    harness: &mut egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
    label: &str,
) -> egui::Rect {
    let border = document_tab_border(harness, label);
    harness.hover_at(border.center());
    harness.run_steps(3);
    document_tab_border(harness, label)
}

fn workspace_tab_border_for_label(
    harness: &egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
    label: &str,
) -> egui::Rect {
    let title = topmost_label_rect(harness, label);
    workspace_tab_border_rects(harness)
        .into_iter()
        .find(|rect| rect.contains(title.center()))
        .unwrap_or_else(|| panic!("workspace tab border not found for {label}: {title:?}"))
}

fn workspace_drop_indicator_visible(
    harness: &egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
    row_rect: egui::Rect,
) -> bool {
    flatten_clipped_shapes(&harness.output().shapes)
        .into_iter()
        .any(|shape| match shape {
            egui::epaint::Shape::LineSegment { points, stroke } => {
                let is_vertical = (points[0].x - points[1].x).abs() <= 0.5;
                let y_min = points[0].y.min(points[1].y);
                let y_max = points[0].y.max(points[1].y);
                is_vertical
                    && stroke.width >= 2.0
                    && y_min <= row_rect.top() + 1.0
                    && y_max >= row_rect.bottom() - 1.0
            }
            _ => false,
        })
}

fn click_trailing_control_for_label(
    harness: &mut egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
    label: &str,
) {
    let close_rect = trailing_control_rect_for_label(harness, label);
    let Some(node) = harness
        .query_all_by_role(egui::accesskit::Role::Button)
        .find(|node| close_rect.contains(node.rect().center()))
    else {
        panic!("close button node not found for {label}: {close_rect:?}");
    };
    node.click();
}

fn open_many_document_tabs(
    harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
    prefix: &str,
    count: usize,
) -> Vec<String> {
    let workspace = fresh_temp_dir(prefix);
    let mut paths = Vec::new();
    let mut names = Vec::new();
    for index in 0..count {
        let name = format!("file_{index:02}.md");
        let path = workspace.join(&name);
        std::fs::write(&path, format!("# File {index}")).unwrap();
        paths.push(path);
        names.push(name);
    }

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(paths));
    harness.run_steps(count + 30);
    names
}

fn assert_document_tab_visible(
    harness: &egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
    label: &str,
) {
    let border = document_tab_border(harness, label);
    assert!(
        border.left() >= 0.0 && border.right() <= TAB_VISIBLE_RIGHT_LIMIT,
        "active document tab must be scrolled into the visible tab strip: {label} {border:?}"
    );
}

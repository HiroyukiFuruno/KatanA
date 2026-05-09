use crate::integration::harness_utils::flatten_clipped_shapes;
use eframe::egui;
use egui_kittest::kittest::Queryable;

pub fn workspace_name(workspace: &std::path::Path) -> String {
    workspace.file_name().unwrap().to_string_lossy().to_string()
}

pub fn workspace_tab_border_rects(
    harness: &egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
) -> Vec<egui::Rect> {
    border_rects(harness)
        .into_iter()
        .filter(|rect| rect.top() < 60.0 && rect.width() > 100.0)
        .collect()
}

pub fn leading_icon_rect_for_label(
    harness: &egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
    label: &str,
) -> egui::Rect {
    let label_rect = topmost_label_rect(harness, label);
    harness
        .query_all_by_role(egui::accesskit::Role::Image)
        .map(|node| node.rect())
        .filter(|rect| rect.right() <= label_rect.left())
        .filter(|rect| (rect.center().y - label_rect.center().y).abs() <= 4.0)
        .max_by(|left, right| left.right().total_cmp(&right.right()))
        .unwrap_or_else(|| panic!("leading icon not found for label: {label}"))
}

pub fn trailing_control_rect_for_label(
    harness: &egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
    label: &str,
) -> egui::Rect {
    maybe_trailing_control_rect_for_label(harness, label)
        .unwrap_or_else(|| panic!("trailing control not found for label: {label}"))
}

pub fn maybe_trailing_control_rect_for_label(
    harness: &egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
    label: &str,
) -> Option<egui::Rect> {
    let label_rect = topmost_label_rect(harness, label);
    let border = matching_border_rect(harness, label_rect)?;
    harness
        .query_all_by_role(egui::accesskit::Role::Button)
        .map(|node| node.rect())
        .filter(|rect| rect.left() >= label_rect.right())
        .filter(|rect| (rect.center().y - label_rect.center().y).abs() <= 4.0)
        .filter(|rect| border.expand(1.0).contains(rect.center()))
        .min_by(|left, right| left.left().total_cmp(&right.left()))
}

pub fn matching_border_rect(
    harness: &egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
    title_rect: egui::Rect,
) -> Option<egui::Rect> {
    border_rects(harness).into_iter().find(|rect| {
        rect.contains(title_rect.center())
            && rect.width() > title_rect.width()
            && (rect.height() - title_rect.height()).abs() <= 2.0
    })
}

pub fn document_tab_border(
    harness: &egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
    label: &str,
) -> egui::Rect {
    let title_rect = topmost_label_rect(harness, label);
    matching_border_rect(harness, title_rect)
        .unwrap_or_else(|| panic!("document tab parent border not found for {label}"))
}

pub fn topmost_label_rect(
    harness: &egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
    label: &str,
) -> egui::Rect {
    label_rects(harness, label)
        .into_iter()
        .min_by(|left, right| left.top().total_cmp(&right.top()))
        .unwrap_or_else(|| panic!("label not found: {label}"))
}

pub fn open_document_names(
    harness: &mut egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
) -> Vec<String> {
    harness
        .state_mut()
        .app_state_mut()
        .document
        .open_documents
        .iter()
        .map(|document| document.file_name().unwrap_or("untitled").to_string())
        .collect()
}

fn border_rects(
    harness: &egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
) -> Vec<egui::Rect> {
    flatten_clipped_shapes(&harness.output().shapes)
        .into_iter()
        .filter_map(|shape| match shape {
            egui::epaint::Shape::Rect(rect_shape)
                if rect_shape.stroke.width > 0.0
                    && rect_shape.fill == egui::Color32::TRANSPARENT =>
            {
                Some(rect_shape.rect)
            }
            _ => None,
        })
        .collect()
}

fn label_rects(
    harness: &egui_kittest::Harness<'_, katana_ui::shell::KatanaApp>,
    label: &str,
) -> Vec<egui::Rect> {
    harness
        .query_all_by_label(label)
        .map(|node| node.rect())
        .collect()
}

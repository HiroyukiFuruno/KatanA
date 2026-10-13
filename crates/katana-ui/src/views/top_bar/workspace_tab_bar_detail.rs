use crate::app_state::AppAction;
use crate::views::top_bar::tab_border::TabBorderOps;
use crate::views::top_bar::workspace_tab_bar_detail_layout::WorkspaceTabBarDetailLayout;
use eframe::egui;

const WORKSPACE_TAB_MIN_WIDTH: f32 = 140.0;
const WORKSPACE_TAB_CORNER_RADIUS: u8 = 4;

pub(crate) struct WorkspaceTabBarDetail;

impl WorkspaceTabBarDetail {
    pub(crate) fn render_workspace_tab(
        ui: &mut egui::Ui,
        path: &str,
        index: usize,
        is_active: bool,
        tab_rect: egui::Rect,
        action: &mut Option<AppAction>,
    ) -> egui::Response {
        let title = Self::workspace_name(path);
        let tab_hovered = TabBorderOps::rect_contains_pointer(ui, tab_rect);
        let response = ui.interact(
            tab_rect,
            egui::Id::new("workspace_tab_interact")
                .with(index)
                .with(path),
            egui::Sense::click_and_drag(),
        );
        let close_response =
            Self::render_parent_tab(ui, path, title, is_active, tab_rect, tab_hovered);
        let corner_radius = Self::tab_corner_radius();
        TabBorderOps::paint_with_radius(ui, tab_rect, tab_hovered, corner_radius);
        if response.clicked() && !is_active {
            *action = Some(AppAction::SelectWorkspaceTab(std::path::PathBuf::from(
                path,
            )));
        }
        if close_response.clicked() {
            *action = Some(AppAction::CloseWorkspaceTab(std::path::PathBuf::from(path)));
        }
        response
    }

    fn render_parent_tab(
        ui: &mut egui::Ui,
        path: &str,
        title: String,
        is_active: bool,
        tab_rect: egui::Rect,
        tab_hovered: bool,
    ) -> egui::Response {
        let content_rect = WorkspaceTabBarDetailLayout::content_rect(tab_rect, tab_rect.height());
        let close_width = ui.spacing().interact_size.x.min(content_rect.width());
        let close_rect = egui::Rect::from_min_size(
            egui::pos2(content_rect.right() - close_width, content_rect.top()),
            egui::vec2(close_width, content_rect.height()),
        );
        let title_rect = egui::Rect::from_min_max(
            content_rect.min,
            egui::pos2(close_rect.left(), content_rect.bottom()),
        );
        ui.push_id(format!("workspace_tab_{path}"), |ui| {
            Self::render_title(ui, title_rect, title, is_active).on_hover_text(path);
            Self::render_close_button(ui, path, close_rect, tab_hovered)
        })
        .inner
    }

    pub(crate) fn render_plus_button(
        ui: &mut egui::Ui,
        plus_rect: egui::Rect,
        action: &mut Option<AppAction>,
    ) {
        let label = crate::i18n::I18nOps::get().menu.open_workspace.clone();
        let response = ui
            .put(
                plus_rect,
                crate::Icon::Plus.button(ui, crate::icon::IconSize::Small),
            )
            .on_hover_text(label.clone());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label.clone())
        });
        if response.clicked() {
            *action = Some(AppAction::PickOpenWorkspace);
        }
    }

    pub(super) fn render_title(
        ui: &mut egui::Ui,
        title_rect: egui::Rect,
        title: String,
        is_active: bool,
    ) -> egui::Response {
        let response = ui.interact(title_rect, ui.next_auto_id(), egui::Sense::hover());
        let icon_width = crate::icon::IconSize::Small.to_vec2().x;
        let gap = WorkspaceTabBarDetailLayout::icon_gap();
        let text_width = Self::title_text_width(ui, &title);
        let max_group_width = WorkspaceTabBarDetailLayout::max_group_width(title_rect.width());
        let group_width = (icon_width + gap + text_width).min(max_group_width);
        let group_rect = WorkspaceTabBarDetailLayout::title_group_rect(title_rect, group_width);
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(
                group_rect.left() + icon_width / 2.0,
                group_rect.center().y + WorkspaceTabBarDetailLayout::icon_baseline_offset_y(),
            ),
            crate::icon::IconSize::Small.to_vec2(),
        );
        ui.put(
            icon_rect,
            crate::Icon::FolderOpen.ui_image(ui, crate::icon::IconSize::Small),
        );
        let available_label_width = (group_width - icon_width - gap).max(0.0);
        let label_width = available_label_width.min(text_width);
        let label_rect = egui::Rect::from_center_size(
            egui::pos2(
                group_rect.left() + icon_width + gap + label_width / 2.0,
                group_rect.center().y,
            ),
            egui::vec2(label_width, title_rect.height()),
        );
        let text = egui::RichText::new(title)
            .color(Self::title_color(ui, is_active))
            .text_style(egui::TextStyle::Button);
        let label = egui::Label::new(text).selectable(false);
        let label = if text_width > available_label_width {
            label.truncate()
        } else {
            label
        };
        ui.put(label_rect, label);
        response
    }

    fn title_text_width(ui: &egui::Ui, title: &str) -> f32 {
        let font_id = egui::TextStyle::Button.resolve(ui.style());
        ui.painter()
            .layout_no_wrap(title.to_owned(), font_id, ui.visuals().text_color())
            .size()
            .x
    }

    fn title_color(ui: &egui::Ui, is_active: bool) -> egui::Color32 {
        if is_active {
            ui.visuals().selection.stroke.color
        } else {
            ui.visuals().widgets.inactive.fg_stroke.color
        }
    }

    pub(crate) fn workspace_name(path: &str) -> String {
        std::path::Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| path.to_string())
    }

    pub(crate) fn tab_width(content_width: f32, tab_count: usize, plus_button_width: f32) -> f32 {
        if tab_count == 0 {
            return 0.0;
        }
        ((content_width - plus_button_width).max(0.0) / tab_count as f32)
            .max(WORKSPACE_TAB_MIN_WIDTH)
    }

    pub(crate) fn content_width(
        available_width: f32,
        tab_count: usize,
        plus_button_width: f32,
    ) -> f32 {
        let minimum_tab_area = WORKSPACE_TAB_MIN_WIDTH * tab_count as f32;
        available_width.max(minimum_tab_area + plus_button_width)
    }

    pub(crate) fn tab_corner_radius() -> egui::CornerRadius {
        egui::CornerRadius::same(WORKSPACE_TAB_CORNER_RADIUS)
    }

    pub(crate) fn row_height(ui: &egui::Ui) -> f32 {
        ui.text_style_height(&egui::TextStyle::Button)
    }
}

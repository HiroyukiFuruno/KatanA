use eframe::egui::{self, Rect};
use katana_core::canvas::{CanvasNode, NodeContent};

use super::{CanvasSurfaceLabels, colors::CanvasColors};

#[path = "accessibility.rs"]
mod accessibility;

pub(super) use accessibility::CanvasLabels;

const CARD_PADDING: f32 = 8.0;
const CARD_CORNER_RADIUS: u8 = 5;
const CARD_BORDER_WIDTH: f32 = 1.0;

pub(super) struct CanvasCards;

impl CanvasCards {
    pub(super) fn draw_ui(
        ui: &egui::Ui,
        node: &CanvasNode,
        rect: Rect,
        labels: &CanvasSurfaceLabels<'_>,
    ) {
        let content = Self::content(node, labels);
        CanvasLabels::register_ui(ui, ui.id().with(("canvas-node", &node.id)), rect, &content);
        let painter = Self::frame_ui(ui, node, rect);
        let content_rect = rect.shrink(CARD_PADDING);
        if content_rect.is_positive() {
            Self::text_ui(ui, &painter, content, content_rect);
        }
    }

    fn frame_ui(ui: &egui::Ui, node: &CanvasNode, rect: Rect) -> egui::Painter {
        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        let stroke = egui::Stroke::new(
            CARD_BORDER_WIDTH,
            CanvasColors::resolve(
                node.color.as_ref(),
                ui.visuals().widgets.noninteractive.bg_stroke.color,
            ),
        );
        let group = matches!(node.content, NodeContent::Group { .. });
        if !group {
            painter.rect_filled(rect, CARD_CORNER_RADIUS, ui.visuals().extreme_bg_color);
        }
        /* WHY: 既存のShape経路でも同じRectShapeを生成し、Inside指定で枠を外形の内側に保つ。 */
        painter.add(egui::Shape::rect_stroke(
            rect,
            CARD_CORNER_RADIUS,
            stroke,
            egui::StrokeKind::Inside,
        ));
        painter
    }

    fn text_ui(ui: &egui::Ui, painter: &egui::Painter, content: String, rect: Rect) {
        /* WHY: 本文全体を保持し、カード領域の描画クリップだけで表示を制限する。 */
        let galley = painter.layout(
            content,
            egui::TextStyle::Body.resolve(ui.style()),
            ui.visuals().text_color(),
            rect.width(),
        );
        painter
            .with_clip_rect(rect.intersect(ui.clip_rect()))
            .galley(rect.min, galley, ui.visuals().text_color());
    }

    pub(super) fn content(node: &CanvasNode, labels: &CanvasSurfaceLabels<'_>) -> String {
        match &node.content {
            NodeContent::Text { text } => text.clone(),
            NodeContent::File { .. } => node.content.file_label().unwrap_or_default().to_owned(),
            NodeContent::Link { url } => format!("{}\n{url}", labels.unsupported_link),
            NodeContent::Unsupported { node_type } => {
                format!("{}\n{node_type}", labels.unsupported_node)
            }
            NodeContent::Group {
                label,
                background,
                background_style,
            } => Self::group_content(
                label.as_deref(),
                background.as_deref(),
                background_style.is_some(),
                labels,
            ),
        }
    }

    fn group_content(
        label: Option<&str>,
        background: Option<&str>,
        has_style: bool,
        labels: &CanvasSurfaceLabels<'_>,
    ) -> String {
        let label = label.unwrap_or_default();
        match background {
            Some(background) => format!("{label}\n{}\n{background}", labels.unsupported_background),
            None if has_style => format!("{label}\n{}", labels.unsupported_background),
            None => label.to_owned(),
        }
    }
}

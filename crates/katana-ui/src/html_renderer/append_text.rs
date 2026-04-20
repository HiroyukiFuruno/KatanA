use eframe::egui;
use eframe::egui::text::LayoutJob;
use katana_core::html::{HtmlNode, LinkAction};

use super::collect_text;
use super::types::HtmlRenderer;

impl<'a> HtmlRenderer<'a> {
    pub(super) fn append_text_node(&mut self, job: &mut LayoutJob, node: &HtmlNode) {
        match node {
            HtmlNode::Text(text) => {
                let mut rich = egui::RichText::new(text.as_str());
                if self.is_strong {
                    rich = rich.strong();
                }
                if self.is_italics {
                    rich = rich.italics();
                }
                if let Some(color) = self.text_color {
                    rich = rich.color(color);
                }
                rich.append_to(
                    job,
                    self.ui.style().as_ref(),
                    egui::FontSelection::Default,
                    egui::Align::Center,
                );
            }
            HtmlNode::Emphasis(children) => {
                let prev = self.is_italics;
                self.is_italics = true;
                for child in children {
                    self.append_text_node(job, child);
                }
                self.is_italics = prev;
            }
            HtmlNode::Strong(children) => {
                let prev = self.is_strong;
                self.is_strong = true;
                for child in children {
                    self.append_text_node(job, child);
                }
                self.is_strong = prev;
            }
            HtmlNode::Link { children, .. } => {
                /* WHY: Link within a pure text flow - unfortunately, it can't be clickable when appended to a LayoutJob natively
                because LayoutJob creates a single Label widget. In katana, when links occur in text sequences
                they are parsed but effectively unclickable unless we split nodes.
                We format it as a link visually. */
                let mut s = String::new();
                for node in children {
                    if let HtmlNode::Text(t) = node {
                        s.push_str(t);
                    }
                }
                if !s.is_empty() {
                    let mut rt = egui::RichText::new(s)
                        .underline()
                        .color(self.ui.visuals().hyperlink_color);
                    if self.is_strong {
                        rt = rt.strong();
                    }
                    if self.is_italics {
                        rt = rt.italics();
                    }
                    rt.append_to(
                        job,
                        self.ui.style().as_ref(),
                        egui::FontSelection::Default,
                        egui::Align::Center,
                    );
                } else {
                    for child in children {
                        self.append_text_node(job, child);
                    }
                }
            }
            HtmlNode::LineBreak => {
                let mut rich = egui::RichText::new("\n");
                if let Some(color) = self.text_color {
                    rich = rich.color(color);
                }
                rich.append_to(
                    job,
                    self.ui.style().as_ref(),
                    egui::FontSelection::Default,
                    egui::Align::Center,
                );
            }
            _ => {}
        }
    }

    pub(super) fn flush_centered_inline_batch(
        &mut self,
        batch: &[&HtmlNode],
        batch_index: usize,
    ) -> Option<LinkAction> {
        if batch.is_empty() {
            return None;
        }

        if batch.iter().copied().all(Self::can_use_layout_job) {
            self.render_text_batch(batch, true);
            return None;
        }

        if batch.len() == 1 {
            let mut action = None;
            self.ui.vertical_centered(|ui| {
                let mut inner = HtmlRenderer::new_inner(ui, self.text_color, self.max_image_width);
                action = inner.render_inline(batch[0]);
            });
            return action;
        }

        let mut action = None;

        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write_usize(batch_index);
        hasher.write_usize(batch.len());
        let text_content = batch
            .iter()
            .map(|n| collect_text(std::slice::from_ref(*n)))
            .collect::<String>();
        hasher.write(text_content.as_bytes());
        let hash = hasher.finish();

        let id = self.ui.id().with("centered_batch").with(hash);

        let mut memorized = true;
        let bounds = self.ui.available_rect_before_wrap();
        let content_size: egui::Vec2 =
            self.ui.ctx().data(|r| r.get_temp(id)).unwrap_or_else(|| {
                memorized = false;
                bounds.size()
            });

        let centered_rect = egui::Align2::CENTER_TOP.align_size_within_rect(content_size, bounds);
        let layout = egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(false);
        let child_max_rect = egui::Rect::from_min_size(
            centered_rect.min,
            egui::vec2(bounds.width(), bounds.height()),
        );
        let builder = egui::UiBuilder::new()
            .max_rect(child_max_rect)
            .layout(layout);

        let mut child_ui = self.ui.new_child(builder);
        const HORIZONTAL_ITEM_SPACING: f32 = 4.0;
        child_ui.spacing_mut().item_spacing.x = HORIZONTAL_ITEM_SPACING;

        for node in batch {
            let mut inner =
                HtmlRenderer::new_inner(&mut child_ui, self.text_color, self.max_image_width);
            if let Some(a) = inner.render_inline(node) {
                action = Some(a);
            }
        }

        let new_size = child_ui.min_size();
        if new_size != content_size || !memorized {
            self.ui.ctx().data_mut(|w| w.insert_temp(id, new_size));
        }

        let row_height = new_size.y;
        self.ui
            .allocate_space(egui::vec2(bounds.width(), row_height));

        action
    }
}

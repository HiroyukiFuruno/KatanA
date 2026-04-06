use eframe::egui;
use eframe::egui::text::LayoutJob;
use katana_core::html::{HtmlNode, LinkAction, TextAlign};

use super::types::HtmlRenderer;
use super::{
    HEADING_H2_SIZE, HEADING_H3_SIZE, HEADING_LEVEL_1, HEADING_LEVEL_2, HEADING_LEVEL_3,
    collect_text,
};

fn build_heading_rt(text: &str, level: u8) -> egui::RichText {
    if level == HEADING_LEVEL_1 {
        egui::RichText::new(text).heading()
    } else if level == HEADING_LEVEL_2 {
        egui::RichText::new(text).strong().size(HEADING_H2_SIZE)
    } else if level == HEADING_LEVEL_3 {
        egui::RichText::new(text).strong().size(HEADING_H3_SIZE)
    } else {
        egui::RichText::new(text).strong()
    }
}

impl<'a> HtmlRenderer<'a> {
    pub(super) fn render_block(&mut self, node: &HtmlNode) -> Option<LinkAction> {
        match node {
            HtmlNode::Paragraph { align, children } => match align {
                Some(TextAlign::Center) => self.render_centered_children(children),
                _ => self.render_nodes(children),
            },
            HtmlNode::Heading {
                level,
                align,
                children,
            } => {
                let text = collect_text(children);
                let mut rt = build_heading_rt(&text, *level);
                if let Some(c) = self.text_color {
                    rt = rt.color(c);
                }

                match align {
                    Some(TextAlign::Center) => {
                        let avail_w = self.ui.available_width();
                        self.ui.allocate_ui_with_layout(
                            egui::vec2(avail_w, 0.0),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.set_width(avail_w);
                                ui.label(rt);
                            },
                        );
                    }
                    _ => {
                        self.ui.label(rt);
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub(super) fn can_use_layout_job(node: &HtmlNode) -> bool {
        match node {
            HtmlNode::Text(_) | HtmlNode::LineBreak => true,
            HtmlNode::Emphasis(children) | HtmlNode::Strong(children) => {
                children.iter().all(Self::can_use_layout_job)
            }
            _ => false,
        }
    }

    pub(super) fn render_text_batch(&mut self, batch: &[&HtmlNode], centered: bool) {
        let mut job = LayoutJob::default();
        job.wrap.max_width = self.ui.available_width();
        job.halign = if centered {
            egui::Align::Center
        } else {
            egui::Align::LEFT
        };

        for node in batch {
            self.append_text_node(&mut job, node);
        }

        let label = egui::Label::new(job).wrap();
        if centered {
            self.ui
                .add_sized(egui::vec2(self.ui.available_width(), 0.0), label);
        } else {
            self.ui.add(label);
        }
    }

    pub(super) fn render_centered_children(&mut self, children: &[HtmlNode]) -> Option<LinkAction> {
        let mut action: Option<LinkAction> = None;
        let mut inline_batch: Vec<&HtmlNode> = Vec::new();
        let mut batch_index: usize = 0;

        for node in children {
            if node.is_block() {
                if let Some(a) = self.flush_centered_inline_batch(&inline_batch, batch_index) {
                    action = Some(a);
                }
                batch_index += 1;
                inline_batch.clear();
                if let Some(a) = self.render_block(node) {
                    action = Some(a);
                }
            } else {
                inline_batch.push(node);
            }
        }

        if let Some(a) = self.flush_centered_inline_batch(&inline_batch, batch_index) {
            action = Some(a);
        }
        action
    }

    pub(super) fn flush_inline_batch(&mut self, batch: &[&HtmlNode]) -> Option<LinkAction> {
        if batch.is_empty() {
            return None;
        }

        if batch.iter().copied().all(Self::can_use_layout_job) {
            self.render_text_batch(batch, false);
            return None;
        }

        if batch.len() == 1 {
            return self.render_inline(batch[0]);
        }

        let mut action = None;
        self.ui.horizontal_wrapped(|ui| {
            for node in batch {
                let mut inner = HtmlRenderer::new_inner(ui, self.text_color, self.max_image_width);
                inner.is_strong = self.is_strong;
                inner.is_italics = self.is_italics;
                if let Some(a) = inner.render_inline(node) {
                    action = Some(a);
                }
            }
        });
        action
    }
}

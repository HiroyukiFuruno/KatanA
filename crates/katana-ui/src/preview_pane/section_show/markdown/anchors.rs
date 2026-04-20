/* WHY: Isolated anchor and span processing logic to manage complexity and satisfy architectural line limits. */

use super::super::render_utils::SectionRenderUtilsOps;
use eframe::egui;

pub struct MarkdownAnchorOps;

impl MarkdownAnchorOps {
    #[allow(clippy::too_many_arguments)]
    pub fn process_anchors(
        md: &str,
        global_line_offset: usize,
        heading_anchors: &mut Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
        block_anchors: &mut Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
        hovered_lines: &mut Option<&mut Vec<std::ops::Range<usize>>>,
        previous_anchor_count: usize,
        local_block_anchors: Vec<(std::ops::Range<usize>, egui::Rect)>,
        local_hovered_spans: Vec<std::ops::Range<usize>>,
    ) {
        if let Some(anchors) = heading_anchors.as_mut() {
            for anchor in &mut anchors[previous_anchor_count..] {
                anchor.0 = SectionRenderUtilsOps::span_to_range(
                    md,
                    &anchor.0,
                    global_line_offset,
                    false,
                    false,
                );
            }
        }
        if let Some(anchors) = block_anchors {
            for (local_span, rect) in local_block_anchors {
                anchors.push((
                    SectionRenderUtilsOps::span_to_range(
                        md,
                        &local_span,
                        global_line_offset,
                        true,
                        false,
                    ),
                    rect,
                ));
            }
        }
        if let Some(hovered) = hovered_lines {
            for local_span in local_hovered_spans {
                hovered.push(SectionRenderUtilsOps::span_to_range(
                    md,
                    &local_span,
                    global_line_offset,
                    true,
                    true,
                ));
            }
        }
    }
}

use super::types::*;

impl<'a> TocPanel<'a> {
    #[cfg(test)]
    pub(crate) fn find_active_toc_index_preview(
        outline_items: &[katana_core::markdown::outline::OutlineItem],
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        threshold: f32,
    ) -> usize {
        let mut active_index = outline_items.first().map(|i| i.index).unwrap_or(0);
        if let Some(item) = Self::find_anchor_for_preview_threshold(anchor_map, threshold)
            && let Some(toc_index) = item.toc_index
        {
            active_index = toc_index;
        }
        active_index
    }

    pub(crate) fn find_anchor_for_preview_threshold(
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        threshold: f32,
    ) -> Option<&crate::preview_pane::types::DocumentAnchorMapItem> {
        let mut active_anchor = None;
        for item in anchor_map {
            let Some(rect) = item.rect else {
                continue;
            };
            if rect.min.y <= threshold {
                active_anchor = Some(item);
            } else {
                break;
            }
        }
        active_anchor
    }

    #[cfg(test)]
    pub(crate) fn find_active_toc_index_editor(
        outline_items: &[katana_core::markdown::outline::OutlineItem],
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        logical_threshold: f32,
    ) -> usize {
        let line = logical_threshold.floor().max(0.0) as usize;
        if let Some(item) = Self::find_anchor_for_line(anchor_map, line)
            && let Some(toc_index) = item.toc_index
        {
            return toc_index;
        }
        outline_items.first().map(|i| i.index).unwrap_or(0)
    }

    pub(crate) fn find_anchor_for_line(
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        line: usize,
    ) -> Option<&crate::preview_pane::types::DocumentAnchorMapItem> {
        if let Some(item) = anchor_map
            .iter()
            .find(|item| Self::line_span_contains_line(&item.line_span, line))
        {
            return Some(item);
        }
        anchor_map
            .iter()
            .take_while(|item| item.line_span.start <= line)
            .last()
    }

    pub(crate) fn find_anchor_for_line_range<'b>(
        anchor_map: &'b [crate::preview_pane::types::DocumentAnchorMapItem],
        line_range: &std::ops::Range<usize>,
    ) -> Option<&'b crate::preview_pane::types::DocumentAnchorMapItem> {
        Self::find_anchor_for_line(anchor_map, line_range.start).or_else(|| {
            anchor_map
                .iter()
                .find(|item| Self::line_spans_overlap(&item.line_span, line_range))
        })
    }

    pub(crate) fn editor_logical_threshold(
        scroll: &crate::app_state::ScrollState,
        row_height: f32,
    ) -> f32 {
        if let Some(target_line) = scroll.toc_scroll_to_line {
            return target_line as f32 + 1.0;
        }
        if let Some(target_line) = scroll.scroll_to_line {
            return target_line as f32 + 1.0;
        }
        if !scroll.editor_line_anchors.is_empty() {
            let visible_line_count = scroll
                .editor_line_anchors
                .partition_point(|anchor_y| *anchor_y <= scroll.editor_y);
            return visible_line_count as f32;
        }
        (scroll.editor_y / row_height) + 1.0
    }

    #[cfg(test)]
    pub(crate) fn should_auto_scroll_active_item(
        previous_active_index: Option<usize>,
        active_index: usize,
    ) -> bool {
        previous_active_index != Some(active_index)
    }

    fn line_span_contains_line(line_span: &std::ops::Range<usize>, line: usize) -> bool {
        let end = line_span.end.max(line_span.start + 1);
        line >= line_span.start && line < end
    }

    fn line_spans_overlap(
        item_span: &std::ops::Range<usize>,
        line_range: &std::ops::Range<usize>,
    ) -> bool {
        let item_end = item_span.end.max(item_span.start + 1);
        let line_end = line_range.end.max(line_range.start + 1);
        item_span.start < line_end && line_range.start < item_end
    }
}

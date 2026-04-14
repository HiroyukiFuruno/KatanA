use super::types::*;

impl<'a> TocPanel<'a> {
    pub fn new(
        preview: &'a mut crate::preview_pane::PreviewPane,
        state: &'a mut crate::app_state::AppState,
    ) -> Self {
        Self { preview, state }
    }

    pub(crate) fn find_active_toc_index_preview(
        outline_items: &[katana_core::markdown::outline::OutlineItem],
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        threshold: f32,
    ) -> usize {
        let mut active_index = outline_items.first().map(|i| i.index).unwrap_or(0);
        for item in anchor_map {
            if matches!(
                item.kind,
                katana_core::markdown::outline::AnchorKind::Heading
            ) && let Some(idx) = item.index
                && let Some(rect) = item.rect
            {
                if rect.min.y <= threshold {
                    active_index = idx;
                } else {
                    break;
                }
            }
        }
        active_index
    }

    pub(crate) fn find_active_toc_index_editor(
        outline_items: &[katana_core::markdown::outline::OutlineItem],
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        logical_threshold: f32,
    ) -> usize {
        let mut active_index = outline_items.first().map(|i| i.index).unwrap_or(0);
        for item in anchor_map {
            if matches!(
                item.kind,
                katana_core::markdown::outline::AnchorKind::Heading
            ) && let Some(idx) = item.index
            {
                if (item.line_span.start as f32) > logical_threshold {
                    break;
                }
                active_index = idx;
            }
        }
        active_index
    }
}

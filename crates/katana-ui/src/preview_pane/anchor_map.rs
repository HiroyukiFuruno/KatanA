use super::types::DocumentAnchorMapItem;
use eframe::egui;

impl DocumentAnchorMapItem {
    pub fn from_document_anchors(
        document_anchors: &[katana_core::markdown::outline::DocumentAnchor],
    ) -> Vec<Self> {
        let mut current_heading_index = 0;
        document_anchors
            .iter()
            .map(|a| {
                let index = if matches!(a.kind, katana_core::markdown::outline::AnchorKind::Heading)
                {
                    let idx = current_heading_index;
                    current_heading_index += 1;
                    Some(idx)
                } else {
                    None
                };
                Self {
                    kind: a.kind.clone(),
                    index,
                    line_span: a.line_start..a.line_end,
                    rect: None,
                }
            })
            .collect()
    }

    pub fn sync_rects(
        anchor_map: &mut [Self],
        heading_anchors: &[(std::ops::Range<usize>, egui::Rect)],
        block_anchors: &[(std::ops::Range<usize>, egui::Rect)],
    ) {
        for item in anchor_map.iter_mut() {
            item.rect = None;
        }

        for (span, rect) in heading_anchors {
            if let Some(item) = anchor_map.iter_mut().find(|a| {
                matches!(a.kind, katana_core::markdown::outline::AnchorKind::Heading)
                    && a.line_span.start.abs_diff(span.start) <= 1
            }) {
                item.rect = Some(item.rect.map_or(*rect, |r| r.union(*rect)));
            }
        }
        for (span, rect) in block_anchors {
            if let Some(item) = anchor_map.iter_mut().find(|a| {
                !matches!(a.kind, katana_core::markdown::outline::AnchorKind::Heading)
                    && a.line_span.start <= span.start
                    && a.line_span.end >= span.start
            }) {
                item.rect = Some(item.rect.map_or(*rect, |r| r.union(*rect)));
            }
            if let Some(item) = anchor_map.iter_mut().find(|a| {
                matches!(a.kind, katana_core::markdown::outline::AnchorKind::Heading)
                    && a.rect.is_none()
                    && a.line_span.start <= span.start
                    && a.line_span.end >= span.start
            }) {
                item.rect = Some(*rect);
            }
        }
    }
}

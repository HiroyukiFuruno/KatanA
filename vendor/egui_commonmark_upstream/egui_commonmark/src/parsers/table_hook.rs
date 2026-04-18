use crate::parsers::pulldown::CommonMarkViewerInternal;
use egui::Ui;
use egui_commonmark_backend::misc::{CommonMarkCache, CommonMarkOptions};
use egui_commonmark_backend::pulldown::{parse_table, EventIteratorItem};
use std::iter::Peekable;

pub(crate) fn handle_custom_table<'a, 'e>(
    viewer: &mut CommonMarkViewerInternal<'a>,
    ui: &mut Ui,
    events: &mut Peekable<impl Iterator<Item = EventIteratorItem<'e>>>,
    cache: &mut CommonMarkCache,
    options: &CommonMarkOptions<'e>,
    max_width: f32,
) -> bool {
    if viewer.table_alignments.is_none() || options.table_fn.is_none() {
        return false;
    }

    let alignments = viewer.table_alignments.take().unwrap();
    let table_data = parse_table(events);

    if let Some(table_fn) = &options.table_fn {
        let table_res = table_fn(
            ui,
            cache,
            options,
            table_data,
            &alignments,
            max_width,
            &mut |inner_ui, inner_cache, block_events| {
                let cell_max_width = inner_ui.available_width();
                for (i, (e, src_span)) in block_events.iter() {
                    viewer.current_event_idx = *i;
                    viewer.event(
                        inner_ui,
                        e.clone(),
                        src_span.clone(),
                        inner_cache,
                        options,
                        cell_max_width,
                    );
                }
                viewer.flush_pending_inline(inner_ui, cell_max_width);
            },
        );

        if let Some((_start_y, span)) = viewer.block_states.pop() {
            let mut rect = table_res.rect;
            // Fix: ensure the rect doesn't expand vertically beyond the table.
            // The table_res.rect (from ScrollArea's InnerResponse) should already be correct,
            // but we expand horizontally to match Katana's full-width block selection style.
            rect.min.x = ui.max_rect().min.x;
            rect.max.x = ui.max_rect().max.x;

            if let Some(active) = &viewer.active_char_range {
                if active.start <= span.end && active.end >= span.start {
                    viewer.active_rects.push((rect, span.clone()));

                    /* WHY: Optical visibility for diagrams/tables. Match pulldown.rs value. */
                    const RECT_ACTIVE_ALPHA: u8 = 40;
                    let highlight_color = viewer.active_bg_color.unwrap_or_else(|| {
                        if ui.visuals().dark_mode {
                            egui::Color32::from_white_alpha(RECT_ACTIVE_ALPHA)
                        } else {
                            egui::Color32::from_black_alpha(RECT_ACTIVE_ALPHA)
                        }
                    });
                    ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, highlight_color);
                }
            }
            if let Some(hovered) = &mut viewer.hovered_spans {
                if let Some(pos) = ui.ctx().pointer_hover_pos() {
                    if rect.contains(pos) {
                        hovered.push(span.clone());
                    }
                }
            }
        }

        return true;
    }
    false
}

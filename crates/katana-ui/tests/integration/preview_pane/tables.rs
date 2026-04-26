use crate::integration::harness_utils::flatten_clipped_shapes;
use eframe::egui;
use katana_ui::preview_pane::PreviewPane;
use std::path::Path;

fn extract_section(source: &str, start_marker: &str, end_marker: &str) -> String {
    let start_pos = source.find(start_marker).unwrap() + start_marker.len();
    let after_start = source[start_pos..]
        .find('\n')
        .map(|p| start_pos + p + 1)
        .unwrap_or(start_pos);
    let end_pos = source[after_start..]
        .find(end_marker)
        .map(|p| after_start + p)
        .unwrap_or(source.len());
    source[after_start..end_pos].trim().to_string()
}

#[derive(Clone)]
struct TextBounds {
    text: String,
    rect: egui::Rect,
}

fn render_section_shapes(
    start_marker: &str,
    end_marker: &str,
    preview_width: f32,
    preview_height: f32,
) -> Vec<egui::epaint::Shape> {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixtures/sample.md"),
    )
    .expect("failed to read sample.md");
    let section_md = extract_section(&source, start_marker, end_marker);

    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(&section_md, Path::new("/tmp/sample_section.md"));

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(preview_width, preview_height),
        )),
        ..Default::default()
    };

    // Render twice to stabilize one-frame-delayed geometry updates.
    let _ = ctx.run_ui(raw_input.clone(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| pane.show(ui));
    });
    let output = ctx.run_ui(raw_input, |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| pane.show(ui));
    });

    flatten_clipped_shapes(&output.shapes)
}

fn render_tables_block_shapes(preview_width: f32) -> Vec<egui::epaint::Shape> {
    render_section_shapes(
        "## 5. Tables (GFM)",
        "## 6. Blockquotes",
        preview_width,
        1600.0,
    )
}

fn raw_input_for_size(width: f32, height: f32) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(width, height),
        )),
        ..Default::default()
    }
}

fn find_text_bounds_contains<'a>(texts: &'a [TextBounds], needle: &str) -> &'a TextBounds {
    texts
        .iter()
        .find(|t| t.text.contains(needle))
        .expect("expected text marker was not rendered")
}

fn collect_text_bounds(shapes: &[egui::epaint::Shape]) -> Vec<TextBounds> {
    let mut texts = Vec::new();
    for shape in shapes {
        if let egui::epaint::Shape::Text(text) = shape {
            let text_str = text.galley.text().trim().to_string();
            if text_str.is_empty() {
                continue;
            }
            let rect = egui::Rect::from_min_max(
                egui::pos2(
                    text.pos.x + text.galley.rect.min.x,
                    text.pos.y + text.galley.rect.min.y,
                ),
                egui::pos2(
                    text.pos.x + text.galley.rect.max.x,
                    text.pos.y + text.galley.rect.max.y,
                ),
            );
            texts.push(TextBounds {
                text: text_str,
                rect,
            });
        }
    }
    texts
}

fn find_table_rect_for_markers(
    shapes: &[egui::epaint::Shape],
    texts: &[TextBounds],
    markers: &[&str],
) -> egui::Rect {
    let marker_centers: Vec<_> = markers
        .iter()
        .filter_map(|marker| {
            texts
                .iter()
                .find(|t| t.text.contains(*marker))
                .map(|t| t.rect.center())
        })
        .collect();

    assert_eq!(
        marker_centers.len(),
        markers.len(),
        "not all marker texts were rendered: {:?}",
        markers
    );

    let mut candidates = Vec::new();
    for shape in shapes {
        if let egui::epaint::Shape::Rect(rect_shape) = shape {
            if rect_shape.stroke.width <= 0.0 || rect_shape.rect.width() < 80.0 {
                continue;
            }
            if marker_centers
                .iter()
                .all(|center| rect_shape.rect.contains(*center))
            {
                candidates.push(rect_shape.rect);
            }
        }
    }

    candidates
        .into_iter()
        .max_by(|a, b| {
            let area_a = a.width() * a.height();
            let area_b = b.width() * b.height();
            area_a
                .partial_cmp(&area_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("failed to detect table border rectangle")
}

fn vertical_boundaries_in_table(
    shapes: &[egui::epaint::Shape],
    table_rect: egui::Rect,
) -> Vec<f32> {
    let mut xs = Vec::new();
    for shape in shapes {
        if let egui::epaint::Shape::LineSegment { points, .. } = shape {
            let is_vertical = (points[0].x - points[1].x).abs() < 1.0;
            let line_height = (points[0].y - points[1].y).abs();
            if !is_vertical || line_height < table_rect.height() * 0.4 {
                continue;
            }
            let x = points[0].x;
            let y_min = points[0].y.min(points[1].y);
            let y_max = points[0].y.max(points[1].y);
            if x > table_rect.left()
                && x < table_rect.right()
                && y_min <= table_rect.bottom()
                && y_max >= table_rect.top()
            {
                xs.push(x);
            }
        }
    }

    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut dedup: Vec<f32> = Vec::new();
    for x in xs {
        if dedup
            .last()
            .map(|prev| (x - *prev).abs() > 1.0)
            .unwrap_or(true)
        {
            dedup.push(x);
        }
    }
    dedup
}

fn horizontal_separators_in_table(
    shapes: &[egui::epaint::Shape],
    table_rect: egui::Rect,
) -> Vec<f32> {
    let mut ys = Vec::new();
    for shape in shapes {
        if let egui::epaint::Shape::LineSegment { points, .. } = shape {
            let is_horizontal = (points[0].y - points[1].y).abs() < 1.0;
            let line_width = (points[0].x - points[1].x).abs();
            if !is_horizontal || line_width < table_rect.width() * 0.8 {
                continue;
            }
            let y = points[0].y;
            if y > table_rect.top() && y < table_rect.bottom() {
                ys.push(y);
            }
        }
    }

    ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ys.dedup_by(|a, b| (*a - *b).abs() < 1.0);
    ys
}

fn col_for_text(text_rect: egui::Rect, edges: &[f32]) -> Option<usize> {
    let x = text_rect.center().x;
    (0..edges.len().saturating_sub(1)).find(|idx| x >= edges[*idx] && x <= edges[*idx + 1])
}

#[test]
fn assert_table_basic_has_balanced_distribution_and_padding() {
    let preview_width = 760.0;
    let shapes = render_section_shapes("### 5.1", "### 5.2", preview_width, 360.0);
    let texts = collect_text_bounds(&shapes);
    let table_rect =
        find_table_rect_for_markers(&shapes, &texts, &["Feature", "Markdown", "Notes"]);

    assert!(
        table_rect.left() >= 4.0 && table_rect.right() <= preview_width - 4.0,
        "table must stay inside panel with ~5px side margins: left={}, right={}, width={}",
        table_rect.left(),
        table_rect.right(),
        preview_width
    );

    let boundaries = vertical_boundaries_in_table(&shapes, table_rect);
    assert_eq!(
        boundaries.len(),
        2,
        "expected 2 vertical separators for 3 columns"
    );

    let edges = vec![
        table_rect.left(),
        boundaries[0],
        boundaries[1],
        table_rect.right(),
    ];
    let widths = [
        edges[1] - edges[0],
        edges[2] - edges[1],
        edges[3] - edges[2],
    ];
    let min_w = widths.iter().copied().fold(f32::INFINITY, f32::min);
    let max_w = widths.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        (max_w - min_w) <= 6.0,
        "columns must stay visually even when total width fits: widths={:?}",
        widths
    );

    let separators = horizontal_separators_in_table(&shapes, table_rect);
    let headers: Vec<_> = texts
        .iter()
        .filter(|t| t.text == "Feature" || t.text == "Status" || t.text == "Notes")
        .collect();
    let header_bottom = separators
        .iter()
        .copied()
        .find(|y| *y > headers.iter().map(|h| h.rect.max.y).fold(0.0, f32::max))
        .expect("failed to find header-bottom separator");

    for header in headers {
        let col_idx =
            col_for_text(header.rect, &edges).expect("header text should be inside a column");
        let left_pad = header.rect.left() - edges[col_idx];
        let right_pad = edges[col_idx + 1] - header.rect.right();
        assert!(
            left_pad >= 2.0 && right_pad >= 2.0,
            "header cell should keep horizontal margin (~2.5px): '{}' left_pad={} right_pad={}",
            header.text,
            left_pad,
            right_pad
        );

        let top_pad = header.rect.top() - table_rect.top();
        let bottom_pad = header_bottom - header.rect.bottom();
        assert!(
            top_pad >= 3.9 && bottom_pad >= 3.9,
            "header vertical padding should keep readable top/bottom space: '{}' top_pad={} bottom_pad={}",
            header.text,
            top_pad,
            bottom_pad
        );
    }
}

#[test]
fn assert_table_with_long_content_keeps_short_columns_visible() {
    let preview_width = 760.0;
    let shapes = render_section_shapes("### 5.5", "## 6. Blockquotes", preview_width, 360.0);
    let texts = collect_text_bounds(&shapes);
    let table_rect =
        find_table_rect_for_markers(&shapes, &texts, &["Long Column Test", "ID", "Notes"]);

    assert!(
        table_rect.left() >= 4.0 && table_rect.right() <= preview_width - 4.0,
        "table with long content must stay inside panel with ~5px side margins: left={}, right={}, width={}",
        table_rect.left(),
        table_rect.right(),
        preview_width
    );

    let boundaries = vertical_boundaries_in_table(&shapes, table_rect);
    assert_eq!(
        boundaries.len(),
        2,
        "expected 2 vertical separators for 3 columns"
    );

    let edges = vec![
        table_rect.left(),
        boundaries[0],
        boundaries[1],
        table_rect.right(),
    ];
    let widths = [
        edges[1] - edges[0],
        edges[2] - edges[1],
        edges[3] - edges[2],
    ];

    assert!(
        widths[0] >= 40.0 && widths[2] >= 40.0,
        "short side columns collapsed in the long-content table: widths={:?}",
        widths
    );
    assert!(
        widths[1] > widths[0] && widths[1] > widths[2],
        "long-content column should receive the largest width in the table: widths={:?}",
        widths
    );

    // Ensure short-cell strings fit in their columns with at least small horizontal padding.
    for target in ["ID", "Notes"] {
        let text = texts
            .iter()
            .find(|t| t.text == target)
            .expect("short-cell text missing");
        let col_idx =
            col_for_text(text.rect, &edges).expect("short-cell text should be inside a column");
        let left_pad = text.rect.left() - edges[col_idx];
        let right_pad = edges[col_idx + 1] - text.rect.right();
        assert!(
            left_pad >= 2.0 && right_pad >= 2.0,
            "short-cell text should not touch cell border: '{}' left_pad={} right_pad={}",
            target,
            left_pad,
            right_pad
        );
    }
}

#[test]
fn table_full_tables_block_keeps_vertical_margins() {
    let preview_width = 1200.0;
    let shapes = render_tables_block_shapes(preview_width);
    let texts = collect_text_bounds(&shapes);

    let basic_table_heading = find_text_bounds_contains(&texts, "5.1 Basic Table");
    let aligned_table_heading = find_text_bounds_contains(&texts, "5.2 Table with Alignment");
    let basic_table =
        find_table_rect_for_markers(&shapes, &texts, &["Feature", "Markdown", "Notes"]);

    assert!(
        basic_table.top() - basic_table_heading.rect.bottom() >= 4.0,
        "table top margin is missing: heading_bottom={} table_top={}",
        basic_table_heading.rect.bottom(),
        basic_table.top()
    );
    assert!(
        aligned_table_heading.rect.top() - basic_table.bottom() >= 4.0,
        "table bottom margin is missing: table_bottom={} next_heading_top={}",
        basic_table.bottom(),
        aligned_table_heading.rect.top()
    );
}

#[test]
fn assert_tables_follow_dynamic_width_allocation_rules() {
    let preview_width = 1200.0;
    let shapes = render_tables_block_shapes(preview_width);
    let texts = collect_text_bounds(&shapes);
    let basic_table =
        find_table_rect_for_markers(&shapes, &texts, &["Feature", "Markdown", "Notes"]);
    let basic_table_boundaries = vertical_boundaries_in_table(&shapes, basic_table);
    assert_eq!(
        basic_table_boundaries.len(),
        2,
        "expected 2 vertical separators in basic table"
    );
    let basic_table_edges = [
        basic_table.left(),
        basic_table_boundaries[0],
        basic_table_boundaries[1],
        basic_table.right(),
    ];
    let basic_table_widths = [
        basic_table_edges[1] - basic_table_edges[0],
        basic_table_edges[2] - basic_table_edges[1],
        basic_table_edges[3] - basic_table_edges[2],
    ];
    let basic_min_width = basic_table_widths
        .iter()
        .copied()
        .fold(f32::INFINITY, f32::min);
    let basic_max_width = basic_table_widths
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        (basic_max_width - basic_min_width) <= 8.0,
        "basic table columns lost balanced distribution: widths={:?}",
        basic_table_widths
    );

    let long_content_table =
        find_table_rect_for_markers(&shapes, &texts, &["Long Column Test", "ID"]);
    let long_content_table_boundaries = vertical_boundaries_in_table(&shapes, long_content_table);
    assert_eq!(
        long_content_table_boundaries.len(),
        2,
        "expected 2 vertical separators in long-content table"
    );
    let long_content_table_edges = [
        long_content_table.left(),
        long_content_table_boundaries[0],
        long_content_table_boundaries[1],
        long_content_table.right(),
    ];
    let long_content_table_widths = [
        long_content_table_edges[1] - long_content_table_edges[0],
        long_content_table_edges[2] - long_content_table_edges[1],
        long_content_table_edges[3] - long_content_table_edges[2],
    ];
    assert!(
        long_content_table_widths[0] >= 40.0 && long_content_table_widths[2] >= 40.0,
        "short side columns collapsed in long-content table: widths={:?}",
        long_content_table_widths
    );
    assert!(
        long_content_table_widths[1] > long_content_table_widths[0]
            && long_content_table_widths[1] > long_content_table_widths[2],
        "middle column should stay dominant for long content: widths={:?}",
        long_content_table_widths
    );
}

#[test]
fn assert_table_preview_width_shrinks_after_resize_with_table_and_code() {
    let markdown = r#"
### Table
| Feature | Status | Notes |
| --- | --- | --- |
| Markdown | ✅ | Full support |
| Mermaid | ✅ | Uses local Mermaid.js |

```rust
fn very_long_line_for_resize_regression() { let x = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"; println!("{x}"); }
```
"#;
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(markdown, Path::new("/tmp/table_code_resize.md"));

    let ctx = egui::Context::default();
    let widths = [1100.0_f32, 620.0_f32, 420.0_f32];
    for width in widths {
        for _ in 0..2 {
            let mut root_min_rect = egui::Rect::NOTHING;
            let _ = ctx.run_ui(raw_input_for_size(width, 500.0), |ctx| {
                egui::CentralPanel::default().show_inside(ctx, |ui| {
                    pane.show(ui);
                    root_min_rect = ui.min_rect();
                });
            });
            assert!(
                root_min_rect.width() <= width + 2.0,
                "preview min width should follow resize even with tables: min_rect={}, width={}",
                root_min_rect.width(),
                width
            );
        }
    }
}

#[test]
fn assert_tables_block_preview_width_shrinks_after_resize() {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixtures/sample.md"),
    )
    .expect("failed to read sample.md");
    let tables_md = extract_section(&source, "## 5. Tables (GFM)", "## 6. Blockquotes");

    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(&tables_md, Path::new("/tmp/tables_resize.md"));

    let ctx = egui::Context::default();
    let widths = [1200.0_f32, 760.0_f32, 520.0_f32, 380.0_f32];
    for width in widths {
        for _ in 0..2 {
            let mut root_min_rect = egui::Rect::NOTHING;
            let _ = ctx.run_ui(raw_input_for_size(width, 900.0), |ctx| {
                egui::CentralPanel::default().show_inside(ctx, |ui| {
                    pane.show(ui);
                    root_min_rect = ui.min_rect();
                });
            });
            assert!(
                root_min_rect.width() <= width + 2.0,
                "tables preview min width should follow resize: min_rect={}, width={}",
                root_min_rect.width(),
                width
            );
        }
    }
}

#[test]
fn assert_table_header_background_matches_border_bounds() {
    let preview_width = 700.0;
    let shapes = render_section_shapes("### 5.1", "### 5.2", preview_width, 360.0);
    let texts = collect_text_bounds(&shapes);
    let table_rect =
        find_table_rect_for_markers(&shapes, &texts, &["Feature", "Markdown", "Notes"]);
    let headers: Vec<_> = texts
        .iter()
        .filter(|t| t.text == "Feature" || t.text == "Status" || t.text == "Notes")
        .collect();
    let header_centers: Vec<_> = headers.iter().map(|t| t.rect.center()).collect();

    let header_bg = shapes
        .iter()
        .filter_map(|shape| {
            if let egui::epaint::Shape::Rect(r) = shape
                && r.fill != egui::Color32::TRANSPARENT
                && r.rect.height() < table_rect.height() * 0.6
                && header_centers.iter().all(|c| r.rect.contains(*c))
            {
                return Some(r.rect);
            }
            None
        })
        .min_by(|a, b| {
            (a.width() * a.height())
                .partial_cmp(&(b.width() * b.height()))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("header background rect was not detected");

    let header_covering_rects = shapes
        .iter()
        .filter_map(|shape| {
            if let egui::epaint::Shape::Rect(r) = shape
                && r.fill != egui::Color32::TRANSPARENT
                && r.rect.height() < table_rect.height() * 0.6
                && header_centers.iter().all(|c| r.rect.contains(*c))
            {
                return Some(r.rect);
            }
            None
        })
        .collect::<Vec<_>>();
    assert_eq!(
        header_covering_rects.len(),
        1,
        "header area should be painted by a single background rect"
    );

    assert!(
        (header_bg.left() - table_rect.left()).abs() <= 1.0
            && (header_bg.right() - table_rect.right()).abs() <= 1.0,
        "header background width should match border: bg={:?} border={:?}",
        header_bg,
        table_rect
    );
    assert!(
        (header_bg.top() - table_rect.top()).abs() <= 1.0,
        "header background top should match border top: bg_top={} border_top={}",
        header_bg.top(),
        table_rect.top()
    );
    let separators = horizontal_separators_in_table(&shapes, table_rect);
    let header_separator = separators
        .first()
        .copied()
        .expect("header separator was not detected");
    assert!(
        (header_bg.bottom() - header_separator).abs() <= 1.0,
        "header background bottom should meet header separator: bg_bottom={} separator={}",
        header_bg.bottom(),
        header_separator
    );
}

#[test]
fn assert_table_multiline_row_short_cells_have_consistent_vertical_alignment() {
    let preview_width = 420.0;
    let shapes = render_section_shapes("### 5.5", "## 6. Blockquotes", preview_width, 420.0);
    let texts = collect_text_bounds(&shapes);
    let table_rect =
        find_table_rect_for_markers(&shapes, &texts, &["Long Column Test", "ID", "Notes"]);
    let separators = horizontal_separators_in_table(&shapes, table_rect);
    let body_top = separators
        .first()
        .copied()
        .expect("header/body separator was not detected");
    let body_bottom = table_rect.bottom();

    let id = texts
        .iter()
        .find(|t| t.text == "ID")
        .expect("ID text was not rendered");
    let notes = texts
        .iter()
        .find(|t| t.text == "Notes")
        .expect("Notes text was not rendered");

    let id_top_pad = id.rect.top() - body_top;
    let notes_top_pad = notes.rect.top() - body_top;
    let id_bottom_pad = body_bottom - id.rect.bottom();
    let notes_bottom_pad = body_bottom - notes.rect.bottom();
    assert!(
        (id_top_pad - notes_top_pad).abs() <= 2.0,
        "short cells should use consistent vertical alignment in multiline rows: id_pad={} notes_pad={}",
        id_top_pad,
        notes_top_pad
    );
    assert!(
        id_top_pad <= id_bottom_pad && notes_top_pad <= notes_bottom_pad,
        "short cells should stay top-aligned in multiline rows: id_top={} id_bottom={} notes_top={} notes_bottom={}",
        id_top_pad,
        id_bottom_pad,
        notes_top_pad,
        notes_bottom_pad
    );
}

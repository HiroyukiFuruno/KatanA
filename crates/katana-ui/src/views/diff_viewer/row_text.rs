use super::style::{DiffTone, DiffViewerPalette};
use eframe::egui;

const TEXT_EXTRA_PADDING: f32 = 12.0;
const EMPTY_LINE_MARKER: &str = "↵";
const SPACE_MARKER: &str = "·";
const TAB_MARKER: &str = "→";

pub(super) struct DiffViewerTextOps;

impl DiffViewerTextOps {
    pub(super) fn segments(
        text: &str,
        highlight_ranges: &[crate::diff_review::TextRange],
        tone: DiffTone,
    ) -> Vec<TextSegment> {
        if text.is_empty() && matches!(tone, DiffTone::Removed | DiffTone::Added) {
            return vec![TextSegment::highlight_marker(EMPTY_LINE_MARKER)];
        }

        let chars = text.chars().collect::<Vec<_>>();
        let ranges = Self::merged_ranges(highlight_ranges, chars.len());
        if ranges.is_empty() {
            return vec![TextSegment::normal(text)];
        }

        let mut segments = Vec::new();
        let mut cursor = 0;
        for range in ranges {
            if cursor < range.start {
                segments.push(TextSegment::from_chars(&chars[cursor..range.start], false));
            }
            segments.push(TextSegment::highlighted(&chars[range.start..range.end]));
            cursor = range.end;
        }
        if cursor < chars.len() {
            segments.push(TextSegment::from_chars(&chars[cursor..], false));
        }
        segments
    }

    pub(super) fn display_width(segments: &[TextSegment]) -> f32 {
        let text = segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<String>();
        text_display_width(&text)
    }

    pub(super) fn show(
        ui: &mut egui::Ui,
        segment: &TextSegment,
        tone: DiffTone,
        palette: &DiffViewerPalette,
    ) {
        let background = segment_background(segment, tone, palette);
        let rich = egui::RichText::new(&segment.text)
            .monospace()
            .color(palette.text_for(tone));
        let response = egui::Frame::NONE
            .fill(background)
            .inner_margin(egui::Margin::symmetric(0, 0))
            .show(ui, |ui| ui.add(egui::Label::new(rich)))
            .response;

        if matches!(tone, DiffTone::Removed) && segment.is_highlighted {
            super::row_wave::DiffViewerWaveOps::paint_removed(ui, response.rect, palette);
        }
    }

    fn merged_ranges(
        highlight_ranges: &[crate::diff_review::TextRange],
        text_len: usize,
    ) -> Vec<crate::diff_review::TextRange> {
        let mut ranges = highlight_ranges
            .iter()
            .map(|range| {
                crate::diff_review::TextRange::new(
                    range.start.min(text_len),
                    range.end.min(text_len),
                )
            })
            .filter(|range| range.start < range.end)
            .collect::<Vec<_>>();
        ranges.sort_by_key(|range| range.start);
        merge_ranges(ranges)
    }
}

pub(super) struct TextSegment {
    text: String,
    is_highlighted: bool,
}

impl TextSegment {
    fn normal(text: &str) -> Self {
        Self {
            text: text.to_string(),
            is_highlighted: false,
        }
    }

    fn from_chars(chars: &[char], is_highlighted: bool) -> Self {
        Self {
            text: chars.iter().collect(),
            is_highlighted,
        }
    }

    fn highlighted(chars: &[char]) -> Self {
        Self {
            text: chars
                .iter()
                .map(|it| match it {
                    ' ' => SPACE_MARKER.to_string(),
                    '\t' => TAB_MARKER.to_string(),
                    _ => it.to_string(),
                })
                .collect(),
            is_highlighted: true,
        }
    }

    fn highlight_marker(marker: &str) -> Self {
        Self {
            text: marker.to_string(),
            is_highlighted: true,
        }
    }
}

fn merge_ranges(
    ranges: Vec<crate::diff_review::TextRange>,
) -> Vec<crate::diff_review::TextRange> {
    let mut merged = Vec::<crate::diff_review::TextRange>::new();
    for range in ranges {
        if let Some(last) = merged.last_mut()
            && range.start <= last.end
        {
            last.end = last.end.max(range.end);
            continue;
        }
        merged.push(range);
    }
    merged
}

fn text_display_width(text: &str) -> f32 {
    const AVG_MONOSPACE_GLYPH_WIDTH: f32 = 7.5;
    (text.chars().count() as f32 * AVG_MONOSPACE_GLYPH_WIDTH) + TEXT_EXTRA_PADDING
}

fn segment_background(
    segment: &TextSegment,
    tone: DiffTone,
    palette: &DiffViewerPalette,
) -> egui::Color32 {
    if segment.is_highlighted {
        palette.highlight_background_for(tone)
    } else {
        palette.background_for(tone)
    }
}

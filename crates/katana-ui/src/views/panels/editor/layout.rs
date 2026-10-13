use crate::views::panels::editor::types::EditorColors;
use eframe::egui;
use egui_commonmark_backend::misc::CommonMarkCache;
use egui_commonmark_backend::syntect::easy::HighlightLines;
use egui_commonmark_backend::syntect::util::LinesWithEndings;

pub struct EditorLayouter<'a> {
    pub colors: EditorColors,
    pub cache: &'a mut CommonMarkCache,
    pub extension: Option<String>,
}

impl EditorLayouter<'_> {
    pub fn layout(
        &mut self,
        ui: &egui::Ui,
        text: &str,
        wrap_width: f32,
        font_id: &egui::FontId,
    ) -> std::sync::Arc<egui::Galley> {
        let mut job = egui::text::LayoutJob::default();
        job.wrap.max_width = wrap_width;

        let theme_name = egui_commonmark_backend::misc::default_theme(ui);

        /* WHY: We assume the themes are loaded in CommonMarkCache::default(). */
        let theme = self
            .cache
            .ts
            .themes
            .get(theme_name)
            .or_else(|| self.cache.ts.themes.values().next())
            .expect("No syntect themes available in CommonMarkCache");

        let syntax = self
            .extension
            .as_ref()
            .and_then(|ext| self.cache.ps.find_syntax_by_extension(ext))
            .or_else(|| self.cache.ps.find_syntax_by_extension("md"));

        if let Some(syntax) = syntax {
            let mut h = HighlightLines::new(syntax, theme);
            for line in LinesWithEndings::from(text) {
                self.append_highlighted_line(&mut job, &mut h, line, font_id);
            }
        } else {
            /* WHY: Fallback to basic text color if syntax is not found. */
            job.append(
                text,
                0.0,
                egui::TextFormat::simple(font_id.clone(), self.colors.1),
            );
        }

        ui.fonts_mut(|f| f.layout_job(job))
    }
}

impl EditorLayouter<'_> {
    fn append_highlighted_line(
        &self,
        job: &mut egui::text::LayoutJob,
        h: &mut HighlightLines,
        line: &str,
        font_id: &egui::FontId,
    ) {
        match h.highlight_line(line, &self.cache.ps) {
            Ok(ranges) => {
                for (style, text) in ranges {
                    let color =
                        egui_commonmark_backend::misc::syntect_color_to_egui(style.foreground);
                    job.append(text, 0.0, egui::TextFormat::simple(font_id.clone(), color));
                }
            }
            Err(_) => {
                job.append(
                    line,
                    0.0,
                    egui::TextFormat::simple(font_id.clone(), self.colors.1),
                );
            }
        }
    }
}

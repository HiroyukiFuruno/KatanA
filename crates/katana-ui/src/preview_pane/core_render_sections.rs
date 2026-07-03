use egui_commonmark::CommonMarkCache;
use katana_core::preview::{ImagePreviewOps, PreviewSection, PreviewSectionOps};

use super::types::*;

impl PreviewPane {
    pub fn update_html_document_sections(
        &mut self,
        source: &str,
        html_file_path: &std::path::Path,
    ) {
        self.md_file_path = html_file_path.to_path_buf();
        self.outline_items.clear();
        self.anchor_map.clear();
        self.document_anchors.clear();
        self.sections = Self::html_document_sections(source);
        self.section_lifecycle
            .resize(self.sections.len(), SectionLifecycle::default());
    }

    pub fn full_render_html_document(
        &mut self,
        source: &str,
        html_file_path: &std::path::Path,
        force: bool,
    ) {
        if force {
            self.commonmark_cache = CommonMarkCache::default();
            self.viewer_states.clear();
            self.fullscreen_viewer_state.reset();
            self.fullscreen_image = None;
        }

        self.md_file_path = html_file_path.to_path_buf();
        self.outline_items.clear();
        self.anchor_map.clear();
        self.document_anchors.clear();
        self.reset_html_document_render_state();
        self.session_generation += 1;
        self.sections = Self::html_document_sections(source);
        self.section_lifecycle = self
            .sections
            .iter()
            .map(|_| SectionLifecycle {
                is_loaded: true,
                is_drawn: false,
            })
            .collect();
    }

    fn reset_html_document_render_state(&mut self) {
        self.image_preload_queue.clear();
        self.image_cache.clear();
        self.render_rx = None;
        self.is_loading = false;
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.cancel_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    }

    fn html_document_sections(source: &str) -> Vec<RenderedSection> {
        if source.trim().is_empty() {
            return Vec::new();
        }
        let source_lines = source.lines().count().max(1);
        vec![RenderedSection::HtmlDocument(
            source.to_string(),
            source_lines,
        )]
    }

    pub fn update_markdown_sections(&mut self, source: &str, md_file_path: &std::path::Path) {
        self.md_file_path = md_file_path.to_path_buf();
        let (outline_items, document_anchors) =
            katana_core::markdown::outline::MarkdownOutlineOps::extract_outline(source);
        self.outline_items = outline_items;

        self.anchor_map = crate::preview_pane::types::DocumentAnchorMapItem::from_document_anchors(
            &document_anchors,
        );
        self.document_anchors = document_anchors;
        let (resolved, extracted_paths) =
            ImagePreviewOps::resolve_image_paths(source, md_file_path);

        for path in extracted_paths {
            if !self.image_cache.contains(&path) && !self.image_preload_queue.contains(&path) {
                self.image_preload_queue.push(path);
            }
        }

        let raw = PreviewSectionOps::split_into_sections(&resolved);
        let mut new_sections = Vec::with_capacity(raw.len());
        let mut diagram_iter = self.sections.iter().filter(|s| {
            !matches!(
                s,
                RenderedSection::Markdown(_, _) | RenderedSection::HtmlDocument(_, _)
            )
        });
        for section in &raw {
            self.push_markdown_section(section, &mut diagram_iter, &mut new_sections);
        }
        self.sections = new_sections;
        self.section_lifecycle
            .resize(self.sections.len(), SectionLifecycle::default());
    }

    fn push_markdown_section<'a>(
        &self,
        section: &PreviewSection,
        diagram_iter: &mut impl Iterator<Item = &'a RenderedSection>,
        new_sections: &mut Vec<RenderedSection>,
    ) {
        match section {
            PreviewSection::Markdown(md, lines) => {
                let processed_md =
                    katana_core::preview::types::MathPreviewOps::process_relaxed_math(md)
                        .to_string();
                new_sections.push(RenderedSection::Markdown(processed_md, *lines));
            }
            PreviewSection::Diagram {
                kind,
                source,
                lines,
            } => {
                let reused =
                    diagram_iter
                        .next()
                        .cloned()
                        .unwrap_or_else(|| RenderedSection::Error {
                            kind: format!("{kind:?}"),
                            _source: source.clone(),
                            message: format!(
                                "{} {}",
                                crate::icon::Icon::Refresh.as_char(),
                                "Please refresh the preview"
                            ),
                            source_lines: *lines,
                        });
                new_sections.push(reused);
            }
            PreviewSection::LocalImage { path, alt, lines } => {
                let path_buf = std::path::PathBuf::from(path.trim_start_matches("file://"));
                new_sections.push(RenderedSection::LocalImage {
                    path: path_buf,
                    alt: alt.clone(),
                    source_lines: *lines,
                });
            }
        }
    }
}

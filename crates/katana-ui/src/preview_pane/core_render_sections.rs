use katana_core::preview::{ImagePreviewOps, PreviewSection, PreviewSectionOps};

use super::types::*;

impl PreviewPane {
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
        let mut diagram_iter = self.sections.iter().filter(|s| !is_markdown_section(s));
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

fn is_markdown_section(section: &RenderedSection) -> bool {
    matches!(section, RenderedSection::Markdown(_, _))
}

use egui_commonmark::CommonMarkCache;
use katana_core::preview::{ImagePreviewOps, PreviewFlattenOps, PreviewSection, PreviewSectionOps};

use super::types::*;

impl PreviewPane {
    pub fn update_markdown_sections(&mut self, source: &str, md_file_path: &std::path::Path) {
        self.md_file_path = md_file_path.to_path_buf();
        self.outline_items =
            katana_core::markdown::outline::MarkdownOutlineOps::extract_outline(source);
        let (resolved, extracted_paths) =
            ImagePreviewOps::resolve_image_paths(source, md_file_path);

        for path in extracted_paths {
            if !self.image_cache.contains(&path) && !self.image_preload_queue.contains(&path) {
                self.image_preload_queue.push(path);
            }
        }

        let flattened = PreviewFlattenOps::flatten_list_code_blocks(&resolved);
        let raw = PreviewSectionOps::split_sections(&flattened);
        let mut new_sections = Vec::with_capacity(raw.len());
        let mut diagram_iter = self
            .sections
            .iter()
            .filter(|s| !matches!(s, RenderedSection::Markdown(_)));
        for section in &raw {
            match section {
                PreviewSection::Markdown(md) => {
                    new_sections.push(RenderedSection::Markdown(md.clone()));
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
                                message: "🔄 Please refresh the preview".to_string(),
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
        self.sections = new_sections;
        self.section_lifecycle
            .resize(self.sections.len(), SectionLifecycle::default());
    }

    pub fn abort_renders(&mut self) {
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.is_loading = false;
        self.render_rx = None;
    }

    pub fn full_render(
        &mut self,
        source: &str,
        md_file_path: &std::path::Path,
        cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
        force: bool,
        diagram_concurrency: usize,
    ) {
        if force {
            self.commonmark_cache = CommonMarkCache::default();
        }

        self.md_file_path = md_file_path.to_path_buf();
        self.outline_items =
            katana_core::markdown::outline::MarkdownOutlineOps::extract_outline(source);
        let (resolved, extracted_paths) =
            ImagePreviewOps::resolve_image_paths(source, md_file_path);

        self.image_preload_queue.clear();
        self.image_cache.clear();
        self.image_preload_queue = extracted_paths;

        let flattened = PreviewFlattenOps::flatten_list_code_blocks(&resolved);
        let raw = PreviewSectionOps::split_sections(&flattened);
        self.render_rx = None;

        let mut sections = Vec::with_capacity(raw.len());
        let mut lifecycle = Vec::with_capacity(raw.len());
        let mut jobs: Vec<RenderJob> = Vec::new();

        self.session_generation += 1;
        let current_generation = self.session_generation;

        for (ordinal, section) in raw.iter().enumerate() {
            match section {
                PreviewSection::Markdown(md) => {
                    sections.push(RenderedSection::Markdown(md.clone()));
                    lifecycle.push(SectionLifecycle {
                        is_loaded: true,
                        is_drawn: false,
                    });
                }
                PreviewSection::Diagram {
                    kind,
                    source,
                    lines,
                } => {
                    sections.push(RenderedSection::Pending {
                        kind: format!("{kind:?}"),
                        source: source.clone(),
                        source_lines: *lines,
                    });
                    jobs.push(RenderJob {
                        kind: kind.clone(),
                        src: source.clone(),
                        path: self.md_file_path.clone(),
                        cache: cache.clone(),
                        force,
                        source_lines: *lines,
                        generation: current_generation,
                        ordinal,
                    });
                    lifecycle.push(SectionLifecycle {
                        is_loaded: false,
                        is_drawn: false,
                    });
                }
                PreviewSection::LocalImage { path, alt, lines } => {
                    let path_buf = std::path::PathBuf::from(path.trim_start_matches("file://"));
                    sections.push(RenderedSection::LocalImage {
                        path: path_buf,
                        alt: alt.clone(),
                        source_lines: *lines,
                    });
                    lifecycle.push(SectionLifecycle {
                        is_loaded: true,
                        is_drawn: false,
                    });
                }
            }
        }
        self.sections = sections;
        self.section_lifecycle = lifecycle;

        if jobs.is_empty() {
            self.is_loading = false;
            return;
        }

        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let current_cancel_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.cancel_token = current_cancel_token.clone();

        self.is_loading = true;
        let (tx, rx) = std::sync::mpsc::channel();
        self.render_rx = Some(rx);

        let concurrency = diagram_concurrency.max(1);
        super::render_workers::spawn_render_workers(
            jobs,
            tx,
            current_cancel_token,
            self.repaint_ctx.clone(),
            concurrency,
        );
    }
}

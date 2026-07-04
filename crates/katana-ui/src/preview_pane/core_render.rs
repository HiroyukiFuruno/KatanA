use egui_commonmark::CommonMarkCache;
use katana_core::preview::{ImagePreviewOps, PreviewSection, PreviewSectionOps};

use super::types::*;

impl PreviewPane {
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
            self.viewer_states.clear();
            self.fullscreen_viewer_state.reset();
            self.fullscreen_image = None;
        }

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

        self.image_preload_queue.clear();
        self.image_cache.clear();
        self.image_preload_queue = extracted_paths;

        let raw = PreviewSectionOps::split_into_sections(&resolved);
        self.render_rx = None;

        let mut sections = Vec::with_capacity(raw.len());
        let mut lifecycle = Vec::with_capacity(raw.len());
        let mut jobs: Vec<RenderJob> = Vec::new();
        let mut active_diagrams =
            super::core_render_diagram::CoreRenderDiagramOps::active_diagrams(&resolved);

        self.session_generation += 1;
        let current_generation = self.session_generation;

        for (ordinal, section) in raw.iter().enumerate() {
            match section {
                PreviewSection::Markdown(md, lines) => {
                    let processed_md =
                        katana_core::preview::types::MathPreviewOps::process_relaxed_math(md)
                            .to_string();
                    sections.push(RenderedSection::Markdown(processed_md, *lines));
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
                    super::core_render_diagram::CoreRenderDiagramOps::handle_diagram_section(
                        &mut active_diagrams,
                        kind,
                        source,
                        *lines,
                        md_file_path,
                        &cache,
                        current_generation,
                        ordinal,
                        force,
                        &mut sections,
                        &mut jobs,
                        &mut lifecycle,
                    );
                }
                PreviewSection::LocalImage { path, alt, lines } => {
                    crate::preview_pane::types::PreviewPaneUtilsOps::handle_local_image_section(
                        path,
                        alt,
                        *lines,
                        &self.md_file_path,
                        cache.clone(),
                        force,
                        current_generation,
                        ordinal,
                        &mut active_diagrams,
                        &mut sections,
                        &mut jobs,
                        &mut lifecycle,
                    );
                }
            }
        }
        self.sections = sections;
        self.section_lifecycle = lifecycle;
        super::diagram_cache::DiagramRenderCacheCoordinator::prune_document(
            &cache,
            md_file_path,
            &active_diagrams,
        );

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

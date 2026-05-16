use super::types::{RenderJob, RenderedSection, RendererLogicOps, SectionLifecycle};
use katana_core::markdown::DiagramKind;
use std::path::Path;
use std::sync::Arc;

pub(super) struct CoreRenderDiagramOps;

impl CoreRenderDiagramOps {
    pub(super) fn active_diagrams(source: &str) -> Vec<(DiagramKind, String)> {
        katana_core::preview::DiagramAstBlockExtractor::extract(source)
            .into_iter()
            .map(|block| (block.kind, block.source))
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn handle_diagram_section(
        active_diagrams: &mut Vec<(DiagramKind, String)>,
        kind: &DiagramKind,
        source: &str,
        lines: usize,
        md_file_path: &Path,
        cache: &Arc<dyn katana_platform::CacheFacade>,
        current_generation: u64,
        ordinal: usize,
        force: bool,
        sections: &mut Vec<RenderedSection>,
        jobs: &mut Vec<RenderJob>,
        lifecycle: &mut Vec<SectionLifecycle>,
    ) {
        Self::push_active_diagram(active_diagrams, kind.clone(), source.to_string());
        if !force
            && Self::push_cached_section(
                cache,
                md_file_path,
                kind,
                source,
                lines,
                sections,
                lifecycle,
            )
        {
            return;
        }
        sections.push(RenderedSection::Pending {
            kind: format!("{kind:?}"),
            source: source.to_string(),
            source_lines: lines,
        });
        jobs.push(RenderJob {
            kind: kind.clone(),
            src: source.to_string(),
            cache: cache.clone(),
            document_path: md_file_path.to_path_buf(),
            source_lines: lines,
            generation: current_generation,
            ordinal,
            force,
        });
        lifecycle.push(SectionLifecycle {
            is_loaded: false,
            is_drawn: false,
        });
    }

    fn push_cached_section(
        cache: &Arc<dyn katana_platform::CacheFacade>,
        md_file_path: &Path,
        kind: &DiagramKind,
        source: &str,
        lines: usize,
        sections: &mut Vec<RenderedSection>,
        lifecycle: &mut Vec<SectionLifecycle>,
    ) -> bool {
        let Some(result) = super::diagram_cache::DiagramRenderCacheCoordinator::cached_result(
            cache,
            md_file_path,
            kind,
            source,
        ) else {
            return false;
        };
        sections.push(RendererLogicOps::map_diagram_result(
            kind, source, result, lines,
        ));
        lifecycle.push(SectionLifecycle {
            is_loaded: true,
            is_drawn: false,
        });
        true
    }

    fn push_active_diagram(
        active_diagrams: &mut Vec<(DiagramKind, String)>,
        kind: DiagramKind,
        source: String,
    ) {
        if active_diagrams
            .iter()
            .any(|(active_kind, active_source)| active_kind == &kind && active_source == &source)
        {
            return;
        }
        active_diagrams.push((kind, source));
    }
}

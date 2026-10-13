use super::content::{DiagramCacheIdentity, DiagramCacheIdentityService};
use super::metrics::{DiagramCacheMetric, DiagramCacheMetrics};
use super::svg_store::DiagramSvgCacheStore;
use katana_core::markdown::{DiagramKind, DiagramResult};
use std::path::Path;
use std::sync::Arc;

pub(crate) struct DiagramRenderCacheCoordinator;

impl DiagramRenderCacheCoordinator {
    pub(crate) fn cache_key(document_path: &Path, kind: &DiagramKind, source: &str) -> String {
        let identity = DiagramCacheIdentityService::build(document_path, kind, source);
        format!(
            "{}/{}/{}",
            identity.document_dir_name,
            identity.kind_dir_name,
            identity.cache_file_name()
        )
    }

    pub(crate) fn identity_for(
        document_path: &Path,
        kind: &DiagramKind,
        source: &str,
    ) -> DiagramCacheIdentity {
        DiagramCacheIdentityService::build(document_path, kind, source)
    }

    pub(crate) fn cached_svg(
        cache: &Arc<dyn katana_platform::CacheFacade>,
        document_path: &Path,
        kind: &DiagramKind,
        source: &str,
    ) -> Option<String> {
        let identity = Self::identity_for(document_path, kind, source);
        let Some(root) = cache.diagram_cache_root() else {
            DiagramCacheMetrics::emit(DiagramCacheMetric::Miss, kind, &identity.content_checksum);
            return None;
        };
        let path = identity.cache_path(&root);
        let Ok(svg) = std::fs::read_to_string(&path) else {
            DiagramCacheMetrics::emit(DiagramCacheMetric::Miss, kind, &identity.content_checksum);
            return None;
        };
        if !is_svg_payload(&svg) {
            DiagramCacheMetrics::emit(
                DiagramCacheMetric::CorruptSvg,
                kind,
                &identity.content_checksum,
            );
            let _ = std::fs::remove_file(path);
            return None;
        }
        DiagramCacheMetrics::emit(DiagramCacheMetric::Hit, kind, &identity.content_checksum);
        Some(svg)
    }

    pub(crate) fn cached_result(
        cache: &Arc<dyn katana_platform::CacheFacade>,
        document_path: &Path,
        kind: &DiagramKind,
        source: &str,
    ) -> Option<DiagramResult> {
        Self::cached_svg(cache, document_path, kind, source).map(DiagramResult::Ok)
    }

    pub(crate) fn store_result(
        cache: &Arc<dyn katana_platform::CacheFacade>,
        document_path: &Path,
        kind: &DiagramKind,
        source: &str,
        result: &DiagramResult,
    ) {
        let DiagramResult::Ok(html) = result else {
            return;
        };
        let Some(svg) = extract_svg_payload(html) else {
            return;
        };
        Self::store_svg(cache, document_path, kind, source, svg);
    }

    pub(crate) fn store_svg(
        cache: &Arc<dyn katana_platform::CacheFacade>,
        document_path: &Path,
        kind: &DiagramKind,
        source: &str,
        svg: &str,
    ) {
        let Some(root) = cache.diagram_cache_root() else {
            return;
        };
        let identity = Self::identity_for(document_path, kind, source);
        let _ = DiagramSvgCacheStore::write_svg(&root, &identity, svg);
    }

    pub(crate) fn prune_document(
        cache: &Arc<dyn katana_platform::CacheFacade>,
        document_path: &Path,
        active_diagrams: &[(DiagramKind, String)],
    ) {
        let Some(root) = cache.diagram_cache_root() else {
            return;
        };
        let document_dir_name = DiagramCacheIdentityService::document_dir_name(document_path);
        let identities = active_diagrams
            .iter()
            .map(|(kind, source)| Self::identity_for(document_path, kind, source))
            .collect::<Vec<_>>();
        let _ = DiagramSvgCacheStore::prune_document(&root, &document_dir_name, &identities);
    }

    pub(crate) fn record_redraw(kind: &DiagramKind, document_path: &Path, source: &str) {
        let identity = Self::identity_for(document_path, kind, source);
        DiagramCacheMetrics::emit(
            DiagramCacheMetric::RedrawExecuted,
            kind,
            &identity.content_checksum,
        );
    }

    pub(crate) fn record_checksum_skipped_by_tab_move() {
        DiagramCacheMetrics::emit_tab_switch_skipped();
    }
}

fn extract_svg_payload(html: &str) -> Option<&str> {
    let start = html.find("<svg")?;
    let end = html.rfind("</svg>")? + "</svg>".len();
    Some(&html[start..end])
}

fn is_svg_payload(svg: &str) -> bool {
    svg.contains("<svg") && svg.contains("</svg>")
}

use crate::preview_pane::PreviewPane;
use crate::shell::TabPreviewCache;

pub(super) fn preview_hash_for_path(
    previews: &[TabPreviewCache],
    path: &std::path::Path,
) -> Option<u64> {
    previews
        .iter()
        .find(|tab| tab.path == path)
        .map(|tab| tab.hash)
}

pub(super) fn update_preview_hash(
    previews: &mut [TabPreviewCache],
    path: &std::path::Path,
    hash: u64,
) {
    if let Some(tab) = previews.iter_mut().find(|tab| tab.path == path) {
        tab.hash = hash;
    }
}

pub(super) fn update_preview_pane(
    pane: &mut PreviewPane,
    path: &std::path::Path,
    source: &str,
    is_html: bool,
) {
    if is_html {
        pane.update_html_document_sections(source, path);
    } else {
        pane.update_markdown_sections(source, path);
    }
}

pub(super) struct FullPreviewRenderOptions {
    pub is_html: bool,
    pub force: bool,
    pub concurrency: usize,
    pub cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
}

pub(super) fn full_render_preview_pane(
    pane: &mut PreviewPane,
    path: &std::path::Path,
    source: &str,
    options: FullPreviewRenderOptions,
) {
    if options.is_html {
        pane.full_render_html_document(source, path, options.force);
    } else {
        pane.full_render(
            source,
            path,
            options.cache,
            options.force,
            options.concurrency,
        );
    }
}

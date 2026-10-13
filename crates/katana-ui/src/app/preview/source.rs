pub(super) fn is_html_preview_path(path: &std::path::Path) -> bool {
    katana_core::workspace::TreeEntry::path_is_html(path)
}

pub(super) fn markdown_preview_source_for_path(path: &std::path::Path, source: &str) -> String {
    if is_drawio_preview_path(path) {
        format!("```drawio\n{}\n```", source)
    } else {
        source.to_string()
    }
}

fn is_drawio_preview_path(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("drawio") || extension.eq_ignore_ascii_case("drowio")
        })
}

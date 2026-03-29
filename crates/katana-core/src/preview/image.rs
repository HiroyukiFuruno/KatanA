use regex::Regex;
use std::path::Path;

/// Resolves relative image paths in Markdown source to absolute `file://` URIs.
///
/// Given the path to the Markdown file being previewed, rewrites image references
/// like `![alt](../assets/image.png)` to `![alt](file:///absolute/path/assets/image.png)`.
///
/// Already-absolute paths, URLs (`http://`, `https://`), and `file://` URIs are left unchanged.
pub fn resolve_image_paths(source: &str, md_file_path: &Path) -> (String, Vec<std::path::PathBuf>) {
    use comrak::nodes::NodeValue;
    use comrak::{parse_document, Arena, Options};

    let arena = Arena::new();
    let root = parse_document(&arena, source, &Options::default());

    let mut line_offsets = vec![0];
    for (i, c) in source.char_indices() {
        if c == '\n' {
            line_offsets.push(i + 1);
        }
    }

    // WHY: Collect all Image AST nodes' source positions
    let mut replacements = Vec::new();
    for node in root.descendants() {
        if let NodeValue::Image(ref img) = node.data.borrow().value {
            let pos = node.data.borrow().sourcepos;
            let start_line_idx = pos.start.line.saturating_sub(1);
            let start_col_offset = pos.start.column.saturating_sub(1);
            let end_line_idx = pos.end.line.saturating_sub(1);
            let end_col_offset = pos.end.column.saturating_sub(1);

            let start_byte = line_offsets.get(start_line_idx).unwrap_or(&0) + start_col_offset;
            let end_byte = line_offsets.get(end_line_idx).unwrap_or(&0) + end_col_offset;

            if start_byte < source.len() && end_byte <= source.len() && start_byte <= end_byte {
                let node_str = &source[start_byte..end_byte];
                // WHY: We know node_str is something like `![alt](url)` or `![alt][ref]`
                // WHY: Let's find the URL portion. This is complex if `alt` texts contain parens.
                // WHY: However, we know `img.url` is exactly the URL string from the AST.
                let url_str = &img.url;
                if !url_str.is_empty() {
                    // WHY: Find the exact occurrence of `url_str` inside `node_str` searching from the end
                    if let Some(url_idx) = node_str.rfind(url_str.as_str()) {
                        replacements.push((
                            start_byte + url_idx,
                            start_byte + url_idx + url_str.len(),
                            url_str.to_string(),
                        ));
                    }
                }
            }
        }
    }

    // WHY: Sort replacements by start byte in reverse order to safely perform string replacements from the end
    replacements.sort_by_key(|&(start, _, _)| std::cmp::Reverse(start));

    let base_dir = md_file_path.parent().unwrap_or(Path::new("."));
    let mut result = source.to_string();

    let mut extracted_paths = Vec::new();

    for (start, end, raw_path) in replacements {
        if raw_path.starts_with("http://")
            || raw_path.starts_with("https://")
            || raw_path.starts_with("file://")
            || raw_path.starts_with("data:")
            || raw_path.starts_with('/')
        {
            continue;
        }

        let resolved = base_dir.join(&raw_path);
        let canonical = resolved.canonicalize().unwrap_or(resolved);
        let absolute_url = format!("file://{}", canonical.display());

        extracted_paths.push(canonical);

        result.replace_range(start..end, &absolute_url);
    }

    (result, extracted_paths)
}

/// Resolves relative `src` attributes in HTML `<img>` tags to absolute `file://` URIs.
///
/// This is the HTML counterpart of [`resolve_image_paths`], handling raw HTML
/// image tags within `HtmlBlock` sections.
pub fn resolve_html_image_paths(html: &str, md_file_path: &Path) -> String {
    /// Regex capture group index for the `<img ... src="` prefix.
    const CAP_PREFIX: usize = 1;
    /// Regex capture group index for the `src` attribute value.
    const CAP_SRC: usize = 2;
    /// Regex capture group index for the `" ...>` suffix.
    const CAP_SUFFIX: usize = 3;

    let base_dir = md_file_path.parent().unwrap_or(Path::new("."));
    let re = Regex::new(r#"(<img\s[^>]*src\s*=\s*")([^"]+)("[^>]*>)"#).unwrap();
    re.replace_all(html, |caps: &regex::Captures| {
        let prefix = caps.get(CAP_PREFIX).unwrap().as_str();
        let src = caps.get(CAP_SRC).unwrap().as_str();
        let suffix = caps.get(CAP_SUFFIX).unwrap().as_str();
        if src.starts_with("http://")
            || src.starts_with("https://")
            || src.starts_with("file://")
            || src.starts_with("data:")
            || src.starts_with('/')
        {
            format!("{prefix}{src}{suffix}")
        } else {
            let resolved = base_dir.join(src);
            let canonical = resolved.canonicalize().unwrap_or(resolved);
            format!("{prefix}file://{}{suffix}", canonical.display())
        }
    })
    .into_owned()
}

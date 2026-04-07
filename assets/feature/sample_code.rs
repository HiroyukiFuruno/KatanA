// sample_code.rs — KatanA Reference Code
//
// This file is opened in reference mode (read-only) by the Help → Demo action.
// It exists to demonstrate syntax highlighting in KatanA's code pane.
//
// The code below is a simplified walkthrough of how KatanA opens a document
// and routes it to the preview pane.

use std::path::PathBuf;

/// Access policy for a document opened in KatanA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentAccess {
    /// Normal editable document. Supports save, update, and replace operations.
    Editable,
    /// Reference document (read-only). All mutation paths are blocked.
    Reference,
}

/// A single document loaded into a KatanA tab.
#[derive(Debug)]
pub struct Document {
    pub path: PathBuf,
    pub buffer: String,
    pub is_dirty: bool,
    pub is_pinned: bool,
    pub access: DocumentAccess,
}

impl Document {
    /// Create a new editable document with the given content.
    pub fn new(path: PathBuf, content: String) -> Self {
        Self {
            path,
            buffer: content,
            is_dirty: false,
            is_pinned: false,
            access: DocumentAccess::Editable,
        }
    }

    /// Create a reference (read-only) document.
    pub fn new_reference(path: PathBuf, content: String) -> Self {
        Self {
            access: DocumentAccess::Reference,
            ..Self::new(path, content)
        }
    }

    /// Returns `true` if this document may be mutated.
    pub fn is_editable(&self) -> bool {
        self.access == DocumentAccess::Editable
    }

    /// Update the buffer. No-op for reference documents.
    pub fn update_buffer(&mut self, new_content: String) {
        if !self.is_editable() {
            return; // reference documents are immutable
        }
        self.buffer = new_content;
        self.is_dirty = true;
    }
}

/// Resolve the demo bundle from `assets/feature` under the given workspace root.
///
/// Resolution rules:
/// - Markdown files: prefer `<name>.ja.md` when `lang == "ja"`, fall back to `<name>.md`.
/// - Non-Markdown text files: opened as-is (reference mode).
/// - Binary/unreadable files: skipped.
pub fn resolve_demo_bundle(workspace_root: &std::path::Path, lang: &str) -> Vec<PathBuf> {
    let feature_dir = workspace_root.join("assets").join("feature");
    if !feature_dir.is_dir() {
        return Vec::new();
    }

    let mut markdown_files: Vec<PathBuf> = Vec::new();
    let mut reference_files: Vec<PathBuf> = Vec::new();

    let entries = std::fs::read_dir(&feature_dir)
        .into_iter()
        .flatten()
        .flatten();

    for entry in entries {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if ext == "md" {
            // Skip `.ja.md` files; we handle them via locale resolution.
            if name.ends_with(".ja.md") {
                continue;
            }
            // Locale resolution: prefer the Japanese variant if it exists.
            let resolved = if lang == "ja" {
                let stem = name.strip_suffix(".md").unwrap_or(name);
                let ja_variant = feature_dir.join(format!("{stem}.ja.md"));
                if ja_variant.exists() { ja_variant } else { path }
            } else {
                path
            };
            markdown_files.push(resolved);
        } else if std::fs::read_to_string(&path).is_ok() {
            // Any readable non-Markdown text file → reference mode.
            reference_files.push(path);
        }
        // Binary/unreadable files are silently skipped.
    }

    markdown_files.sort();
    reference_files.sort();
    markdown_files.extend(reference_files);
    markdown_files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_document_reference_immutable() {
        let mut doc = Document::new_reference(
            PathBuf::from("/demo/sample_code.rs"),
            "fn main() {}".to_string(),
        );
        doc.update_buffer("hacked".to_string());
        assert_eq!(doc.buffer, "fn main() {}");
        assert!(!doc.is_dirty);
    }

    #[test]
    fn test_resolve_demo_bundle_ja_fallback() {
        let tmp = TempDir::new().unwrap();
        let feature = tmp.path().join("assets/feature");
        fs::create_dir_all(&feature).unwrap();
        fs::write(feature.join("welcome.md"), "en").unwrap();
        // No welcome.ja.md — should fall back to welcome.md
        let result = resolve_demo_bundle(tmp.path(), "ja");
        assert!(result.iter().any(|p| p.ends_with("welcome.md")));
    }
}

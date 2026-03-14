use std::path::PathBuf;
use thiserror::Error;

/// Represents a single document in a workspace.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// Absolute path to the source file on disk.
    pub path: PathBuf,
    /// In-memory buffer content (may differ from disk if dirty).
    pub buffer: String,
    /// Whether the buffer has unsaved changes.
    pub is_dirty: bool,
}

impl Document {
    /// Create a new document with `content` loaded from `path`.
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            buffer: content.into(),
            is_dirty: false,
        }
    }

    /// Update the in-memory buffer content. Marks the document as dirty.
    pub fn update_buffer(&mut self, content: impl Into<String>) {
        let new = content.into();
        if self.buffer != new {
            self.buffer = new;
            self.is_dirty = true;
        }
    }

    /// Mark the document as clean (called after a successful save).
    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
    }

    /// Returns the file name of this document, if available.
    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name()?.to_str()
    }
}

/// Errors related to document operations.
#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("Failed to read document at {path}: {source}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to save document to {path}: {source}")]
    SaveFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl DocumentError {
    pub fn read_failed(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::ReadFailed {
            path: path.into(),
            source,
        }
    }

    pub fn save_failed(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::SaveFailed {
            path: path.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_document_is_clean() {
        let doc = Document::new("/tmp/test.md", "# Hello");
        assert!(!doc.is_dirty);
        assert_eq!(doc.buffer, "# Hello");
    }

    #[test]
    fn update_buffer_marks_dirty() {
        let mut doc = Document::new("/tmp/test.md", "# Hello");
        doc.update_buffer("# Hello World");
        assert!(doc.is_dirty);
        assert_eq!(doc.buffer, "# Hello World");
    }

    #[test]
    fn mark_clean_clears_dirty_flag() {
        let mut doc = Document::new("/tmp/test.md", "# Hello");
        doc.update_buffer("# Changed");
        doc.mark_clean();
        assert!(!doc.is_dirty);
    }

    #[test]
    fn update_with_same_content_does_not_dirty() {
        let mut doc = Document::new("/tmp/test.md", "# Hello");
        doc.update_buffer("# Hello");
        assert!(!doc.is_dirty);
    }

    #[test]
    fn file_name_returns_basename() {
        let doc = Document::new("/workspace/spec.md", "");
        assert_eq!(doc.file_name(), Some("spec.md"));
    }
}

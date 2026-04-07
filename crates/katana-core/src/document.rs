use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub path: PathBuf,
    pub buffer: String,
    pub is_dirty: bool,
    pub is_loaded: bool,
    pub is_pinned: bool,
    pub is_reference: bool,
    pub last_imported_disk_hash: Option<u64>,
    pub pending_dirty_warning_hash: Option<u64>,
}

const FNV1A_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV1A_PRIME: u64 = 0x100000001b3;

pub struct DocumentOps;

impl DocumentOps {
    pub fn compute_hash(s: &str) -> u64 {
        let mut h: u64 = FNV1A_OFFSET_BASIS;
        for b in s.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(FNV1A_PRIME);
        }
        h
    }
}

impl Document {
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        let content = content.into();
        let hash = DocumentOps::compute_hash(&content);
        Self {
            path: path.into(),
            buffer: content,
            is_dirty: false,
            is_loaded: true,
            is_pinned: false,
            is_reference: false,
            last_imported_disk_hash: Some(hash),
            pending_dirty_warning_hash: None,
        }
    }

    pub fn new_empty(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            buffer: String::new(),
            is_dirty: false,
            is_loaded: false,
            is_pinned: false,
            is_reference: false,
            last_imported_disk_hash: None,
            pending_dirty_warning_hash: None,
        }
    }

    pub fn update_buffer(&mut self, content: impl Into<String>) {
        let new = content.into();
        if self.buffer != new {
            self.buffer = new;
            self.is_dirty = true;
        }
    }

    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
        self.last_imported_disk_hash = Some(DocumentOps::compute_hash(&self.buffer));
        self.pending_dirty_warning_hash = None;
    }

    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name()?.to_str()
    }
}

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
    fn test_document_new_empty() {
        let path = PathBuf::from("test.md");
        let doc = Document::new_empty(path.clone());
        assert_eq!(doc.path, path);
        assert!(doc.buffer.is_empty());
        assert!(!doc.is_loaded);
        assert_eq!(doc.last_imported_disk_hash, None);
        assert_eq!(doc.pending_dirty_warning_hash, None);
    }

    #[test]
    fn test_document_new_with_content() {
        let path = PathBuf::from("test.md");
        let doc = Document::new(path.clone(), "hello");
        assert_eq!(doc.path, path);
        assert_eq!(doc.buffer, "hello");
        assert!(!doc.is_dirty);
        assert!(doc.is_loaded);
        assert!(doc.last_imported_disk_hash.is_some());
        assert_eq!(doc.pending_dirty_warning_hash, None);
    }

    #[test]
    fn test_document_mark_clean_updates_hash() {
        let mut doc = Document::new("test.md", "hello");
        doc.update_buffer("world");
        assert!(doc.is_dirty);
        doc.mark_clean();
        assert!(!doc.is_dirty);
        assert_eq!(
            doc.last_imported_disk_hash,
            Some(DocumentOps::compute_hash("world"))
        );
        assert_eq!(doc.pending_dirty_warning_hash, None);
    }
}

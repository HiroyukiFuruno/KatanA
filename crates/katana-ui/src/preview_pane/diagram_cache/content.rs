use katana_core::markdown::{
    DiagramBackendFactory, DiagramBackendLanguage, DiagramKind, DiagramThemeSnapshot,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DiagramCacheIdentity {
    pub(crate) document_dir_name: String,
    pub(crate) kind_dir_name: String,
    pub(crate) content_checksum: String,
    pub(crate) renderer_version: String,
    pub(crate) theme_hash: String,
}

impl DiagramCacheIdentity {
    pub(crate) fn cache_file_name(&self) -> String {
        format!(
            "{}_{}_{}.svg",
            self.content_checksum, self.renderer_version, self.theme_hash
        )
    }

    pub(crate) fn cache_path(&self, root: &Path) -> PathBuf {
        root.join(&self.document_dir_name)
            .join(&self.kind_dir_name)
            .join(self.cache_file_name())
    }

    pub(crate) fn active_token(&self) -> DiagramActiveToken {
        DiagramActiveToken {
            kind_dir_name: self.kind_dir_name.clone(),
            content_checksum: self.content_checksum.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DiagramActiveToken {
    pub(crate) kind_dir_name: String,
    pub(crate) content_checksum: String,
}

pub(crate) struct DiagramCacheIdentityService;

impl DiagramCacheIdentityService {
    pub(crate) fn build(
        document_path: &Path,
        kind: &DiagramKind,
        source: &str,
    ) -> DiagramCacheIdentity {
        let backend_language = Self::backend_language(kind);
        let backend = DiagramBackendFactory::create(backend_language);
        let theme = DiagramThemeSnapshot::current();
        DiagramCacheIdentity {
            document_dir_name: Self::document_dir_name(document_path),
            kind_dir_name: kind_dir_name(kind).to_string(),
            content_checksum: DiagramChecksumService::checksum(kind, source),
            renderer_version: sanitize_filename(&backend.version().value),
            theme_hash: format!("{:x}", deterministic_hash(&theme.fingerprint())),
        }
    }

    pub(crate) fn document_dir_name(document_path: &Path) -> String {
        format!(
            "doc_{:x}",
            deterministic_hash(&absolute_path(document_path))
        )
    }

    fn backend_language(kind: &DiagramKind) -> DiagramBackendLanguage {
        match kind {
            DiagramKind::Mermaid => DiagramBackendLanguage::Mermaid,
            DiagramKind::PlantUml => DiagramBackendLanguage::PlantUml,
            DiagramKind::DrawIo => DiagramBackendLanguage::DrawIo,
        }
    }
}

pub(crate) struct DiagramContentCanonicalizer;

impl DiagramContentCanonicalizer {
    pub(crate) fn canonicalize(kind: &DiagramKind, source: &str) -> String {
        let normalized_source = source.replace("\r\n", "\n").replace('\r', "\n");
        format!("kind={}\nsource={normalized_source}", kind_dir_name(kind))
    }
}

pub(crate) struct DiagramChecksumService;

impl DiagramChecksumService {
    pub(crate) fn checksum(kind: &DiagramKind, source: &str) -> String {
        let canonical = DiagramContentCanonicalizer::canonicalize(kind, source);
        format!("{:x}", deterministic_hash(&canonical))
    }
}

fn absolute_path(path: &Path) -> String {
    if path.is_absolute() {
        return path.to_string_lossy().to_string();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}

fn kind_dir_name(kind: &DiagramKind) -> &'static str {
    match kind {
        DiagramKind::Mermaid => "mermaid",
        DiagramKind::PlantUml => "plantuml",
        DiagramKind::DrawIo => "drawio",
    }
}

fn sanitize_filename(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn deterministic_hash(data: &str) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in data.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use katana_core::markdown::DiagramKind;

    #[test]
    fn identity_ignores_line_ending_platform_difference() {
        let path = Path::new("/tmp/doc.md");
        let lf = DiagramCacheIdentityService::build(path, &DiagramKind::Mermaid, "graph TD\nA-->B");
        let crlf =
            DiagramCacheIdentityService::build(path, &DiagramKind::Mermaid, "graph TD\r\nA-->B");

        assert_eq!(lf.content_checksum, crlf.content_checksum);
    }

    #[test]
    fn identity_changes_when_diagram_content_changes() {
        let path = Path::new("/tmp/doc.md");
        let first =
            DiagramCacheIdentityService::build(path, &DiagramKind::Mermaid, "graph TD\nA-->B");
        let second =
            DiagramCacheIdentityService::build(path, &DiagramKind::Mermaid, "graph TD\nA-->C");

        assert_ne!(first.content_checksum, second.content_checksum);
    }
}

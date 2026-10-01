use std::path::PathBuf;
use std::sync::Arc;

pub(super) const MAX_FONT_PAYLOAD_BYTES: u64 = 32 * 1024 * 1024;
pub(super) const FONT_READ_CHUNK_BYTES: usize = 64 * 1024;
pub(super) const REGULAR_WEIGHT: u16 = 400;
pub(super) const BOLD_WEIGHT: u16 = 700;
pub(super) const MAX_FONT_WEIGHT: f32 = 1000.0;
pub(super) const BOLD_WEIGHT_THRESHOLD: u16 = 700;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct FontFaceRequest {
    pub(crate) family: String,
    pub(crate) bold: bool,
    pub(crate) italic: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FontFaceResolution {
    pub(crate) faces: Vec<ResolvedFontFace>,
    pub(crate) diagnostics: Vec<FontFaceResolutionDiagnostic>,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedFontFace {
    pub(crate) request: FontFaceRequest,
    pub(crate) family: String,
    pub(crate) weight: u16,
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) monospaced: bool,
    pub(crate) path: PathBuf,
    pub(crate) face_index: u32,
    pub(crate) payload: Arc<egui::FontData>,
}

#[derive(Clone, Debug)]
pub(crate) enum FontFaceResolutionDiagnostic {
    FileTooLarge {
        path: PathBuf,
        bytes: u64,
        limit: u64,
    },
    FileUnreadable {
        path: PathBuf,
        kind: std::io::ErrorKind,
    },
    NonRegularFile {
        path: PathBuf,
    },
    Cancelled,
    InvalidFontFile {
        path: PathBuf,
    },
    InvalidFontFace {
        path: PathBuf,
        face_index: usize,
    },
    RequestedFaceUnavailable {
        request: FontFaceRequest,
        search_incomplete: bool,
    },
}

impl FontFaceResolutionDiagnostic {
    pub(crate) fn description(&self) -> String {
        match self {
            Self::FileTooLarge { path, bytes, limit } => format!(
                "font file {} is {bytes} bytes; limit is {limit}",
                path.display()
            ),
            Self::FileUnreadable { path, kind } => {
                format!("font file {} could not be read: {kind:?}", path.display())
            }
            Self::NonRegularFile { path } => {
                format!("font candidate {} is not a regular file", path.display())
            }
            Self::Cancelled => "font resolution was cancelled".to_owned(),
            Self::InvalidFontFile { path } => {
                format!("font file {} has invalid font data", path.display())
            }
            Self::InvalidFontFace { path, face_index } => format!(
                "font file {} has invalid face index {face_index}",
                path.display()
            ),
            Self::RequestedFaceUnavailable {
                request,
                search_incomplete,
            } => format!(
                "font face unavailable: family={:?} bold={} italic={} search_incomplete={search_incomplete}",
                request.family, request.bold, request.italic
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{FontFaceRequest, FontFaceResolutionDiagnostic};

    #[test]
    fn descriptions_include_each_diagnostic_cause_and_context() {
        let messages = diagnostic_messages();
        let expected = diagnostic_contexts();
        assert_eq!(messages.len(), expected.len());
        for (message, expected) in messages.iter().zip(expected) {
            assert!(message.contains(expected), "{message}");
        }
    }

    fn diagnostic_messages() -> [String; 7] {
        let path = PathBuf::from("font.ttf");
        [
            FontFaceResolutionDiagnostic::FileTooLarge {
                path: path.clone(),
                bytes: 9,
                limit: 8,
            },
            FontFaceResolutionDiagnostic::FileUnreadable {
                path: path.clone(),
                kind: std::io::ErrorKind::PermissionDenied,
            },
            FontFaceResolutionDiagnostic::NonRegularFile { path: path.clone() },
            FontFaceResolutionDiagnostic::Cancelled,
            FontFaceResolutionDiagnostic::InvalidFontFile { path: path.clone() },
            FontFaceResolutionDiagnostic::InvalidFontFace {
                path: path.clone(),
                face_index: 2,
            },
            FontFaceResolutionDiagnostic::RequestedFaceUnavailable {
                request: FontFaceRequest {
                    family: "Example".into(),
                    bold: true,
                    italic: false,
                },
                search_incomplete: true,
            },
        ]
        .map(|diagnostic| diagnostic.description())
    }

    fn diagnostic_contexts() -> [&'static str; 7] {
        [
            "9 bytes; limit is 8",
            "PermissionDenied",
            "not a regular file",
            "cancelled",
            "invalid font data",
            "face index 2",
            "family=\"Example\" bold=true italic=false search_incomplete=true",
        ]
    }
}

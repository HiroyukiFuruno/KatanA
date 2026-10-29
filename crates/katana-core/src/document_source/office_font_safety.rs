use super::BinaryDocumentFormat;
use thiserror::Error;

mod archive;
mod attributes;
mod names;
mod scan;
mod xml;

/// 埋込みフォントをnative converterへ渡す前の出力名検査。
pub struct OfficeFontSafetyPolicy;

/// 埋込みフォントの出力前に拒否した入力と検査失敗。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OfficeFontSafetyError {
    #[error("Office font package is invalid: {0}")]
    InvalidArchive(String),
    #[error("Office font metadata `{part}` is invalid: {reason}")]
    InvalidMetadata { part: &'static str, reason: String },
    #[error("Office font input `{part}` exceeds the existing {limit}-byte resource limit")]
    ResourceLimit { part: &'static str, limit: u64 },
    #[error("Office embedded font in `{part}` has an unsafe output name")]
    UnsafeFontName { part: &'static str },
}

struct FontParts {
    format: BinaryDocumentFormat,
    metadata: &'static str,
}

impl FontParts {
    fn new(format: BinaryDocumentFormat) -> Option<Self> {
        let metadata = match format {
            BinaryDocumentFormat::Docx => "word/fontTable.xml",
            BinaryDocumentFormat::Pptx => "ppt/presentation.xml",
            BinaryDocumentFormat::Pdf | BinaryDocumentFormat::Xlsx => return None,
        };
        Some(Self { format, metadata })
    }

    fn contains(&self, name: &[u8]) -> bool {
        name == self.metadata.as_bytes()
    }
}

impl OfficeFontSafetyPolicy {
    /// PDF/XLSXは解析せず、DOCX/PPTXの埋込みフォント出力名を検査する。
    ///
    /// # Errors
    /// ZIP/XMLの不正、既存KDV上限超過、危険な埋込みフォント名を返す。
    pub fn validate(
        format: BinaryDocumentFormat,
        bytes: &[u8],
    ) -> Result<(), OfficeFontSafetyError> {
        let Some(parts) = FontParts::new(format) else {
            return Ok(());
        };
        archive::FontArchive::validate(bytes, &parts)
    }
}

impl OfficeFontSafetyError {
    fn archive(error: impl std::fmt::Display) -> Self {
        Self::InvalidArchive(error.to_string())
    }

    fn metadata(part: &'static str, error: impl std::fmt::Display) -> Self {
        Self::InvalidMetadata {
            part,
            reason: error.to_string(),
        }
    }

    fn ensure_size(part: &'static str, size: u64, limit: u64) -> Result<(), Self> {
        if size > limit {
            Err(Self::ResourceLimit { part, limit })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "office_font_safety_tests.rs"]
mod tests;

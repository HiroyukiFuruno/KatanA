//! JSON Canvas 1.0 の読み取り専用モデル。

mod boundary;
mod decode;
mod error;
mod model;
mod style;
mod validation;

pub use error::{CanvasError, GeometryField};
pub use model::{CanvasDiagnostic, CanvasDocument, CanvasLocation, CanvasNode, NodeContent};
pub use model::{CanvasEdge, CanvasGeometry};
pub use style::{BackgroundStyle, CanvasColor, EdgeEnd, EdgeSide};

impl CanvasDocument {
    /// JSON を検証して読み込む。ファイル参照や URL へのアクセスは行わない。
    ///
    /// # Errors
    /// JSON・必須属性・座標・ID・接続先が不正な場合は型付きエラーを返す。
    pub fn parse(source: &str) -> Result<Self, CanvasError> {
        let raw = serde_json::from_str(source).map_err(CanvasError::from_json)?;
        let document = decode::CanvasDecoder::decode_document(raw)?;
        validation::CanvasValidator::validate_references(&document)?;
        Ok(document)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod validation_tests;

//! JSON Canvas 1.0 の読み取り専用モデル。

mod boundary;
mod cancellable_deserializer;
mod cancellable_visitor;
mod cancellation;
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
    /// 値の読込境界で取消を確認し、取消時は部分モデルを返さない。
    ///
    /// # Errors
    /// 取消より先に確認した不正な入力は既存の型付きエラーを返す。
    pub fn parse_cancellable(
        source: &str,
        cancelled: impl Fn() -> bool,
    ) -> Result<Option<Self>, CanvasError> {
        cancellation::CanvasParser::parse(source, cancelled)
    }

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

#[cfg(test)]
mod cancellation_tests;

#[cfg(test)]
mod cancellable_contract_tests;

#[cfg(test)]
mod diagnostic_contract_tests;

#[cfg(test)]
mod cancellable_map_contract_tests;

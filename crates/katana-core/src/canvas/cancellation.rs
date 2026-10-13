use std::cell::Cell;

use serde::Deserialize;

use super::boundary::RawDocument;
use super::cancellable_deserializer::CheckedDeserializer;
use super::{CanvasDocument, CanvasError, decode, validation};

pub(super) struct Cancellation<F> {
    callback: F,
    observed: Cell<bool>,
}

impl<F: Fn() -> bool> Cancellation<F> {
    pub(super) fn new(callback: F) -> Self {
        Self {
            callback,
            observed: Cell::new(false),
        }
    }

    pub(super) fn check(&self) -> Result<(), ParseFailure> {
        if self.observed.get() || (self.callback)() {
            self.observed.set(true);
            return Err(ParseFailure::Cancelled);
        }
        Ok(())
    }

    pub(super) fn check_json<E: serde::de::Error>(&self) -> Result<(), E> {
        self.check()
            .map_err(|_| E::custom("Canvas parsing was cancelled"))
    }
}

pub(super) enum ParseFailure {
    Cancelled,
    Invalid(CanvasError),
}

impl From<CanvasError> for ParseFailure {
    fn from(error: CanvasError) -> Self {
        Self::Invalid(error)
    }
}

pub(super) struct CanvasParser;

impl CanvasParser {
    pub(super) fn parse(
        source: &str,
        cancelled: impl Fn() -> bool,
    ) -> Result<Option<CanvasDocument>, CanvasError> {
        let cancellation = Cancellation::new(cancelled);
        let result = Self::parse_checked(source, &cancellation);
        /* WHY: serde の取消通知を入力不正として公開せず、部分モデルも破棄する。 */
        if cancellation.observed.get() {
            return Ok(None);
        }
        match result {
            Ok(document) => Ok(Some(document)),
            Err(ParseFailure::Cancelled) => Ok(None),
            Err(ParseFailure::Invalid(error)) => Err(error),
        }
    }

    fn parse_checked<F: Fn() -> bool>(
        source: &str,
        cancellation: &Cancellation<F>,
    ) -> Result<CanvasDocument, ParseFailure> {
        cancellation.check()?;
        let mut parser = serde_json::Deserializer::from_str(source);
        let raw = RawDocument::deserialize(CheckedDeserializer::new(&mut parser, cancellation))
            .map_err(CanvasError::from_json)?;
        cancellation.check()?;
        parser.end().map_err(CanvasError::from_json)?;
        cancellation.check()?;
        let document = decode::CanvasDecoder::decode_document_with(raw, || cancellation.check())?;
        validation::CanvasValidator::validate_references_with(&document, || cancellation.check())?;
        cancellation.check()?;
        Ok(document)
    }
}

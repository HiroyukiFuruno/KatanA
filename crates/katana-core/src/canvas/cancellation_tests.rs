use super::{CanvasDocument, CanvasError};
use std::cell::Cell;

const CANCEL_CHECK: usize = 16;
const UNKNOWN_ELEMENTS: usize = 256;

#[test]
fn cancellation_inside_unknown_array_precedes_invalid_tail() {
    let values = std::iter::repeat_n("0", UNKNOWN_ELEMENTS)
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(r#"{{"extension":[{values}],"nodes":INVALID}}"#);
    let checks = Cell::new(0);
    let result = CanvasDocument::parse_cancellable(&source, || {
        checks.set(checks.get() + 1);
        checks.get() == CANCEL_CHECK
    });
    assert_eq!(result, Ok(None));
    assert_eq!(checks.get(), CANCEL_CHECK);
}

#[test]
fn cancellation_before_parsing_rejects_even_invalid_input() {
    assert_eq!(
        CanvasDocument::parse_cancellable("INVALID", || true),
        Ok(None)
    );
}

#[test]
fn uncancelled_input_preserves_original_models_and_errors() {
    for source in [
        include_str!("../../../../assets/fixtures/katana/read-only-preview.canvas"),
        r#"{"nodes":[],"nodes":[]}"#,
        r#"{"nodes":[],"edges":[]} garbage"#,
        r#"{"nodes":[{"id":"bad","type":"text","x":0.5}]}"#,
    ] {
        assert_eq!(
            CanvasDocument::parse_cancellable(source, || false),
            CanvasDocument::parse(source).map(Some)
        );
    }
}

#[test]
fn invalid_input_without_cancellation_is_not_a_cancelled_outcome() {
    assert!(matches!(
        CanvasDocument::parse_cancellable("INVALID", || false),
        Err(CanvasError::InvalidJson { .. })
    ));
}

use std::collections::BTreeMap;

use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;

use super::{CanvasDiagnostic, CanvasError, CanvasGeometry, CanvasLocation, GeometryField};

pub(super) type BoundaryFields = BTreeMap<String, Value>;

const MAX_EXACT_JSON_INTEGER: f64 = 9_007_199_254_740_991.0;

#[derive(Deserialize)]
pub(super) struct RawDocument {
    #[serde(default)]
    pub nodes: Vec<BoundaryFields>,
    #[serde(default)]
    pub edges: Vec<BoundaryFields>,
    #[serde(flatten)]
    pub extra: BoundaryFields,
}

pub(super) struct BoundaryDecoder;

impl BoundaryDecoder {
    pub(super) fn required<T: DeserializeOwned>(
        fields: &mut BoundaryFields,
        field: &'static str,
        location: CanvasLocation,
    ) -> Result<T, CanvasError> {
        let value = fields
            .remove(field)
            .ok_or(CanvasError::InvalidField { location, field })?;
        serde_json::from_value(value).map_err(|_| CanvasError::InvalidField { location, field })
    }

    pub(super) fn optional<T: DeserializeOwned>(
        fields: &mut BoundaryFields,
        field: &'static str,
        location: CanvasLocation,
    ) -> Result<Option<T>, CanvasError> {
        fields
            .remove(field)
            .map(|value| {
                serde_json::from_value(value)
                    .map_err(|_| CanvasError::InvalidField { location, field })
            })
            .transpose()
    }

    pub(super) fn geometry(
        fields: &mut BoundaryFields,
        index: usize,
    ) -> Result<CanvasGeometry, CanvasError> {
        Ok(CanvasGeometry {
            x: Self::coordinate(fields, "x", GeometryField::X, index, false)?,
            y: Self::coordinate(fields, "y", GeometryField::Y, index, false)?,
            width: Self::coordinate(fields, "width", GeometryField::Width, index, true)?,
            height: Self::coordinate(fields, "height", GeometryField::Height, index, true)?,
        })
    }

    fn coordinate(
        fields: &mut BoundaryFields,
        name: &'static str,
        field: GeometryField,
        index: usize,
        positive: bool,
    ) -> Result<f64, CanvasError> {
        let value = fields
            .remove(name)
            .and_then(|value| {
                let number = value.as_number()?;
                (number.is_i64() || number.is_u64())
                    .then(|| number.as_f64())
                    .flatten()
            })
            .ok_or(CanvasError::InvalidGeometry { index, field })?;
        /* WHY: 整数の精度と有限な描画座標を保証し、極端な値で座標演算を壊さない。 */
        if !value.is_finite()
            || value.abs() > MAX_EXACT_JSON_INTEGER
            || value.fract() != 0.0
            || (positive && value <= 0.0)
        {
            return Err(CanvasError::InvalidGeometry { index, field });
        }
        Ok(value)
    }

    pub(super) fn extra_fields(
        fields: BoundaryFields,
        location: CanvasLocation,
        diagnostics: &mut Vec<CanvasDiagnostic>,
    ) {
        if !fields.is_empty() {
            diagnostics.push(CanvasDiagnostic::UnsupportedFields {
                location,
                fields: fields.into_keys().collect(),
            });
        }
    }
}

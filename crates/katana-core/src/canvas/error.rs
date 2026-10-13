use std::fmt;

use super::CanvasLocation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryField {
    X,
    Y,
    Width,
    Height,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanvasError {
    InvalidJson {
        line: usize,
        column: usize,
    },
    InvalidGeometry {
        index: usize,
        field: GeometryField,
    },
    InvalidField {
        location: CanvasLocation,
        field: &'static str,
    },
    DuplicateId {
        location: CanvasLocation,
    },
    DanglingEdge {
        index: usize,
    },
}

impl CanvasError {
    pub(super) fn from_json(error: serde_json::Error) -> Self {
        Self::InvalidJson {
            line: error.line(),
            column: error.column(),
        }
    }
}

impl fmt::Display for CanvasError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        /* WHY: 入力内容を含めず、巨大な属性や外部パスをエラー表示へ流出させない。 */
        match self {
            Self::InvalidJson { line, column } => {
                write!(formatter, "Invalid Canvas JSON at {line}:{column}")
            }
            Self::InvalidGeometry { index, field } => {
                write!(formatter, "Invalid Canvas node {index} geometry: {field:?}")
            }
            Self::InvalidField { location, field } => {
                write!(formatter, "Invalid Canvas field {field} at {location:?}")
            }
            Self::DuplicateId { location } => {
                write!(formatter, "Duplicate Canvas ID at {location:?}")
            }
            Self::DanglingEdge { index } => {
                write!(formatter, "Canvas edge {index} references a missing node")
            }
        }
    }
}

impl std::error::Error for CanvasError {}

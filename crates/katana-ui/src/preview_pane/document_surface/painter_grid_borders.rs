use eframe::egui;
use katana_document_viewer::{
    DocumentGridBorderSide, DocumentGridCellBorders, DocumentGridCoordinate, DocumentSurfaceFrame,
};

pub(super) use super::painter_grid_border_style::{PreparedBorderStyle, parse_style};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PreparedBorderSide {
    pub(super) style: PreparedBorderStyle,
    pub(super) color: Option<egui::Color32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PreparedCellBorders {
    pub(super) coordinate: DocumentGridCoordinate,
    pub(super) left: Option<PreparedBorderSide>,
    pub(super) right: Option<PreparedBorderSide>,
    pub(super) top: Option<PreparedBorderSide>,
    pub(super) bottom: Option<PreparedBorderSide>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct PreparedGridBorders {
    pub(super) cells: Vec<PreparedCellBorders>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum BorderPreparationError {
    UnknownStyle {
        coordinate: DocumentGridCoordinate,
        side: &'static str,
        style: String,
    },
    InvalidColor {
        coordinate: DocumentGridCoordinate,
        side: &'static str,
        color: String,
    },
}

impl std::fmt::Display for BorderPreparationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownStyle {
                coordinate,
                side,
                style,
            } => write!(
                formatter,
                "unknown {side} border style {style:?} at ({},{})",
                coordinate.row, coordinate.column
            ),
            Self::InvalidColor {
                coordinate,
                side,
                color,
            } => write!(
                formatter,
                "invalid {side} border color {color:?} at ({},{})",
                coordinate.row, coordinate.column
            ),
        }
    }
}

pub(super) fn prepare(
    frame: &DocumentSurfaceFrame,
) -> Result<PreparedGridBorders, BorderPreparationError> {
    if frame.grid().is_none() {
        return Ok(PreparedGridBorders::default());
    }
    prepare_entries(frame.grid_cell_border_entries())
}

fn prepare_entries(
    entries: &[(DocumentGridCoordinate, DocumentGridCellBorders)],
) -> Result<PreparedGridBorders, BorderPreparationError> {
    let mut cells = Vec::with_capacity(entries.len());
    for (coordinate, borders) in entries {
        cells.push(prepare_cell(*coordinate, Some(borders))?);
    }
    Ok(PreparedGridBorders { cells })
}

fn prepare_cell(
    coordinate: DocumentGridCoordinate,
    source: Option<&DocumentGridCellBorders>,
) -> Result<PreparedCellBorders, BorderPreparationError> {
    Ok(PreparedCellBorders {
        coordinate,
        left: prepare_side(
            coordinate,
            "left",
            source.and_then(|borders| borders.left.as_ref()),
        )?,
        right: prepare_side(
            coordinate,
            "right",
            source.and_then(|borders| borders.right.as_ref()),
        )?,
        top: prepare_side(
            coordinate,
            "top",
            source.and_then(|borders| borders.top.as_ref()),
        )?,
        bottom: prepare_side(
            coordinate,
            "bottom",
            source.and_then(|borders| borders.bottom.as_ref()),
        )?,
    })
}

fn prepare_side(
    coordinate: DocumentGridCoordinate,
    side: &'static str,
    source: Option<&DocumentGridBorderSide>,
) -> Result<Option<PreparedBorderSide>, BorderPreparationError> {
    let Some(source) = source else {
        return Ok(None);
    };
    if source.style == "none" {
        return Ok(None);
    }
    let style = parse_style(coordinate, side, &source.style)?;
    let color = source
        .color
        .as_deref()
        .map(|value| {
            egui::Color32::from_hex(value).map_err(|_| BorderPreparationError::InvalidColor {
                coordinate,
                side,
                color: value.to_owned(),
            })
        })
        .transpose()?;
    Ok(Some(PreparedBorderSide { style, color }))
}

#[cfg(test)]
#[path = "painter_grid_borders_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "painter_grid_border_batch_tests.rs"]
mod batch_tests;

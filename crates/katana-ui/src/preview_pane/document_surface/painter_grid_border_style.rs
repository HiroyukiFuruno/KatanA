use super::painter_grid_borders::BorderPreparationError;
use katana_document_viewer::DocumentGridCoordinate;

const THICK_BORDER_WIDTH: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PreparedBorderStyle {
    Hair,
    Thin,
    Medium,
    Thick,
    Double,
    Dotted,
    Dashed,
    DashDot,
    DashDotDot,
    MediumDashed,
    MediumDashDot,
    MediumDashDotDot,
    SlantDashDot,
    Solid,
}

impl PreparedBorderStyle {
    pub(super) const fn width(self) -> f32 {
        match self {
            Self::Medium
            | Self::MediumDashed
            | Self::MediumDashDot
            | Self::MediumDashDotDot
            | Self::SlantDashDot => 2.0,
            Self::Thick | Self::Double => THICK_BORDER_WIDTH,
            _ => 1.0,
        }
    }
}

pub(super) fn parse_style(
    coordinate: DocumentGridCoordinate,
    side: &'static str,
    style: &str,
) -> Result<PreparedBorderStyle, BorderPreparationError> {
    let parsed = match style {
        "hair" => PreparedBorderStyle::Hair,
        "thin" => PreparedBorderStyle::Thin,
        "medium" => PreparedBorderStyle::Medium,
        "thick" => PreparedBorderStyle::Thick,
        "double" => PreparedBorderStyle::Double,
        "dotted" => PreparedBorderStyle::Dotted,
        "dashed" => PreparedBorderStyle::Dashed,
        "dashDot" => PreparedBorderStyle::DashDot,
        "dashDotDot" => PreparedBorderStyle::DashDotDot,
        "mediumDashed" => PreparedBorderStyle::MediumDashed,
        "mediumDashDot" => PreparedBorderStyle::MediumDashDot,
        "mediumDashDotDot" => PreparedBorderStyle::MediumDashDotDot,
        "slantDashDot" => PreparedBorderStyle::SlantDashDot,
        "solid" => PreparedBorderStyle::Solid,
        other => {
            return Err(BorderPreparationError::UnknownStyle {
                coordinate,
                side,
                style: other.to_owned(),
            });
        }
    };
    Ok(parsed)
}

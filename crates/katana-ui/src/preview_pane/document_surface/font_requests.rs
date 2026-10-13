use std::collections::BTreeSet;

use katana_document_viewer::{DocumentGridSurfaceFrame, DocumentSurfaceFrame};

use crate::font_loader::office_faces::{FontFaceRequest, FontFamilyIdentity};

#[cfg(test)]
use super::painter_tests;

#[cfg(test)]
#[path = "font_requests_tests.rs"]
mod tests;

pub(super) struct DocumentFontRequestsOps;

impl DocumentFontRequestsOps {
    pub(super) fn project(frame: &DocumentSurfaceFrame) -> Vec<FontFaceRequest> {
        let Some(grid) = frame.grid() else {
            return Vec::new();
        };
        Self::grid_requests(grid)
    }

    fn grid_requests(grid: &DocumentGridSurfaceFrame) -> Vec<FontFaceRequest> {
        let mut requests = BTreeSet::new();
        for cell in &grid.cells {
            if cell.text.is_empty() {
                continue;
            }
            let family = cell.appearance.font_family.trim();
            if family.is_empty() {
                continue;
            }
            Self::insert_style_requests(
                &mut requests,
                &FontFamilyIdentity::key(family),
                cell.appearance.bold,
                cell.appearance.italic,
            );
        }
        requests.into_iter().collect()
    }

    fn insert_style_requests(
        requests: &mut BTreeSet<FontFaceRequest>,
        family: &str,
        bold: bool,
        italic: bool,
    ) {
        let mut insert = |bold, italic| {
            requests.insert(FontFaceRequest {
                family: family.to_owned(),
                bold,
                italic,
            });
        };
        insert(bold, italic);
        if bold && italic {
            insert(true, false);
            insert(false, true);
        }
        if bold || italic {
            insert(false, false);
        }
    }
}

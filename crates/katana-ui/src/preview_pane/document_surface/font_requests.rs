use std::collections::BTreeSet;

use katana_document_viewer::{DocumentGridSurfaceFrame, DocumentSurfaceFrame};

use crate::font_loader::office_faces::FontFaceRequest;

pub(super) struct DocumentFontRequestsOps;

impl DocumentFontRequestsOps {
    pub(super) fn project(frame: &DocumentSurfaceFrame) -> Vec<FontFaceRequest> {
        let Some(grid) = frame.grid() else {
            return Vec::new();
        };
        Self::grid_requests(grid)
    }

    fn grid_requests(grid: &DocumentGridSurfaceFrame) -> Vec<FontFaceRequest> {
        grid.cells
            .iter()
            .filter(|cell| !cell.text.is_empty())
            .filter_map(|cell| {
                let family = cell.appearance.font_family.trim();
                (!family.is_empty()).then(|| FontFaceRequest {
                    family: family.to_ascii_lowercase(),
                    bold: cell.appearance.bold,
                    italic: cell.appearance.italic,
                })
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::DocumentFontRequestsOps;

    #[test]
    fn empty_and_blank_cells_do_not_request_fonts() {
        let mut grid = super::super::painter_tests::grid_surface();
        grid.cells[0].text.clear();
        assert!(DocumentFontRequestsOps::grid_requests(&grid).is_empty());
        grid.cells[0].text = "visible".to_owned();
        grid.cells[0].appearance.font_family = " ".to_owned();
        assert!(DocumentFontRequestsOps::grid_requests(&grid).is_empty());
    }

    #[test]
    fn family_case_is_deduplicated_but_actual_styles_are_distinct() {
        let mut grid = super::super::painter_tests::grid_surface();
        let mut regular = grid.cells[0].clone();
        regular.appearance.font_family = " Arial ".to_owned();
        let mut same = regular.clone();
        same.appearance.font_family = "arial".to_owned();
        let mut bold = regular.clone();
        bold.appearance.bold = true;
        let mut italic = regular.clone();
        italic.appearance.italic = true;
        grid.cells = vec![regular, same, bold, italic];
        let requests = DocumentFontRequestsOps::grid_requests(&grid);
        assert_eq!(requests.len(), 3);
        assert!(requests.iter().all(|request| request.family == "arial"));
        assert!(
            requests
                .iter()
                .any(|request| request.bold && !request.italic)
        );
        assert!(
            requests
                .iter()
                .any(|request| !request.bold && request.italic)
        );
    }
}

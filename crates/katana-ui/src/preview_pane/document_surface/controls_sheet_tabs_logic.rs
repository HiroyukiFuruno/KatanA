pub(crate) struct SheetTabLabelOps;

impl SheetTabLabelOps {
    pub(crate) fn labels(labels: &[String], item_count: usize) -> Vec<(usize, String)> {
        (0..item_count)
            .map(|index| {
                let label = labels
                    .get(index)
                    .filter(|label| !label.trim().is_empty())
                    .cloned()
                    .unwrap_or_else(|| index.saturating_add(1).to_string());
                (index, label)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::SheetTabLabelOps;
    use katana_document_viewer::ViewerDocumentFormat;

    #[test]
    fn sheet_tabs_use_document_labels_with_a_bounded_fallback() {
        assert_eq!(
            SheetTabLabelOps::labels(&["Summary".to_owned(), "Data".to_owned()], 3),
            vec![
                (0, "Summary".to_owned()),
                (1, "Data".to_owned()),
                (2, "3".to_owned())
            ]
        );
    }

    #[test]
    fn xlsx_reserves_a_bottom_sheet_tab_rail() {
        use super::super::sheet_tab_rail_height;

        assert!(sheet_tab_rail_height(ViewerDocumentFormat::Xlsx) > 0.0);
        assert_eq!(sheet_tab_rail_height(ViewerDocumentFormat::Pdf), 0.0);
    }
}

pub(crate) struct TocAvailability;

impl TocAvailability {
    pub(crate) fn for_path(path: Option<&std::path::Path>) -> bool {
        !matches!(
            path.and_then(katana_core::document_source::BinaryDocumentFormat::from_path),
            Some(
                katana_core::document_source::BinaryDocumentFormat::Docx
                    | katana_core::document_source::BinaryDocumentFormat::Xlsx
                    | katana_core::document_source::BinaryDocumentFormat::Pptx
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::TocAvailability;

    #[test]
    fn office_documents_disable_table_of_contents() {
        for path in ["report.docx", "book.xlsx", "deck.pptx"] {
            assert!(!TocAvailability::for_path(Some(std::path::Path::new(path))));
        }
    }

    #[test]
    fn pdf_and_markdown_keep_table_of_contents_available() {
        assert!(TocAvailability::for_path(Some(std::path::Path::new(
            "manual.pdf"
        ))));
        assert!(TocAvailability::for_path(Some(std::path::Path::new(
            "readme.md"
        ))));
    }
}

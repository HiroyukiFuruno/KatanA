pub(crate) struct TocAvailability;

#[derive(Clone, Copy)]
pub(crate) enum PreviewMenu {
    Toc,
    Export,
    Story,
    Tools,
    Slideshow,
}

pub(crate) struct PreviewMenuAvailability;

impl PreviewMenuAvailability {
    pub(crate) fn for_path(path: Option<&std::path::Path>, menu: PreviewMenu) -> bool {
        let Some(path) = path else {
            return !matches!(menu, PreviewMenu::Slideshow);
        };
        let is_office = matches!(
            katana_core::document_source::BinaryDocumentFormat::from_path(path),
            Some(
                katana_core::document_source::BinaryDocumentFormat::Docx
                    | katana_core::document_source::BinaryDocumentFormat::Xlsx
                    | katana_core::document_source::BinaryDocumentFormat::Pptx
            )
        );
        let is_html = path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("html") || extension.eq_ignore_ascii_case("htm")
            });
        let format = katana_core::document_source::BinaryDocumentFormat::from_path(path);
        let is_image = katana_core::workspace::TreeEntry::path_is_image(path);
        match menu {
            PreviewMenu::Toc => !is_office && !is_html && !is_image,
            PreviewMenu::Tools => !is_office && !is_html && !is_image,
            PreviewMenu::Export | PreviewMenu::Story => {
                !is_office && !is_html && !is_image && format.is_none()
            }
            PreviewMenu::Slideshow => {
                !is_office && !is_html && !is_image && format.is_none()
                    || matches!(
                        format,
                        Some(katana_core::document_source::BinaryDocumentFormat::Pptx)
                    )
            }
        }
    }
}

impl TocAvailability {
    pub(crate) fn for_path(path: Option<&std::path::Path>) -> bool {
        PreviewMenuAvailability::for_path(path, PreviewMenu::Toc)
    }
}

#[cfg(test)]
mod tests {
    use super::{PreviewMenu, PreviewMenuAvailability, TocAvailability};

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

    #[test]
    fn pdf_keeps_view_tools_without_markdown_only_menus() {
        let path = std::path::Path::new("manual.pdf");
        assert!(PreviewMenuAvailability::for_path(
            Some(path),
            PreviewMenu::Tools
        ));
        for menu in [
            PreviewMenu::Export,
            PreviewMenu::Story,
            PreviewMenu::Slideshow,
        ] {
            assert!(!PreviewMenuAvailability::for_path(Some(path), menu));
        }
    }

    #[test]
    fn html_and_non_pptx_office_disable_all_preview_menus() {
        for path in ["report.html", "REPORT.HTM", "report.docx", "book.xlsx"] {
            for menu in [
                PreviewMenu::Toc,
                PreviewMenu::Export,
                PreviewMenu::Story,
                PreviewMenu::Tools,
                PreviewMenu::Slideshow,
            ] {
                assert!(!PreviewMenuAvailability::for_path(
                    Some(std::path::Path::new(path)),
                    menu
                ));
            }
        }
    }

    #[test]
    fn pptx_enables_only_slideshow() {
        let path = std::path::Path::new("deck.pptx");
        assert!(PreviewMenuAvailability::for_path(
            Some(path),
            PreviewMenu::Slideshow
        ));
        for menu in [
            PreviewMenu::Toc,
            PreviewMenu::Export,
            PreviewMenu::Story,
            PreviewMenu::Tools,
        ] {
            assert!(!PreviewMenuAvailability::for_path(Some(path), menu));
        }
    }

    #[test]
    fn images_disable_document_menus() {
        let path = std::path::Path::new("photo.png");
        for menu in [
            PreviewMenu::Toc,
            PreviewMenu::Export,
            PreviewMenu::Story,
            PreviewMenu::Tools,
            PreviewMenu::Slideshow,
        ] {
            assert!(!PreviewMenuAvailability::for_path(Some(path), menu));
        }
    }
}

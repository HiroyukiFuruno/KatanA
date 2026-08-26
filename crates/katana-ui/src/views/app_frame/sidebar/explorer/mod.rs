pub mod bottom_items;
pub mod drag;
pub mod history_popup;
mod hover;
pub mod items;
mod rail_items;
pub mod theme_bridge_popup;
pub mod ui;

fn active_explorer_path(
    document: Option<&katana_core::document::Document>,
    workspace_root: Option<&std::path::Path>,
) -> Option<std::path::PathBuf> {
    document
        .map(|document| document.path.clone())
        .filter(|path| workspace_root.is_some_and(|root| path.starts_with(root)))
}

#[cfg(test)]
mod tests {
    use super::active_explorer_path;

    #[test]
    fn local_reference_document_remains_selectable_in_explorer() {
        let root = std::path::Path::new("/workspace");
        let mut document =
            katana_core::document::Document::new("/workspace/report.xlsx", String::new());
        document.is_reference = true;

        assert_eq!(
            active_explorer_path(Some(&document), Some(root)),
            Some(std::path::PathBuf::from("/workspace/report.xlsx"))
        );
    }

    #[test]
    fn virtual_reference_document_is_not_selected_in_explorer() {
        let root = std::path::Path::new("/workspace");
        let mut document =
            katana_core::document::Document::new("Katana://URL/example.html", String::new());
        document.is_reference = true;

        assert_eq!(active_explorer_path(Some(&document), Some(root)), None);
    }
}

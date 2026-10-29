use super::ImageDocumentOps;
use katana_platform::FilesystemService;

#[test]
fn missing_image_stays_a_reference_for_the_background_loader() {
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join("not-yet-available.png");
    let document = ImageDocumentOps::load_or_create(&FilesystemService, &path)
        .expect("reference construction does not require source IO");
    assert!(document.is_reference);
    assert_eq!(document.path, path);
    assert_eq!(document.buffer, ImageDocumentOps::buffer_for_path(&path));
}

#[test]
fn unavailable_binary_formats_do_not_fall_through_to_text_loading() {
    let directory = tempfile::tempdir().expect("fixture directory");
    for extension in ["pdf", "docx", "xlsx", "pptx"] {
        let path = directory.path().join(format!("unavailable.{extension}"));
        let document = ImageDocumentOps::load_or_create(&FilesystemService, &path)
            .expect("binary reference construction");
        assert!(document.is_reference, "{extension}");
        assert!(document.buffer.is_empty(), "{extension}");
    }
}

#[test]
fn invalid_metadata_path_does_not_change_supported_format_classification() {
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join(format!("{}.png", "x".repeat(300)));
    assert!(std::fs::metadata(&path).is_err());
    assert!(ImageDocumentOps::is_image_path(&path));
}

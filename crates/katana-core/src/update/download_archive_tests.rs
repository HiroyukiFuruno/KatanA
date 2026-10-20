use super::{UpdateDownloadOps, UpdateProgress};

const EXECUTABLE_MODE: u32 = 0o755;

fn create_archive(
    directory: &tempfile::TempDir,
    method: zip::CompressionMethod,
) -> std::path::PathBuf {
    use std::io::Write;

    let archive_path = directory.path().join("update.zip");
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(EXECUTABLE_MODE);
    let mut archive = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
    archive.add_directory("nested/", options).unwrap();
    archive.start_file("nested/application", options).unwrap();
    archive.write_all(b"verified update bytes").unwrap();
    archive.finish().unwrap();
    archive_path
}

fn assert_extracted(directory: &tempfile::TempDir, archive_path: &std::path::Path) {
    /* WHY: 対応する圧縮方式で展開後のbytes・権限・完了通知を同時に守る。 */
    let output = directory.path().join("application");
    let mut progress = Vec::new();
    UpdateDownloadOps::extract_update(archive_path, &output, |event| progress.push(event)).unwrap();

    assert_eq!(
        std::fs::read(output.join("nested/application")).unwrap(),
        b"verified update bytes"
    );
    assert!(matches!(
        progress.last(),
        Some(UpdateProgress::Extracting {
            current: 2,
            total: 2
        })
    ));
    #[cfg(unix)]
    assert_permissions(&output.join("nested/application"));
}

#[cfg(unix)]
fn assert_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        path.metadata().unwrap().permissions().mode() & 0o777,
        EXECUTABLE_MODE
    );
}

fn assert_archive_extracts(method: zip::CompressionMethod) {
    let directory = tempfile::tempdir().unwrap();
    let archive_path = create_archive(&directory, method);
    assert_archive_method(&archive_path, method);
    assert_extracted(&directory, &archive_path);
}

fn assert_archive_method(path: &std::path::Path, expected: zip::CompressionMethod) {
    let file = std::fs::File::open(path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    assert_eq!(
        archive.by_name("nested/application").unwrap().compression(),
        expected
    );
}

#[test]
fn deflated_archive_preserves_nested_bytes_and_progress() {
    assert_archive_extracts(zip::CompressionMethod::Deflated);
}

#[test]
fn xz_archive_preserves_nested_bytes_and_progress() {
    assert_archive_extracts(zip::CompressionMethod::Xz);
}

use super::{UpdateDownloadOps, UpdateProgress};

#[test]
fn test_download_update() {
    let proxy_env = crate::update::proxy_env_test::ProxyEnvGuard::capture();
    proxy_env.clear_proxy_env();

    use std::io::Write;
    use std::net::TcpListener;
    use std::thread;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{}/update.zip", port);
    thread::spawn(move || {
        use std::io::Read;
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut buf = [0; 1024];
        let _ = stream.read(&mut buf);
        let body = b"mock zip payload";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.write_all(body);
    });

    let temp_dir = tempfile::tempdir().unwrap();
    let dest = temp_dir.path().join("update.zip");
    UpdateDownloadOps::download_update(&url, &dest, |_| {}).unwrap();
    assert!(dest.exists());
    assert_eq!(std::fs::read(&dest).unwrap(), b"mock zip payload");
}
#[test]
fn test_extract_update() {
    use std::io::Write;

    let temp_dir = tempfile::tempdir().unwrap();
    let zip_path = temp_dir.path().join("test.zip");
    let extract_dir = temp_dir.path().join("extracted");
    let file = std::fs::File::create(&zip_path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file("hello.txt", options).unwrap();
    zip.write_all(b"Hello from ZIP").unwrap();
    zip.add_directory("somedir/", options).unwrap();
    zip.start_file("../outside.txt", options).unwrap();
    zip.write_all(b"Should be skipped").unwrap();
    zip.finish().unwrap();

    UpdateDownloadOps::extract_update(&zip_path, &extract_dir, |_| {}).unwrap();
    let extracted_file = extract_dir.join("hello.txt");
    assert!(extracted_file.exists());
    assert_eq!(std::fs::read(&extracted_file).unwrap(), b"Hello from ZIP");
    assert!(extract_dir.join("somedir").is_dir());
}
#[test]
fn zip_update_extracts_nested_files_without_directory_traversal() {
    use std::io::Write;

    let directory = tempfile::tempdir().unwrap();
    let archive_path = directory.path().join("update.zip");
    let output = directory.path().join("application");
    let mut archive = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for entry in ["nested/application.txt", "../outside.txt"] {
        archive.start_file(entry, options).unwrap();
        archive.write_all(b"verified update bytes").unwrap();
    }
    archive.finish().unwrap();

    UpdateDownloadOps::extract_update(&archive_path, &output, |_| {}).unwrap();
    assert_eq!(
        std::fs::read(output.join("nested/application.txt")).unwrap(),
        b"verified update bytes"
    );
    assert!(!directory.path().join("outside.txt").exists());
    assert!(!output.join("outside.txt").exists());
}

const EXECUTABLE_MODE: u32 = 0o755;
#[cfg(unix)]
const UNIX_PERMISSION_MASK: u32 = 0o777;
#[cfg(unix)]
const UNIX_MODE_MASK: u32 = 0o7777;
#[cfg(unix)]
const CENTRAL_DIRECTORY_EXTERNAL_ATTRIBUTES_OFFSET: usize = 38;
#[cfg(unix)]
const CENTRAL_DIRECTORY_MODE_SHIFT: u32 = 16;

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
        path.metadata().unwrap().permissions().mode() & UNIX_PERMISSION_MASK,
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

#[cfg(unix)]
#[test]
fn zip_archive_strips_privileged_permission_bits() {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let archive_path = directory.path().join("privileged.zip");
    let options = zip::write::SimpleFileOptions::default().unix_permissions(0o755);
    let mut archive = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
    archive.start_file("application", options).unwrap();
    archive.write_all(b"verified update bytes").unwrap();
    archive.finish().unwrap();
    patch_central_directory_mode(&archive_path, 0o1006755);
    let mut patched_archive =
        zip::ZipArchive::new(std::fs::File::open(&archive_path).unwrap()).unwrap();
    let patched_entry = patched_archive.by_name("application").unwrap();
    assert_eq!(patched_entry.external_attributes(), 0o1006755 << 16);
    assert_eq!(patched_entry.unix_mode(), Some(0o755));

    let output = directory.path().join("application");
    UpdateDownloadOps::extract_update(&archive_path, &output, |_| {}).unwrap();

    assert_eq!(
        output
            .join("application")
            .metadata()
            .unwrap()
            .permissions()
            .mode()
            & UNIX_MODE_MASK,
        0o755
    );
}

#[test]
fn zip_archive_rejects_symlink_entries() {
    let directory = tempfile::tempdir().unwrap();
    let archive_path = directory.path().join("symlink.zip");
    let options = zip::write::SimpleFileOptions::default();
    let mut archive = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
    archive
        .add_symlink("application", "../outside", options)
        .unwrap();
    archive.finish().unwrap();

    let output = directory.path().join("application");
    assert!(UpdateDownloadOps::extract_update(&archive_path, &output, |_| {}).is_err());
    assert!(!output.exists());
}

#[cfg(unix)]
fn patch_central_directory_mode(path: &std::path::Path, mode: u32) {
    const CENTRAL_DIRECTORY_SIGNATURE: &[u8; 4] = b"PK\x01\x02";
    let mut bytes = std::fs::read(path).unwrap();
    let signature_offset = bytes
        .windows(CENTRAL_DIRECTORY_SIGNATURE.len())
        .position(|window| window == CENTRAL_DIRECTORY_SIGNATURE)
        .unwrap();
    let attributes_offset = signature_offset + CENTRAL_DIRECTORY_EXTERNAL_ATTRIBUTES_OFFSET;
    assert!(attributes_offset + std::mem::size_of::<u32>() <= bytes.len());
    bytes[attributes_offset..attributes_offset + std::mem::size_of::<u32>()]
        .copy_from_slice(&(mode << CENTRAL_DIRECTORY_MODE_SHIFT).to_le_bytes());
    std::fs::write(path, bytes).unwrap();
}
#[cfg(unix)]
#[test]
fn zip_archive_rejects_preexisting_symlink_output_directory() {
    use std::io::Write;
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let archive_path = directory.path().join("nested.zip");
    let outside = directory.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    let output = directory.path().join("application");
    std::fs::create_dir(&output).unwrap();
    symlink(&outside, output.join("nested")).unwrap();

    let options = zip::write::SimpleFileOptions::default();
    let mut archive = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
    archive.start_file("nested/application", options).unwrap();
    archive.write_all(b"must not escape").unwrap();
    archive.finish().unwrap();

    assert!(UpdateDownloadOps::extract_update(&archive_path, &output, |_| {}).is_err());
    assert!(!outside.join("application").exists());
}
#[cfg(unix)]
#[test]
fn zip_archive_rejects_preexisting_final_symlink() {
    use std::io::Write;
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let archive_path = directory.path().join("final-symlink.zip");
    let outside = directory.path().join("outside");
    std::fs::write(&outside, b"must remain unchanged").unwrap();
    let output = directory.path().join("application");
    std::fs::create_dir(&output).unwrap();
    symlink(&outside, output.join("application")).unwrap();

    let options = zip::write::SimpleFileOptions::default();
    let mut archive = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
    archive.start_file("application", options).unwrap();
    archive.write_all(b"must not overwrite").unwrap();
    archive.finish().unwrap();

    assert!(UpdateDownloadOps::extract_update(&archive_path, &output, |_| {}).is_err());
    assert_eq!(std::fs::read(&outside).unwrap(), b"must remain unchanged");
}

#[cfg(unix)]
#[test]
fn zip_archive_rejects_dangling_symlink_root() {
    use std::io::Write;
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let archive_path = directory.path().join("dangling-root.zip");
    let output = directory.path().join("application");
    symlink(directory.path().join("missing"), &output).unwrap();

    let options = zip::write::SimpleFileOptions::default();
    let mut archive = zip::ZipWriter::new(std::fs::File::create(&archive_path).unwrap());
    archive.start_file("nested/application", options).unwrap();
    archive.write_all(b"must not escape").unwrap();
    archive.finish().unwrap();

    assert!(UpdateDownloadOps::extract_update(&archive_path, &output, |_| {}).is_err());
}

use super::{CheckUpdateError, UpdateDownloadOps};
use std::io::{Read, Write};
use std::net::TcpListener;

const REQUEST_BUFFER_BYTES: usize = 1024;
const RELATIVE_CHILD: &str = "KATANA_UPDATE_RELATIVE_PATH_CHILD";
const RELATIVE_TEST: &str = "update::download_capacity_tests::relative_download_and_archive_paths_preserve_existing_behavior";

#[test]
fn oversized_download_is_rejected_before_body_write_or_progress() {
    let environment = super::proxy_env_test::ProxyEnvGuard::capture();
    environment.clear_proxy_env();
    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = server.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        let (mut connection, _) = server.accept().unwrap();
        let mut request = [0_u8; REQUEST_BUFFER_BYTES];
        assert!(connection.read(&mut request).unwrap() > 0);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            u64::MAX / 2
        );
        connection.write_all(response.as_bytes()).unwrap();
    });
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("update.zip");
    let mut received_progress = false;
    let error = UpdateDownloadOps::download_update(
        &format!("http://{address}/update.zip"),
        &destination,
        |_| received_progress = true,
    )
    .unwrap_err();
    worker.join().unwrap();
    assert!(
        matches!(error.downcast_ref::<CheckUpdateError>(), Some(CheckUpdateError::InsufficientStorage { required_bytes, available_bytes }) if required_bytes > available_bytes)
    );
    assert!(!received_progress);
    assert!(!destination.exists());
}

#[test]
fn relative_download_and_archive_paths_preserve_existing_behavior() {
    if std::env::var_os(RELATIVE_CHILD).is_some() {
        download_relative_archive();
        return;
    }
    let environment = super::proxy_env_test::ProxyEnvGuard::capture();
    environment.clear_proxy_env();
    let directory = tempfile::tempdir().unwrap();
    /* WHY: 相対パスの回帰を検査するため、他の並列検査のcwdを変えず子プロセスへ隔離する。 */
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", RELATIVE_TEST, "--nocapture"])
        .env(RELATIVE_CHILD, "1")
        .current_dir(directory.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn download_relative_archive() {
    let environment = super::proxy_env_test::ProxyEnvGuard::capture();
    environment.clear_proxy_env();
    let payload = relative_archive_payload();
    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = server.local_addr().unwrap();
    let served_payload = payload.clone();
    let worker = std::thread::spawn(move || serve_archive(server, &served_payload));
    let mut downloaded = false;
    UpdateDownloadOps::download_update(
        &format!("http://{address}/update.zip"),
        "update.zip",
        |progress| {
            if let super::UpdateProgress::Downloading {
                downloaded: bytes, ..
            } = progress
            {
                downloaded |= bytes == payload.len() as u64;
            }
        },
    )
    .unwrap();
    worker.join().unwrap();
    assert!(downloaded);
    assert_eq!(std::fs::read("update.zip").unwrap(), payload);
    super::download::storage::UpdateStorageOps::check_archive(
        std::path::Path::new("update.zip"),
        std::path::Path::new("KatanA Desktop.app"),
    )
    .unwrap();
}

fn relative_archive_payload() -> Vec<u8> {
    let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    archive
        .start_file("application", zip::write::SimpleFileOptions::default())
        .unwrap();
    archive.write_all(b"application payload").unwrap();
    archive.finish().unwrap().into_inner()
}

fn serve_archive(server: TcpListener, payload: &[u8]) {
    let (mut connection, _) = server.accept().unwrap();
    let mut request = [0_u8; REQUEST_BUFFER_BYTES];
    assert!(connection.read(&mut request).unwrap() > 0);
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    connection.write_all(header.as_bytes()).unwrap();
    connection.write_all(payload).unwrap();
}

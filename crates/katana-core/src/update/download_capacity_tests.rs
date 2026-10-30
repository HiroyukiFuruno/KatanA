use super::{CheckUpdateError, UpdateDownloadOps};
use std::io::{Read, Write};
use std::net::TcpListener;

#[test]
fn oversized_download_is_rejected_before_body_write_or_progress() {
    let environment = super::proxy_env_test::ProxyEnvGuard::capture();
    environment.clear_proxy_env();
    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = server.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        let (mut connection, _) = server.accept().unwrap();
        let mut request = [0_u8; 1024];
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

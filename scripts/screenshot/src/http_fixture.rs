use crate::request::HttpServerFixture;
use anyhow::{bail, ensure, Context, Result};
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

pub struct FixtureHttpServer {
    base_url: String,
    address: SocketAddr,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<std::io::Result<()>>>,
}

impl FixtureHttpServer {
    pub fn start(root: &Path, config: &HttpServerFixture) -> Result<Self> {
        ensure!(
            config.mount_prefix.starts_with('/') && config.mount_prefix.ends_with('/'),
            "fixture HTTP mount_prefix must start and end with '/': {}",
            config.mount_prefix
        );
        let root = root
            .canonicalize()
            .with_context(|| format!("fixture HTTP root is unavailable: {}", root.display()))?;
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&requests);
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let mount_prefix = config.mount_prefix.clone();
        let redirects = config.redirects.clone();
        let worker = std::thread::spawn(move || {
            serve(
                listener,
                &root,
                &mount_prefix,
                &redirects,
                &recorded,
                &worker_stop,
            )
        });
        Ok(Self {
            base_url: format!("http://{address}"),
            address,
            requests,
            stop,
            worker: Some(worker),
        })
    }

    pub fn url(&self, path: &str) -> Result<String> {
        ensure!(
            path.starts_with('/'),
            "fixture URL path must start with '/': {path}"
        );
        Ok(format!("{}{path}", self.base_url))
    }

    pub fn assert_requested(&self, expected: &[String]) -> Result<()> {
        let actual = self.requested_paths()?;
        let missing = missing_requests(expected, &actual);
        if !missing.is_empty() {
            bail!("fixture HTTP requests are missing {missing:?}; received {actual:?}");
        }
        println!("  fixture HTTP requests matched: {expected:?}");
        Ok(())
    }

    pub fn requested_paths(&self) -> Result<Vec<String>> {
        Ok(self
            .requests
            .lock()
            .map_err(|_| anyhow::anyhow!("fixture HTTP request log lock was poisoned"))?
            .clone())
    }
}

fn missing_requests(expected: &[String], actual: &[String]) -> Vec<String> {
    expected
        .iter()
        .filter(|path| !actual.contains(path))
        .cloned()
        .collect()
}

impl Drop for FixtureHttpServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = TcpStream::connect(self.address);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn serve(
    listener: TcpListener,
    root: &Path,
    mount_prefix: &str,
    redirects: &std::collections::HashMap<String, String>,
    requests: &Mutex<Vec<String>>,
    stop: &AtomicBool,
) -> std::io::Result<()> {
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                if stop.load(Ordering::Acquire) {
                    break;
                }
                stream.set_nonblocking(false)?;
                if let Err(error) =
                    serve_request(&mut stream, root, mount_prefix, redirects, requests)
                {
                    match error.kind() {
                        std::io::ErrorKind::UnexpectedEof
                        | std::io::ErrorKind::InvalidData
                        | std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::WouldBlock
                        | std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::BrokenPipe => continue,
                        _ => return Err(error),
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn serve_request(
    stream: &mut TcpStream,
    root: &Path,
    mount_prefix: &str,
    redirects: &std::collections::HashMap<String, String>,
    requests: &Mutex<Vec<String>>,
) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let path = request_path(stream)?;
    requests
        .lock()
        .map_err(|_| std::io::Error::other("fixture HTTP request log lock was poisoned"))?
        .push(path.clone());
    if let Some(location) = redirects.get(&path) {
        return write_response(stream, "302 Found", &[('L', location)], b"");
    }
    match resolve_file(root, mount_prefix, &path) {
        Ok(path) => {
            let body = std::fs::read(&path)?;
            let content_type = content_type(&path);
            write_response(stream, "200 OK", &[('C', content_type)], &body)
        }
        Err(_) => write_response(stream, "404 Not Found", &[], b"not found"),
    }
}

fn request_path(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut request = [0_u8; 8192];
    let length = stream.read(&mut request)?;
    if length == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "fixture HTTP client disconnected before sending a request",
        ));
    }
    let text = std::str::from_utf8(&request[..length])
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    let raw = text.split_ascii_whitespace().nth(1).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "fixture HTTP request path is missing",
        )
    })?;
    Ok(raw.split('?').next().unwrap_or(raw).to_string())
}

fn resolve_file(root: &Path, mount_prefix: &str, request_path: &str) -> Result<PathBuf> {
    let relative = request_path
        .strip_prefix(mount_prefix)
        .context("request is outside fixture HTTP mount")?;
    ensure!(
        !relative.is_empty(),
        "fixture HTTP request has no file path"
    );
    let relative_path = Path::new(relative);
    ensure!(
        relative_path
            .components()
            .all(|component| matches!(component, Component::Normal(_))),
        "fixture HTTP request contains an unsafe path"
    );
    let resolved = root.join(relative_path).canonicalize()?;
    ensure!(
        resolved.starts_with(root),
        "fixture HTTP request escaped its root"
    );
    ensure!(resolved.is_file(), "fixture HTTP request is not a file");
    Ok(resolved)
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    }
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    headers: &[(char, &str)],
    body: &[u8],
) -> std::io::Result<()> {
    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (kind, value) in headers {
        let name = match kind {
            'L' => "Location",
            'C' => "Content-Type",
            _ => continue,
        };
        response.push_str(&format!("{name}: {value}\r\n"));
    }
    response.push_str("\r\n");
    stream.write_all(response.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()?;
    stream.shutdown(Shutdown::Write)
}

#[cfg(test)]
mod tests {
    use super::{content_type, resolve_file, FixtureHttpServer};
    use crate::request::HttpServerFixture;
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::path::Path;

    #[test]
    fn fixture_paths_stay_inside_the_mounted_workspace() -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        std::fs::write(root.path().join("index.html"), "<!doctype html>")?;
        let canonical_root = root.path().canonicalize()?;

        assert_eq!(
            resolve_file(&canonical_root, "/app/", "/app/index.html")?,
            canonical_root.join("index.html")
        );
        assert!(resolve_file(&canonical_root, "/app/", "/outside.html").is_err());
        assert!(resolve_file(&canonical_root, "/app/", "/app/../index.html").is_err());
        assert_eq!(
            content_type(Path::new("page.css")),
            "text/css; charset=utf-8"
        );
        Ok(())
    }

    #[test]
    fn disconnected_client_does_not_stop_following_fixture_requests(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        std::fs::write(root.path().join("index.html"), "<!doctype html>")?;
        let server = FixtureHttpServer::start(
            root.path(),
            &HttpServerFixture {
                mount_prefix: "/app/".to_string(),
                redirects: HashMap::new(),
            },
        )?;

        drop(TcpStream::connect(server.address)?);
        let mut client = TcpStream::connect(server.address)?;
        client.write_all(b"GET /app/index.html HTTP/1.1\r\nHost: localhost\r\n\r\n")?;
        let mut response = String::new();
        client.read_to_string(&mut response)?;

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        server.assert_requested(&["/app/index.html".to_string()])?;
        Ok(())
    }
}

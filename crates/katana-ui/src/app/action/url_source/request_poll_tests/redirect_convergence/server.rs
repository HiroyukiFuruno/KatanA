use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

mod request_tests;

pub(super) const LOAD_TIMEOUT: Duration = Duration::from_secs(2);
const REQUEST_BUFFER_BYTES: usize = 1024;
const PDF: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pdf"
));

#[derive(Clone, Copy)]
pub(super) enum Release {
    A,
    B,
}
pub(super) struct RedirectServer {
    pub(super) a_url: String,
    pub(super) b_url: String,
    pub(super) final_url: String,
    ready_rx: mpsc::Receiver<()>,
    release_tx: mpsc::Sender<Release>,
    server: thread::JoinHandle<std::io::Result<Vec<String>>>,
}

impl RedirectServer {
    pub(super) fn body(marker: u8) -> Vec<u8> {
        [PDF, b"\n% KatanA redirect source ", &[marker], b"\n"].concat()
    }

    pub(super) fn start() -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let base = format!("http://{}", listener.local_addr()?);
        let (ready_tx, ready_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let server = thread::spawn(move || serve(listener, ready_tx, release_rx));
        Ok(Self {
            a_url: format!("{base}/a"),
            b_url: format!("{base}/b"),
            final_url: format!("{base}/final.pdf"),
            ready_rx,
            release_tx,
            server,
        })
    }
    pub(super) fn wait_ready(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.ready_rx.recv_timeout(LOAD_TIMEOUT)?;
        Ok(())
    }
    pub(super) fn release(&self, release: Release) -> Result<(), Box<dyn std::error::Error>> {
        self.release_tx.send(release)?;
        Ok(())
    }
    pub(super) fn join(self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Ok(self
            .server
            .join()
            .map_err(|_| "redirect server panicked")??)
    }
}

fn serve(
    listener: TcpListener,
    ready_tx: mpsc::Sender<()>,
    release_rx: mpsc::Receiver<Release>,
) -> std::io::Result<Vec<String>> {
    let (mut a, mut b, mut log) = accept_sources(&listener)?;
    let mut final_a = redirect_and_accept(&listener, &mut a, &mut log)?;
    let mut final_b = redirect_and_accept(&listener, &mut b, &mut log)?;
    ready_tx.send(()).map_err(|_| broken_pipe())?;
    for _ in [Release::A, Release::B] {
        let release = release_rx
            .recv_timeout(LOAD_TIMEOUT)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::TimedOut, error))?;
        match release {
            Release::A => write_pdf(&mut final_a, &RedirectServer::body(b'A'))?,
            Release::B => write_pdf(&mut final_b, &RedirectServer::body(b'B'))?,
        }
    }
    Ok(log)
}

fn accept_sources(listener: &TcpListener) -> std::io::Result<(TcpStream, TcpStream, Vec<String>)> {
    let (first, first_path) = accept_request(listener)?;
    let (second, second_path) = accept_request(listener)?;
    if first_path == "/a" {
        Ok((first, second, vec![first_path, second_path]))
    } else {
        Ok((second, first, vec![second_path, first_path]))
    }
}
fn redirect_and_accept(
    listener: &TcpListener,
    source: &mut TcpStream,
    log: &mut Vec<String>,
) -> std::io::Result<TcpStream> {
    write!(
        source,
        "HTTP/1.1 302 Found\r\nLocation: /final.pdf\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )?;
    let (final_stream, path) = accept_request(listener)?;
    log.push(path);
    Ok(final_stream)
}
fn accept_request(listener: &TcpListener) -> std::io::Result<(TcpStream, String)> {
    let deadline = Instant::now() + LOAD_TIMEOUT;
    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_nonblocking(false)?;
                stream.set_read_timeout(Some(LOAD_TIMEOUT))?;
                let path = request_path(&mut stream)?;
                return Ok((stream, path));
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
            {
                thread::yield_now()
            }
            Err(error) => return Err(error),
        }
    }
}

fn request_path(reader: impl Read) -> std::io::Result<String> {
    let mut request = Vec::new();
    BufReader::new(reader.take(REQUEST_BUFFER_BYTES as u64)).read_until(b'\n', &mut request)?;
    if !request.ends_with(b"\r\n") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "incomplete request line",
        ));
    }
    let line = std::str::from_utf8(&request)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    line.split_whitespace()
        .nth(1)
        .map(str::to_owned)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "request path missing"))
}
fn write_pdf(stream: &mut TcpStream, body: &[u8]) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/pdf\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)
}
fn broken_pipe() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::BrokenPipe, "test receiver dropped")
}

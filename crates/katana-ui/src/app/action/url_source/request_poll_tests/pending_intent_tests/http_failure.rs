use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub(in super::super) const LOAD_TIMEOUT: Duration = Duration::from_secs(2);
const REQUEST_BUFFER_BYTES: usize = 1024;
const HTTP_OK: u16 = 200;
const HTTP_INTERNAL_SERVER_ERROR: u16 = 500;

pub(in super::super) struct ControlledResponse {
    pub(in super::super) url: String,
    accepted_rx: mpsc::Receiver<()>,
    release_tx: mpsc::Sender<()>,
    server: thread::JoinHandle<std::io::Result<String>>,
}

pub(super) type ControlledFailure = ControlledResponse;

impl ControlledResponse {
    pub(in super::super) fn start() -> std::io::Result<Self> {
        Self::start_with(HTTP_INTERNAL_SERVER_ERROR, "b.html", Vec::new())
    }

    pub(in super::super) fn start_with(
        status: u16,
        path: &str,
        body: Vec<u8>,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let url = format!("http://{}/{}", listener.local_addr()?, path);
        let (accepted_tx, accepted_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let server =
            thread::spawn(move || serve_response(listener, accepted_tx, release_rx, status, body));
        Ok(Self {
            url,
            accepted_rx,
            release_tx,
            server,
        })
    }

    pub(in super::super) fn wait_accepted(&self) -> Result<(), mpsc::RecvTimeoutError> {
        self.accepted_rx.recv_timeout(LOAD_TIMEOUT)
    }

    pub(in super::super) fn finish(self) -> Result<String, Box<dyn std::error::Error>> {
        self.release_tx.send(())?;
        Ok(self.server.join().map_err(|_| "server panicked")??)
    }
}

fn serve_response(
    listener: TcpListener,
    accepted_tx: mpsc::Sender<()>,
    release_rx: mpsc::Receiver<()>,
    status: u16,
    body: Vec<u8>,
) -> std::io::Result<String> {
    let mut stream = accept_connection(&listener)?;
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(LOAD_TIMEOUT))?;
    let mut request = [0; REQUEST_BUFFER_BYTES];
    let bytes = stream.read(&mut request)?;
    let path = String::from_utf8_lossy(&request[..bytes])
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_owned();
    accepted_tx.send(()).map_err(|_| broken_pipe())?;
    release_rx
        .recv_timeout(LOAD_TIMEOUT)
        .map_err(|_| timed_out())?;
    write_response(&mut stream, status, &body)?;
    Ok(path)
}

fn write_response(stream: &mut TcpStream, status: u16, body: &[u8]) -> std::io::Result<()> {
    let reason = if status == HTTP_OK {
        "OK"
    } else {
        "Internal Server Error"
    };
    let headers = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(headers.as_bytes())?;
    stream.write_all(body)
}

fn accept_connection(listener: &TcpListener) -> std::io::Result<TcpStream> {
    let deadline = Instant::now() + LOAD_TIMEOUT;
    loop {
        match listener.accept() {
            Ok((stream, _)) => return Ok(stream),
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
            {
                thread::yield_now();
            }
            Err(error) => return Err(error),
        }
    }
}

fn broken_pipe() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::BrokenPipe, "test receiver dropped")
}

fn timed_out() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::TimedOut, "test release not received")
}

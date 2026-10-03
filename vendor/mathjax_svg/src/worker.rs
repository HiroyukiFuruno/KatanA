use std::sync::{Mutex, mpsc};

use anyhow::Context as _;

use super::{Result, convert_to_svg_inner};

const WORKER_STACK_BYTES: usize = 12 * 1024 * 1024;
static WORKER: Mutex<Option<mpsc::SyncSender<RenderRequest>>> = Mutex::new(None);

struct RenderRequest {
    latex: String,
    display: bool,
    reply: mpsc::Sender<Result<String>>,
}

pub(super) fn convert(latex: &str, display: bool) -> Result<String> {
    let sender = worker_sender()?;
    let (reply, response) = mpsc::channel();
    sender
        .send(RenderRequest {
            latex: latex.to_owned(),
            display,
            reply,
        })
        .map_err(|_| anyhow::anyhow!("MathJax worker request channel disconnected"))?;
    response
        .recv()
        .context("MathJax worker response channel disconnected")?
}

fn worker_sender() -> Result<mpsc::SyncSender<RenderRequest>> {
    let mut worker = WORKER
        .lock()
        .map_err(|_| anyhow::anyhow!("MathJax worker initialization lock poisoned"))?;
    if let Some(sender) = worker.as_ref() {
        return Ok(sender.clone());
    }
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("katana-mathjax".to_owned())
        .stack_size(WORKER_STACK_BYTES)
        .spawn(move || run(receiver))
        .context("failed to spawn MathJax worker")?;
    *worker = Some(sender.clone());
    Ok(sender)
}

fn run(receiver: mpsc::Receiver<RenderRequest>) {
    let mut context = None;
    for request in receiver {
        let result = convert_to_svg_inner(&mut context, &request.latex, request.display);
        let _ = request.reply.send(result);
    }
}

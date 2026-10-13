use katana_core::markdown::DiagramResult;

use super::diagram_cache::DiagramRenderCacheCoordinator;
use super::types::{RenderJob, RenderMessage, RendererLogicOps};

fn dispatch_and_reduce(
    job: &RenderJob,
    tx: &std::sync::mpsc::Sender<RenderMessage>,
) -> DiagramResult {
    let block = katana_core::markdown::DiagramBlock {
        kind: job.kind.clone(),
        source: job.src.clone(),
    };
    let res = RendererLogicOps::dispatch_renderer(&block);
    if matches!(res, DiagramResult::Err { .. }) {
        let _ = tx.send(RenderMessage::ReduceConcurrency);
    }
    res
}

fn resolve_diagram_result(
    job: &RenderJob,
    tx: &std::sync::mpsc::Sender<RenderMessage>,
) -> DiagramResult {
    if !job.force
        && let Some(result) = DiagramRenderCacheCoordinator::cached_result(
            &job.cache,
            &job.document_path,
            &job.kind,
            &job.src,
        )
    {
        return result;
    }
    DiagramRenderCacheCoordinator::record_redraw(&job.kind, &job.document_path, &job.src);
    let result = dispatch_and_reduce(job, tx);
    DiagramRenderCacheCoordinator::store_result(
        &job.cache,
        &job.document_path,
        &job.kind,
        &job.src,
        &result,
    );
    result
}

pub(super) fn spawn_render_workers(
    jobs: Vec<RenderJob>,
    tx: std::sync::mpsc::Sender<RenderMessage>,
    current_cancel_token: std::sync::Arc<std::sync::atomic::AtomicBool>,
    repaint_ctx: Option<egui::Context>,
    concurrency: usize,
) {
    let jobs_len = jobs.len();
    let jobs_rx = std::sync::Arc::new(std::sync::Mutex::new(jobs.into_iter()));

    for _ in 0..concurrency.min(jobs_len) {
        let tx = tx.clone();
        let jobs_rx = jobs_rx.clone();
        let current_cancel_token = current_cancel_token.clone();
        let repaint_ctx = repaint_ctx.clone();
        std::thread::spawn(move || {
            loop {
                let job = {
                    let mut lock = jobs_rx.lock().unwrap();
                    lock.next()
                };
                let Some(job) = job else {
                    break;
                };
                if current_cancel_token.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                let result = resolve_diagram_result(&job, &tx);

                let section = RendererLogicOps::map_diagram_result(
                    &job.kind,
                    &job.src,
                    result,
                    job.source_lines,
                );
                let msg = RenderMessage::Section {
                    generation: job.generation,
                    ordinal: job.ordinal,
                    section,
                };
                if tx.send(msg).is_err() {
                    break;
                }
                if let Some(ctx) = &repaint_ctx {
                    ctx.request_repaint();
                }
            }
        });
    }
}

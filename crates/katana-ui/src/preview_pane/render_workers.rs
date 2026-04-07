use katana_core::markdown::DiagramResult;

use super::types::{RenderJob, RenderMessage, RendererLogicOps};

fn cache_if_serializable_inner(
    result: &DiagramResult,
    is_http: bool,
    key: &str,
    cache: &std::sync::Arc<dyn katana_platform::CacheFacade>,
) {
    let Ok(json) = serde_json::to_string(result) else {
        return;
    };
    if is_http {
        cache.set_memory(key, json);
    } else {
        let _ = cache.set_persistent(key, json);
    }
}

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

fn resolve_cached_result(
    cached_result: Option<String>,
    job: &RenderJob,
    is_http: bool,
    cache_key: &str,
    tx: &std::sync::mpsc::Sender<RenderMessage>,
) -> DiagramResult {
    let Some(json) = cached_result else {
        let res = dispatch_and_reduce(job, tx);
        cache_if_serializable_inner(&res, is_http, cache_key, &job.cache);
        return res;
    };
    match serde_json::from_str::<DiagramResult>(&json) {
        Ok(res)
            if matches!(job.kind, katana_core::markdown::DiagramKind::Mermaid)
                && matches!(res, DiagramResult::Ok(_)) =>
        {
            /* WHY: Old caches may still store Mermaid output as Ok(html) (SVG format). */
            /* WHY: or high CPU usage. We MUST bypass the bad cache and force a re-render to PNG. */
            let new_res = dispatch_and_reduce(job, tx);
            cache_if_serializable_inner(&new_res, is_http, cache_key, &job.cache);
            new_res
        }
        Ok(res) => res,
        Err(_) => dispatch_and_reduce(job, tx),
    }
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

                let cache_key = RendererLogicOps::get_cache_key(&job.path, &job.kind, &job.src);
                let is_http = job.src.contains("http://") || job.src.contains("https://");

                let cached_result: Option<String> = if !job.force {
                    if is_http {
                        job.cache.get_memory(&cache_key)
                    } else {
                        job.cache.get_persistent(&cache_key)
                    }
                } else {
                    None
                };

                let result = resolve_cached_result(cached_result, &job, is_http, &cache_key, &tx);

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

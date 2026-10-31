use super::{Coordinator, Inner, WatchCommand, WatchRequest};
use crate::preview_pane::image_perf::{ImagePerf, ImageRequestTrace};
use std::path::PathBuf;
use std::sync::Weak;

impl Coordinator {
    pub(in crate::preview_pane::local_image_loader) fn request(
        &self,
        owner: Weak<Inner>,
        path: PathBuf,
        generation: u64,
    ) -> Result<(), String> {
        if let Some(error) = &self.startup_error {
            return Err(error.clone());
        }
        let request = WatchRequest {
            owner,
            path,
            generation,
        };
        let perf = request
            .perf_trace()
            .map(|trace| (trace, request.path.clone()));
        ImagePerf::once(
            request.perf_trace(),
            "watch_queue_attempt",
            &request.path,
            None,
        );
        self.request_tx
            .try_send(WatchCommand::Watch(request))
            .map(|()| {
                if let Some((trace, path)) = perf {
                    ImagePerf::emit(
                        Some(trace),
                        "watch_queued",
                        &path,
                        format_args!("accepted=true"),
                    );
                }
            })
            .map_err(|error| format!("image watcher queue unavailable: {error}"))
    }

    pub(in crate::preview_pane::local_image_loader) fn clear(&self, owner: Weak<Inner>) {
        if let Err(error) = self.request_tx.try_send(WatchCommand::Clear(owner)) {
            tracing::warn!(%error, "local image watcher clear request dropped");
        }
    }
}

impl WatchRequest {
    pub(super) fn perf_trace(&self) -> Option<ImageRequestTrace> {
        if !ImagePerf::enabled() {
            return None;
        }
        let owner = self.owner.upgrade()?;
        let revision = owner
            .path_revisions
            .lock()
            .ok()?
            .get(&self.path)
            .copied()
            .unwrap_or(0);
        ImagePerf::request(self.owner.as_ptr() as usize, self.generation, revision)
    }

    pub(super) fn perf_event(&self, stage: &'static str, details: std::fmt::Arguments<'_>) {
        ImagePerf::emit(self.perf_trace(), stage, &self.path, details);
    }
}

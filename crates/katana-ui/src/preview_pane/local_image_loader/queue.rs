use super::{Arc, GlobalPool, LocalImageLoader, LocalImageStatus, RequestKey, Work, mpsc};
use crate::preview_pane::image_perf::ImagePerf;

impl LocalImageLoader {
    pub(super) fn enqueue(&self, key: RequestKey) -> LocalImageStatus {
        let Some(pool) = GlobalPool::get() else {
            return LocalImageStatus::Failed("image loader unavailable".to_owned());
        };
        let generation = self.generation();
        let revision = self.path_revision(&key.path);
        let Ok(mut pending) = self.inner.pending.lock() else {
            return LocalImageStatus::Failed("image loader unavailable".to_owned());
        };
        pending.insert(key.clone());
        drop(pending);
        let work = Work {
            owner: Arc::downgrade(&self.inner),
            result_tx: self.inner.result_tx.clone(),
            key: key.clone(),
            generation,
            revision,
        };
        self.enqueue_to(&pool.request_tx, &key, work)
    }

    pub(super) fn enqueue_to(
        &self,
        sender: &mpsc::SyncSender<Work>,
        key: &RequestKey,
        work: Work,
    ) -> LocalImageStatus {
        let trace =
            ImagePerf::request(work.owner.as_ptr() as usize, work.generation, work.revision);
        ImagePerf::once(trace, "decode_queue_attempt", &key.path, None);
        match sender.try_send(work) {
            Ok(()) => {
                ImagePerf::emit(
                    trace,
                    "decode_queued",
                    &key.path,
                    format_args!("accepted=true"),
                );
                LocalImageStatus::Pending
            }
            Err(mpsc::TrySendError::Full(_)) => {
                ImagePerf::once(trace, "decode_queue_full", &key.path, None);
                self.remove_pending(key);
                LocalImageStatus::Pending
            }
            Err(mpsc::TrySendError::Disconnected(_)) => {
                self.remove_pending(key);
                LocalImageStatus::Failed("image loader unavailable".to_owned())
            }
        }
    }
}

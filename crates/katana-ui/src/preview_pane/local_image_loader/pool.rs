use super::super::cancellable_read::ReadCancellation;
use super::{Inner, decode, egui};
use crate::preview_pane::image_perf::ImagePerf;
use std::path::Path;
use std::sync::{OnceLock, Weak, mpsc};

pub(super) struct Work {
    pub(super) owner: Weak<Inner>,
    pub(super) result_tx: mpsc::Sender<ResultMessage>,
    pub(super) key: super::RequestKey,
    pub(super) generation: u64,
    pub(super) revision: u64,
}

impl Work {
    fn is_cancelled(&self) -> bool {
        let Some(owner) = self.owner.upgrade() else {
            return true;
        };
        if owner.generation.load(std::sync::atomic::Ordering::Acquire) != self.generation {
            return true;
        }
        /* WHY: ファイル更新は世代を変えないため、同じ所有者の旧版読込も取り消す。 */
        owner.path_revisions.lock().map_or(true, |revisions| {
            revisions.get(&self.key.path).copied().unwrap_or(0) != self.revision
        })
    }

    fn publish_result(self, image: Result<egui::ColorImage, String>) {
        let owner = self.owner.clone();
        if self
            .result_tx
            .send(ResultMessage {
                key: self.key,
                generation: self.generation,
                revision: self.revision,
                image,
            })
            .is_ok()
        {
            Self::request_repaint(&owner);
        }
    }

    fn request_repaint(owner: &Weak<Inner>) {
        let Some(owner) = owner.upgrade() else {
            return;
        };
        let context = owner
            .repaint_context
            .lock()
            .ok()
            .and_then(|context| context.clone());
        if let Some(context) = context {
            /* WHY: poll_rendersが結果を回収する次のframeをdecode完了時に要求する。 */
            context.request_repaint();
        }
    }

    fn run(self) {
        let trace =
            ImagePerf::request(self.owner.as_ptr() as usize, self.generation, self.revision);
        ImagePerf::emit(trace, "decode_start", &self.key.path, format_args!(""));
        let cancellation = ReadCancellation::new(|| self.is_cancelled());
        if cancellation.check().is_err() {
            return;
        }
        let image = decode::ImageDecodeOps::load_image(
            Path::new(&self.key.path),
            self.key.background,
            &cancellation,
            trace,
        );
        ImagePerf::emit(
            trace,
            "decode_end",
            &self.key.path,
            format_args!("success={}", image.is_ok()),
        );
        if cancellation.check().is_err() {
            return;
        }
        self.publish_result(image);
    }
}

pub(super) struct ResultMessage {
    pub(super) key: super::RequestKey,
    pub(super) generation: u64,
    pub(super) revision: u64,
    pub(super) image: Result<egui::ColorImage, String>,
}

pub(super) struct GlobalPool {
    pub(super) request_tx: mpsc::SyncSender<Work>,
}

impl GlobalPool {
    pub(super) fn get() -> Option<&'static Self> {
        static POOL: OnceLock<Option<GlobalPool>> = OnceLock::new();
        POOL.get_or_init(|| {
            let (request_tx, request_rx) = mpsc::sync_channel::<Work>(super::REQUEST_CAPACITY);
            let worker = std::thread::Builder::new()
                .name("katana-local-image-loader".into())
                .spawn(move || {
                    while let Ok(work) = request_rx.recv() {
                        work.run();
                    }
                });
            worker.ok().map(|_| GlobalPool { request_tx })
        })
        .as_ref()
    }
}

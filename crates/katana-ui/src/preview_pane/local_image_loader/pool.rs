use super::super::cancellable_read::ReadCancellation;
use super::{Inner, decode, egui};
use std::path::Path;
use std::sync::{OnceLock, Weak, mpsc};

pub(super) struct Work {
    pub(super) owner: Weak<Inner>,
    pub(super) result_tx: mpsc::Sender<ResultMessage>,
    pub(super) key: super::RequestKey,
    pub(super) generation: u64,
    pub(super) revision: u64,
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
                        let Some(owner) = work.owner.upgrade() else {
                            continue;
                        };
                        if owner.generation.load(std::sync::atomic::Ordering::Acquire)
                            != work.generation
                        {
                            continue;
                        }
                        drop(owner);
                        let owner = work.owner.clone();
                        let generation = work.generation;
                        let cancellation = ReadCancellation::new(move || {
                            owner.upgrade().is_none_or(|owner| {
                                owner.generation.load(std::sync::atomic::Ordering::Acquire)
                                    != generation
                            })
                        });
                        if cancellation.check().is_err() {
                            continue;
                        }
                        let image = decode::ImageDecodeOps::load_image(
                            Path::new(&work.key.path),
                            work.key.background,
                            &cancellation,
                        );
                        if cancellation.check().is_err() {
                            continue;
                        }
                        let _ = work.result_tx.send(ResultMessage {
                            key: work.key,
                            generation: work.generation,
                            revision: work.revision,
                            image,
                        });
                    }
                });
            worker.ok().map(|_| GlobalPool { request_tx })
        })
        .as_ref()
    }
}

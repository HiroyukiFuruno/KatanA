use std::sync::{Arc, atomic::AtomicBool, mpsc::Receiver};

use crate::font_loader::office_faces::FontFaceResolution;

pub(super) struct FontLookupResult {
    pub(super) surface_generation: u64,
    pub(super) lookup_generation: u64,
    pub(super) resolution: FontFaceResolution,
}

pub(super) struct FontLookupJob {
    pub(super) cancelled: Arc<AtomicBool>,
    pub(super) results: Receiver<FontLookupResult>,
}

#[derive(Debug)]
pub(super) enum FontLookupFailure {
    RegistryUnavailable,
    WorkerSpawn(std::io::ErrorKind),
    WorkerDisconnected,
}

impl FontLookupFailure {
    pub(super) fn description(&self) -> String {
        match self {
            Self::RegistryUnavailable => "font registry unavailable".to_owned(),
            Self::WorkerSpawn(kind) => format!("font lookup worker spawn failed: {kind:?}"),
            Self::WorkerDisconnected => "font lookup worker disconnected".to_owned(),
        }
    }
}

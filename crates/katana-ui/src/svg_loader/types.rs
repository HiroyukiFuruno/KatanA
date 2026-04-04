use std::sync::{Arc, atomic::AtomicU64};

use egui::{ColorImage, load::SizeHint, mutex::Mutex};

pub(crate) const HEX_RADIX: u32 = 16;
pub(crate) const PERCENT_ENCODE_LEN: usize = 3;

pub(crate) struct Entry {
    pub(crate) last_used: AtomicU64,
    pub(crate) result: Result<Arc<ColorImage>, String>,
}

pub(crate) struct SvgCacheEntry {
    pub(crate) size_hint: SizeHint,
    pub(crate) data: Entry,
}

pub(crate) struct SvgCacheBucket {
    pub(crate) uri: String,
    pub(crate) entries: Vec<SvgCacheEntry>,
}

pub struct KatanaSvgLoader {
    pub(crate) pass_index: AtomicU64,
    pub(crate) cache: Mutex<Vec<SvgCacheBucket>>,
    pub(crate) options: resvg::usvg::Options<'static>,
}

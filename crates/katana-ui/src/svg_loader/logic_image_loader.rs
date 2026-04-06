use super::types::*;
use std::mem::size_of;
use std::sync::{Arc, atomic::AtomicU64, atomic::Ordering::Relaxed};

use egui::{
    Context,
    load::{BytesPoll, ImageLoadResult, ImageLoader, ImagePoll, LoadError, SizeHint},
};

impl ImageLoader for KatanaSvgLoader {
    fn id(&self) -> &str {
        Self::ID
    }

    fn load(&self, ctx: &Context, uri: &str, size_hint: SizeHint) -> ImageLoadResult {
        if !Self::is_supported(uri) {
            return Err(LoadError::NotSupported);
        }

        let mut cache = self.cache.lock();
        let bucket_idx = if let Some(idx) = cache.iter().position(|b| b.uri == uri) {
            idx
        } else {
            cache.push(SvgCacheBucket {
                uri: uri.to_owned(),
                entries: Vec::new(),
            });
            cache.len() - 1
        };
        let bucket = &mut cache[bucket_idx];

        if let Some(entry) = bucket
            .entries
            .iter()
            .find(|e| e.size_hint == size_hint)
            .map(|e| &e.data)
        {
            entry
                .last_used
                .store(self.pass_index.load(Relaxed), Relaxed);
            match entry.result.clone() {
                Ok(image) => Ok(ImagePoll::Ready { image }),
                Err(_) => Err(LoadError::NotSupported),
            }
        } else {
            let bytes_load_result = load_bytes_from_uri(ctx, uri);

            match bytes_load_result {
                Ok(BytesPoll::Ready { bytes, .. }) => {
                    let result = Self::preprocess_svg_bytes(&bytes)
                        .and_then(|svg| {
                            Self::rasterize_svg_bytes_with_size(
                                svg.as_bytes(),
                                size_hint,
                                &self.options,
                            )
                        })
                        .map(Arc::new);

                    bucket.entries.push(SvgCacheEntry {
                        size_hint,
                        data: Entry {
                            last_used: AtomicU64::new(self.pass_index.load(Relaxed)),
                            result: result.clone(),
                        },
                    });

                    match result {
                        Ok(image) => Ok(ImagePoll::Ready { image }),
                        Err(e) => {
                            tracing::warn!("SVG rasterization failed for {uri}: {e}");
                            Err(LoadError::NotSupported)
                        }
                    }
                }
                Ok(BytesPoll::Pending { size }) => Ok(ImagePoll::Pending { size }),
                Err(err) => {
                    bucket.entries.push(SvgCacheEntry {
                        size_hint,
                        data: Entry {
                            last_used: AtomicU64::new(self.pass_index.load(Relaxed)),
                            result: Err(err.to_string()),
                        },
                    });
                    Err(err)
                }
            }
        }
    }

    fn forget(&self, uri: &str) {
        self.cache.lock().retain(|bucket| bucket.uri != uri);
    }

    fn forget_all(&self) {
        self.cache.lock().clear();
    }

    fn byte_size(&self) -> usize {
        self.cache
            .lock()
            .iter()
            .flat_map(|bucket| bucket.entries.iter())
            .map(|entry| match &entry.data.result {
                Ok(image) => image.pixels.len() * size_of::<egui::Color32>(),
                Err(err) => err.len(),
            })
            .sum()
    }

    fn end_pass(&self, pass_index: u64) {
        self.pass_index.store(pass_index, Relaxed);
        let mut cache = self.cache.lock();
        cache.retain_mut(|bucket| {
            if 2 <= bucket.entries.len() {
                bucket
                    .entries
                    .retain(|entry| pass_index <= entry.data.last_used.load(Relaxed) + 1);
            }
            !bucket.entries.is_empty()
        });
    }
}

fn parse_data_uri_body(meta: &str, content: &str) -> Result<BytesPoll, egui::load::LoadError> {
    if meta.ends_with(";base64") {
        use base64::{Engine as _, engine::general_purpose};
        return match general_purpose::STANDARD.decode(content.trim()) {
            Ok(bytes) => Ok(BytesPoll::Ready {
                size: None,
                bytes: egui::load::Bytes::Shared(std::sync::Arc::from(bytes)),
                mime: None,
            }),
            Err(e) => Err(egui::load::LoadError::Loading(format!(
                "Base64 decode error: {}",
                e
            ))),
        };
    }
    let bytes = content.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(hex) = std::str::from_utf8(&bytes[i + 1..=i + 2])
            && let Ok(byte) = u8::from_str_radix(hex, HEX_RADIX)
        {
            decoded.push(byte);
            i += PERCENT_ENCODE_LEN;
            continue;
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    Ok(BytesPoll::Ready {
        size: None,
        bytes: egui::load::Bytes::Shared(std::sync::Arc::from(
            String::from_utf8_lossy(&decoded).into_owned().into_bytes(),
        )),
        mime: None,
    })
}

fn load_bytes_from_uri(ctx: &Context, uri: &str) -> Result<BytesPoll, egui::load::LoadError> {
    let Some(data) = uri.strip_prefix("data:") else {
        return ctx.try_load_bytes(uri);
    };
    let Some((meta, content)) = data.split_once(',') else {
        return Err(egui::load::LoadError::Loading(
            "Invalid data URI format".into(),
        ));
    };
    parse_data_uri_body(meta, content)
}

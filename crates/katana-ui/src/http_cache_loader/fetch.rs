use std::{path::Path, task::Poll};

use egui::load::{Bytes, BytesLoadResult, BytesPoll, LoadError};

use super::types::{CachedFile, Entry};

pub(crate) const HTTP_PROTOCOL: &str = "http://";
pub(crate) const HTTPS_PROTOCOL: &str = "https://";

pub(crate) struct HttpCacheFetchOps;

impl HttpCacheFetchOps {
    pub(crate) fn is_http_uri(uri: &str) -> bool {
        uri.starts_with(HTTP_PROTOCOL) || uri.starts_with(HTTPS_PROTOCOL)
    }

    pub(crate) fn entry_to_bytes_result(entry: Entry) -> BytesLoadResult {
        match entry {
            Poll::Ready(Ok(file)) => Ok(BytesPoll::Ready {
                size: None,
                bytes: Bytes::Shared(file.bytes),
                mime: file.mime,
            }),
            Poll::Ready(Err(err)) => Err(LoadError::Loading(err)),
            Poll::Pending => Ok(BytesPoll::Pending { size: None }),
        }
    }

    pub(crate) fn process_fetch_response(
        uri: &str,
        cache_dir: &Path,
        response: ehttp::Result<ehttp::Response>,
    ) -> Result<CachedFile, String> {
        match response {
            Ok(response) => CachedFile::from_response(uri, response).inspect(|file| {
                if let Err(err) =
                    super::disk::HttpCacheDiskOps::write_cached_file_for_uri(cache_dir, uri, file)
                {
                    tracing::warn!("Failed to persist HTTP image cache for {uri}: {err}");
                }
            }),
            Err(err) => {
                tracing::error!("Failed to load {uri:?}: {err}");
                Err(format!("Failed to load {uri:?}"))
            }
        }
    }
}

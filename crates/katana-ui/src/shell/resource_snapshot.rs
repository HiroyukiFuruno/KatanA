use super::KatanaApp;
use crate::preview_pane::{DocumentWorkerLifecycle, HtmlReceiptObservation};
use katana_document_viewer::{DocumentResourceSnapshot, DocumentSession};
use serde::Serialize;
use std::time::Instant;

#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static CAPTURE_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub(crate) struct HostResourceCounts {
    pub(crate) previews: usize,
    pub(crate) html_surfaces: usize,
    pub(crate) document_surfaces: usize,
    pub(crate) frames: usize,
    pub(crate) textures: usize,
    pub(crate) cache_entries: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct DocumentSurfaceIdentity {
    pub(crate) uri: String,
    pub(crate) generation: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct HtmlSurfaceReceiptObservation {
    pub(crate) source_uri: String,
    pub(crate) katana_adapter_instance_id: u64,
    pub(crate) katana_renderer_session_epoch: u64,
    pub(crate) frame_generation: u64,
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
    pub(crate) scroll_y: f32,
    pub(crate) content_height: f32,
    pub(crate) idle: bool,
}

impl From<HtmlReceiptObservation> for HtmlSurfaceReceiptObservation {
    fn from(value: HtmlReceiptObservation) -> Self {
        Self {
            source_uri: value.source_uri,
            katana_adapter_instance_id: value.katana_adapter_instance_id,
            katana_renderer_session_epoch: value.katana_renderer_session_epoch,
            frame_generation: value.frame_generation,
            viewport_width: value.viewport_width,
            viewport_height: value.viewport_height,
            scroll_y: value.scroll_y,
            content_height: value.content_height,
            idle: value.idle,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub(crate) struct UiResourceSnapshot {
    pub(crate) host: HostResourceCounts,
    pub(crate) document_workers_live: usize,
    pub(crate) kdv: DocumentResourceSnapshot,
    pub(crate) document_surfaces: Vec<DocumentSurfaceIdentity>,
    pub(crate) html_observations: Vec<HtmlSurfaceReceiptObservation>,
}

impl UiResourceSnapshot {
    pub(crate) fn capture(app: &KatanaApp) -> (Self, u64) {
        #[cfg(test)]
        CAPTURE_COUNT.with(|count| count.set(count.get().saturating_add(1)));
        let started = Instant::now();
        let mut snapshot = Self {
            host: HostResourceCounts {
                previews: app.tab_previews.len(),
                ..HostResourceCounts::default()
            },
            document_workers_live: DocumentWorkerLifecycle::live_count(),
            kdv: DocumentSession::resource_snapshot(),
            document_surfaces: Vec::new(),
            html_observations: Vec::new(),
        };
        for preview in &app.tab_previews {
            let (html_surfaces, html_frames, html_textures) =
                preview.pane.html_browser_resource_counts();
            let (document_surfaces, document_frames, document_textures) =
                preview.pane.document_resource_counts();
            snapshot.host.html_surfaces += html_surfaces;
            snapshot.host.document_surfaces += document_surfaces;
            snapshot.host.frames += html_frames + document_frames;
            snapshot.host.textures += html_textures + document_textures;
            snapshot.host.cache_entries += preview.pane.sections.len()
                + preview.pane.image_cache.len()
                + preview.pane.viewer_states.len();
            if let Some((uri, generation)) = preview.pane.document_resource_identity() {
                snapshot
                    .document_surfaces
                    .push(DocumentSurfaceIdentity { uri, generation });
            }
        }
        (
            snapshot,
            started.elapsed().as_micros().try_into().unwrap_or(u64::MAX),
        )
    }

    pub(crate) fn capture_for_receipt(app: &KatanaApp) -> Result<(Self, u64), String> {
        let started = Instant::now();
        let (mut snapshot, _) = Self::capture(app);
        for preview in &app.tab_previews {
            if let Some(observation) = preview.pane.html_browser_receipt_observation()? {
                snapshot.html_observations.push(observation.into());
            }
        }
        Ok((
            snapshot,
            started.elapsed().as_micros().try_into().unwrap_or(u64::MAX),
        ))
    }

    pub(crate) fn preview_counts_tuple(&self) -> (usize, usize, usize, usize, usize, usize) {
        let host = &self.host;
        (
            host.previews,
            host.html_surfaces,
            host.document_surfaces,
            host.frames,
            host.textures,
            host.cache_entries,
        )
    }

    pub(crate) fn document_lifecycle_tuple(&self) -> (usize, DocumentResourceSnapshot) {
        (self.document_workers_live, self.kdv)
    }
}

impl KatanaApp {
    pub(crate) fn record_completed_frame_resources(&mut self) {
        let should_capture = self
            .resource_receipt
            .as_mut()
            .is_some_and(super::resource_receipt::ResourceReceiptState::begin_capture);
        if !should_capture {
            return;
        }
        match UiResourceSnapshot::capture_for_receipt(self) {
            Ok((snapshot, elapsed_us)) => {
                if let Some(receipt) = &mut self.resource_receipt {
                    receipt.record(snapshot, elapsed_us);
                }
            }
            Err(error) => {
                if let Some(receipt) = &mut self.resource_receipt {
                    receipt.record_capture_error(error);
                }
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn completed_frame_capture_count_for_test() -> usize {
    CAPTURE_COUNT.with(Cell::get)
}

#[cfg(test)]
pub(crate) fn reset_completed_frame_capture_count_for_test() {
    CAPTURE_COUNT.with(|count| count.set(0));
}

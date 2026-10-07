use crate::preview_pane::types::{ViewerState, ViewerTextureIdentity, ViewerTextureSource};
use eframe::egui;
use katana_core::markdown::svg_rasterize::RasterizedSvg;
use std::path::Path;

impl std::fmt::Debug for ViewerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewerState")
            .field("zoom", &self.zoom)
            .field("pan", &self.pan)
            .field("texture", &self.texture.as_ref().map(|t| t.id()))
            .field("texture_background", &self.texture_background)
            .field("texture_identity", &self.texture_identity)
            .field("closing_since", &self.closing_since)
            .finish()
    }
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            texture: None,
            texture_background: None,
            texture_identity: None,
            closing_since: None,
        }
    }
}

impl ViewerTextureIdentity {
    pub fn rasterized(image: &RasterizedSvg) -> Self {
        Self {
            source: ViewerTextureSource::Rasterized,
            width: image.width,
            height: image.height,
            display_width_bits: image.display_width.to_bits(),
            display_height_bits: image.display_height.to_bits(),
            content_hash: image.content_hash,
        }
    }

    pub fn local_file(path: &Path) -> Self {
        Self {
            source: ViewerTextureSource::LocalFile,
            width: 0,
            height: 0,
            display_width_bits: 0,
            display_height_bits: 0,
            content_hash: stable_path_hash(path),
        }
    }

    pub(crate) fn local_file_revision(path: &Path, revision: u64) -> Self {
        let mut identity = Self::local_file(path);
        if revision != 0 {
            identity.content_hash = stable_hash_u64(identity.content_hash, revision);
        }
        identity
    }
}

impl ViewerState {
    pub(crate) fn prepare_texture(
        &mut self,
        identity: ViewerTextureIdentity,
        background: egui::Color32,
    ) {
        if self.texture_identity != Some(identity) {
            self.zoom = 1.0;
            self.pan = egui::Vec2::ZERO;
            self.texture = None;
            self.texture_background = None;
            self.texture_identity = Some(identity);
            return;
        }
        if self.texture_background != Some(background) {
            self.texture = None;
            self.texture_background = None;
        }
    }

    pub fn zoom_in(&mut self) {
        const VIEWER_ZOOM_STEP: f32 = 0.25;
        const VIEWER_ZOOM_MAX: f32 = 4.0;
        self.zoom = (self.zoom + VIEWER_ZOOM_STEP).min(VIEWER_ZOOM_MAX);
    }

    pub fn zoom_out(&mut self) {
        const VIEWER_ZOOM_STEP: f32 = 0.25;
        const VIEWER_ZOOM_MIN: f32 = 0.25;
        self.zoom = (self.zoom - VIEWER_ZOOM_STEP).max(VIEWER_ZOOM_MIN);
    }

    pub fn pan_by(&mut self, delta: egui::Vec2) {
        self.pan += delta;
    }

    pub fn pan_up(&mut self) {
        const VIEWER_PAN_STEP: f32 = 50.0;
        self.pan_by(egui::vec2(0.0, -VIEWER_PAN_STEP));
    }

    pub fn pan_down(&mut self) {
        const VIEWER_PAN_STEP: f32 = 50.0;
        self.pan_by(egui::vec2(0.0, VIEWER_PAN_STEP));
    }

    pub fn pan_left(&mut self) {
        const VIEWER_PAN_STEP: f32 = 50.0;
        self.pan_by(egui::vec2(-VIEWER_PAN_STEP, 0.0));
    }

    pub fn pan_right(&mut self) {
        const VIEWER_PAN_STEP: f32 = 50.0;
        self.pan_by(egui::vec2(VIEWER_PAN_STEP, 0.0));
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

fn stable_path_hash(path: &Path) -> u64 {
    let mut hash = stable_hash_bytes(path.to_string_lossy().as_bytes());
    hash = stable_hash_u64(hash, 0);
    stable_hash_u64(hash, 0)
}

fn stable_hash_u64(hash: u64, value: u64) -> u64 {
    value.to_le_bytes().iter().fold(hash, |acc, byte| {
        (acc ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    })
}

fn stable_hash_bytes(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    })
}

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

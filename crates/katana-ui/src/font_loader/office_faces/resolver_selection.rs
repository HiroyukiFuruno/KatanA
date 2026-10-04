use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use skrifa::FontRef;

use super::face_payload::FacePayload;
use super::font_metadata::{FaceMetadata, face_metadata, matches_request, weight_distance};
use super::types::{FontFaceRequest, FontFaceResolutionDiagnostic, ResolvedFontFace};

pub(super) type RequestKey = (String, bool, bool);

pub(super) struct SelectedFace {
    metadata: FaceMetadata,
    path: PathBuf,
    index: u32,
    distance: u16,
    payload: Arc<egui::FontData>,
}

pub(super) fn all_exact_matches(
    selected: &BTreeMap<RequestKey, SelectedFace>,
    requests: &BTreeMap<RequestKey, FontFaceRequest>,
) -> bool {
    selected.len() == requests.len() && selected.values().all(|face| face.distance == 0)
}

impl SelectedFace {
    pub(super) fn into_resolved(self, request: FontFaceRequest) -> ResolvedFontFace {
        let payload_digest = ResolvedFontFace::digest_payload(&self.payload);
        ResolvedFontFace {
            request,
            family: self.metadata.family,
            weight: self.metadata.weight,
            bold: self.metadata.bold,
            italic: self.metadata.italic,
            monospaced: self.metadata.monospaced,
            path: self.path,
            face_index: self.index,
            payload: self.payload,
            payload_digest,
        }
    }
}

pub(super) struct PendingMatch {
    key: RequestKey,
    metadata: FaceMetadata,
    index: u32,
    distance: u16,
}

pub(super) fn selected_match(
    face: &FontRef<'_>,
    index: usize,
    requests: &BTreeMap<RequestKey, FontFaceRequest>,
) -> Vec<PendingMatch> {
    let Ok(index) = u32::try_from(index) else {
        return Vec::new();
    };
    let Some(metadata) = face_metadata(face) else {
        return Vec::new();
    };
    requests
        .iter()
        .filter_map(|(key, request)| {
            if !matches_request(&metadata, &request.family, request.bold, request.italic) {
                return None;
            }
            Some(PendingMatch {
                key: key.clone(),
                distance: weight_distance(&metadata, request.bold),
                metadata: metadata.clone(),
                index,
            })
        })
        .collect()
}

pub(super) fn append_matches(
    path: &Path,
    matches: Vec<PendingMatch>,
    payload: Vec<u8>,
    selected: &mut BTreeMap<RequestKey, SelectedFace>,
) -> Vec<FontFaceResolutionDiagnostic> {
    let mut payload = FacePayload::new(payload);
    let mut shared_payloads = BTreeMap::<u32, Arc<egui::FontData>>::new();
    let mut diagnostics = Vec::new();
    for found in matches {
        if !should_replace(selected.get(&found.key), found.distance) {
            continue;
        }
        let shared = if let Some(existing) = shared_payloads.get(&found.index) {
            existing.clone()
        } else {
            let Ok(font_data) = payload.take_face(found.index) else {
                diagnostics.push(FontFaceResolutionDiagnostic::InvalidFontFace {
                    path: path.to_owned(),
                    face_index: found.index as usize,
                });
                continue;
            };
            let shared = Arc::new(font_data);
            shared_payloads.insert(found.index, shared.clone());
            shared
        };
        insert_match(path, found, shared, selected);
    }
    diagnostics
}

fn insert_match(
    path: &Path,
    found: PendingMatch,
    payload: Arc<egui::FontData>,
    selected: &mut BTreeMap<RequestKey, SelectedFace>,
) {
    selected.insert(
        found.key,
        SelectedFace {
            metadata: found.metadata,
            path: path.to_owned(),
            index: found.index,
            distance: found.distance,
            payload,
        },
    );
}

fn should_replace(current: Option<&SelectedFace>, distance: u16) -> bool {
    current.is_none_or(|selected| distance < selected.distance)
}

pub(super) fn request_key(request: &FontFaceRequest) -> RequestKey {
    (request.family.to_lowercase(), request.bold, request.italic)
}

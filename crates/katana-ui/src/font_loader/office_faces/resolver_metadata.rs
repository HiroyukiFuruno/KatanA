use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use skrifa::FontRef;

use super::font_file::FontCandidate;
use super::metadata_reader::FontMetadataReader;
use super::resolver_debug::selected_match;
use super::resolver_selection::{RequestKey, SelectedFace};
use super::types::{FontFaceRequest, FontFaceResolution, FontFaceResolutionDiagnostic};

pub(super) struct CandidateMetadata;

impl CandidateMetadata {
    pub(super) fn payload(
        path: &std::path::Path,
        requests: &BTreeMap<RequestKey, FontFaceRequest>,
        selected: &BTreeMap<RequestKey, SelectedFace>,
        result: &mut FontFaceResolution,
        incomplete: &mut bool,
        cancelled: &AtomicBool,
    ) -> Option<Vec<u8>> {
        let mut candidate = match FontCandidate::open(path, cancelled) {
            Ok(candidate) => candidate,
            Err(error) => return Self::failure(error, result, incomplete),
        };
        let (needed, diagnostics) = Self::inspect(&mut candidate, requests, selected, cancelled);
        if !needed {
            *incomplete |= !diagnostics.is_empty();
            result.diagnostics.extend(diagnostics);
            return None;
        }
        match super::resolver_debug::read_candidate(&mut candidate, cancelled) {
            Ok(bytes) => Some(bytes),
            Err(error) => Self::failure(error, result, incomplete),
        }
    }

    fn failure(
        error: FontFaceResolutionDiagnostic,
        result: &mut FontFaceResolution,
        incomplete: &mut bool,
    ) -> Option<Vec<u8>> {
        *incomplete = true;
        result.diagnostics.push(error);
        None
    }

    pub(super) fn inspect(
        candidate: &mut FontCandidate,
        requests: &BTreeMap<RequestKey, FontFaceRequest>,
        selected: &BTreeMap<RequestKey, SelectedFace>,
        cancelled: &AtomicBool,
    ) -> (bool, Vec<FontFaceResolutionDiagnostic>) {
        let path = candidate.path().to_owned();
        let mut reader =
            FontMetadataReader::new(&mut candidate.file, &path, candidate.size, cancelled);
        let offsets = match reader.offsets() {
            Ok(offsets) => offsets,
            Err(error) => return (false, vec![error]),
        };
        let mut diagnostics = Vec::new();
        for (index, offset) in offsets.into_iter().enumerate() {
            match reader.face(offset, index) {
                Ok(bytes) if Self::improves(&bytes, index, &path, requests, selected) => {
                    return (true, Vec::new());
                }
                Ok(_) => {}
                Err(error) => {
                    if Self::record_error(error, &mut diagnostics) {
                        break;
                    }
                }
            }
            if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                break;
            }
        }
        (false, diagnostics)
    }

    fn record_error(
        error: FontFaceResolutionDiagnostic,
        diagnostics: &mut Vec<FontFaceResolutionDiagnostic>,
    ) -> bool {
        let unreadable = matches!(error, FontFaceResolutionDiagnostic::FileUnreadable { .. });
        diagnostics.push(error);
        unreadable
    }

    fn improves(
        bytes: &[u8],
        index: usize,
        path: &std::path::Path,
        requests: &BTreeMap<RequestKey, FontFaceRequest>,
        selected: &BTreeMap<RequestKey, SelectedFace>,
    ) -> bool {
        let Ok(face) = FontRef::new(bytes) else {
            return false;
        };
        selected_match(&face, index, path, requests)
            .iter()
            .any(|found| found.improves(selected))
    }
}

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use skrifa::raw::FileRef;

use super::resolver_debug::{read_candidate, selected_match};
use super::resolver_selection::{
    PendingMatch, RequestKey, SelectedFace, all_exact_matches, append_matches,
};
use super::types::{FontFaceRequest, FontFaceResolution, FontFaceResolutionDiagnostic};

pub(super) fn scan_candidates(
    candidates: &[(String, String)],
    requests: &BTreeMap<RequestKey, FontFaceRequest>,
    selected: &mut BTreeMap<RequestKey, SelectedFace>,
    result: &mut FontFaceResolution,
    incomplete: &mut bool,
    cancelled: &AtomicBool,
) {
    let mut seen = BTreeSet::new();
    for (_, value) in candidates {
        if stop_if_cancelled(cancelled, result) || all_exact_matches(selected, requests) {
            break;
        }
        inspect_new_path(
            Path::new(value),
            &mut seen,
            requests,
            selected,
            result,
            incomplete,
            cancelled,
        );
    }
}

fn inspect_new_path(
    path: &Path,
    seen: &mut BTreeSet<PathBuf>,
    requests: &BTreeMap<RequestKey, FontFaceRequest>,
    selected: &mut BTreeMap<RequestKey, SelectedFace>,
    result: &mut FontFaceResolution,
    incomplete: &mut bool,
    cancelled: &AtomicBool,
) {
    if seen.insert(path.to_owned()) {
        inspect_candidate(path, requests, selected, result, incomplete, cancelled);
    }
}

fn inspect_candidate(
    path: &Path,
    requests: &BTreeMap<RequestKey, FontFaceRequest>,
    selected: &mut BTreeMap<RequestKey, SelectedFace>,
    result: &mut FontFaceResolution,
    incomplete: &mut bool,
    cancelled: &AtomicBool,
) {
    let payload = match read_candidate(path, cancelled) {
        Ok(bytes) => Some(bytes),
        Err(diagnostic) => return_diagnostic(diagnostic, result, incomplete),
    };
    let Some(payload) = payload else { return };
    inspect_payload(
        path, payload, requests, selected, result, incomplete, cancelled,
    );
}

fn return_diagnostic(
    diagnostic: FontFaceResolutionDiagnostic,
    result: &mut FontFaceResolution,
    incomplete: &mut bool,
) -> Option<Vec<u8>> {
    *incomplete = true;
    result.diagnostics.push(diagnostic);
    None
}

fn inspect_payload(
    path: &Path,
    payload: Vec<u8>,
    requests: &BTreeMap<RequestKey, FontFaceRequest>,
    selected: &mut BTreeMap<RequestKey, SelectedFace>,
    result: &mut FontFaceResolution,
    incomplete: &mut bool,
    cancelled: &AtomicBool,
) {
    let matches = {
        let Some(file) = font_file(path, &payload, result, incomplete) else {
            return;
        };
        collect_faces(file, path, requests, result, incomplete, cancelled)
    };
    append_matches(path, matches, payload, selected);
}

fn font_file<'a>(
    path: &Path,
    payload: &'a [u8],
    result: &mut FontFaceResolution,
    incomplete: &mut bool,
) -> Option<FileRef<'a>> {
    match FileRef::new(payload) {
        Ok(file) => Some(file),
        Err(_) => {
            *incomplete = true;
            result
                .diagnostics
                .push(FontFaceResolutionDiagnostic::InvalidFontFile {
                    path: path.to_owned(),
                });
            None
        }
    }
}

fn collect_faces(
    file: FileRef<'_>,
    path: &Path,
    requests: &BTreeMap<RequestKey, FontFaceRequest>,
    result: &mut FontFaceResolution,
    incomplete: &mut bool,
    cancelled: &AtomicBool,
) -> Vec<PendingMatch> {
    let mut collection = FaceCollection {
        path,
        requests,
        matches: Vec::new(),
        result,
        incomplete,
    };
    for (index, parsed) in file.fonts().enumerate() {
        if stop_if_cancelled(cancelled, collection.result) {
            break;
        }
        collect_face(parsed, index, &mut collection);
    }
    collection.matches
}

struct FaceCollection<'a> {
    path: &'a Path,
    requests: &'a BTreeMap<RequestKey, FontFaceRequest>,
    matches: Vec<PendingMatch>,
    result: &'a mut FontFaceResolution,
    incomplete: &'a mut bool,
}

fn collect_face(
    parsed: Result<skrifa::FontRef<'_>, skrifa::raw::ReadError>,
    index: usize,
    collection: &mut FaceCollection<'_>,
) {
    match parsed {
        Ok(face) => collection.matches.extend(selected_match(
            &face,
            index,
            collection.path,
            collection.requests,
        )),
        Err(_) => {
            *collection.incomplete = true;
            collection
                .result
                .diagnostics
                .push(FontFaceResolutionDiagnostic::InvalidFontFace {
                    path: collection.path.to_owned(),
                    face_index: index,
                });
        }
    }
}

fn stop_if_cancelled(cancelled: &AtomicBool, result: &mut FontFaceResolution) -> bool {
    if !cancelled.load(Ordering::Acquire) {
        return false;
    }
    if !result
        .diagnostics
        .iter()
        .any(|item| matches!(item, FontFaceResolutionDiagnostic::Cancelled))
    {
        result
            .diagnostics
            .push(FontFaceResolutionDiagnostic::Cancelled);
    }
    true
}

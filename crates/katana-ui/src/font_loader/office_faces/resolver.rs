use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use super::resolver_scan::scan_candidates;
use super::resolver_selection::{RequestKey, SelectedFace, request_key};
use super::types::{FontFaceRequest, FontFaceResolution, FontFaceResolutionDiagnostic};

pub(crate) struct FontFaceResolver;

impl FontFaceResolver {
    pub(crate) fn resolve(
        candidates: &[(String, String)],
        requests: &[FontFaceRequest],
        cancelled: &AtomicBool,
    ) -> FontFaceResolution {
        let mut result = FontFaceResolution::default();
        let pending = unique_requests(requests);
        if pending.is_empty() {
            return result;
        }
        let mut selected = BTreeMap::<RequestKey, SelectedFace>::new();
        let mut incomplete = false;
        scan_candidates(
            candidates,
            &pending,
            &mut selected,
            &mut result,
            &mut incomplete,
            cancelled,
        );
        finish_resolution(pending, selected, &mut result, incomplete);
        result
    }
}

fn finish_resolution(
    pending: BTreeMap<RequestKey, FontFaceRequest>,
    mut selected: BTreeMap<RequestKey, SelectedFace>,
    result: &mut FontFaceResolution,
    incomplete: bool,
) {
    let cancelled = result
        .diagnostics
        .iter()
        .any(|item| matches!(item, FontFaceResolutionDiagnostic::Cancelled));
    if cancelled {
        return;
    }
    for (key, request) in pending {
        if let Some(face) = selected.remove(&key) {
            result.faces.push(face.into_resolved(request));
        } else {
            result
                .diagnostics
                .push(FontFaceResolutionDiagnostic::RequestedFaceUnavailable {
                    request,
                    search_incomplete: incomplete,
                });
        }
    }
    result.faces.sort_by_key(|face| request_key(&face.request));
}

fn unique_requests(requests: &[FontFaceRequest]) -> BTreeMap<RequestKey, FontFaceRequest> {
    requests
        .iter()
        .map(|request| (request_key(request), request.clone()))
        .collect()
}

use crate::shell::KatanaApp;
use egui_kittest::Harness;

pub(super) struct TimeoutDiagnostics;

impl TimeoutDiagnostics {
    pub(super) fn capture(harness: &mut Harness<'_, KatanaApp>) -> String {
        /* WHY: assert失敗時だけowned snapshotを作り、正常pollの時間と状態を変えない。 */
        let app = harness.state_mut();
        let active_path = app.state.active_path();
        let frame = app.document_frame_for_test();
        let idle = app.document_is_idle_for_test();
        let preview_counts = app.preview_resource_counts_for_test();
        let (workers, kdv) = app.document_lifecycle_resources_for_test();
        format!(
            "active_path={active_path:?} frame={frame:?} idle={idle:?} preview_counts={preview_counts:?} workers={workers} kdv={kdv:?}"
        )
    }
}

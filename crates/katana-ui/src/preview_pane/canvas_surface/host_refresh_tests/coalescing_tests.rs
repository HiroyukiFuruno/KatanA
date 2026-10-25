use super::revision_tests::RevisionCase;
use super::{ActionOps, CanvasRefreshTest};
use crate::preview_pane::canvas_request::wait_until_completed;
use eframe::egui;

const ORIGINAL_TEXT: &str = "original coalesced Canvas";
const UPDATED_TEXT: &str = "updated coalesced Canvas";
const INITIAL_WIDTH: i64 = 120;
const UPDATED_X: i64 = 700;
const UPDATED_WIDTH: i64 = 320;

struct CoalescingCase {
    _directory: tempfile::TempDir,
    path: std::path::PathBuf,
    context: egui::Context,
    app: crate::shell::KatanaApp,
}

impl CoalescingCase {
    fn new() -> Self {
        let directory = tempfile::tempdir().expect("Canvas directory");
        let path = directory.path().join("graph.canvas");
        let context = egui::Context::default();
        let source = RevisionCase::source(ORIGINAL_TEXT, 0, INITIAL_WIDTH);
        let app = RevisionCase::app(&path, &source, &context);
        Self {
            _directory: directory,
            path,
            context,
            app,
        }
    }

    fn refresh(&mut self, manual: bool) {
        self.app.process_action(
            &self.context,
            crate::app_state::AppAction::RefreshDocument { is_manual: manual },
        );
    }

    fn wait_completed(&self) {
        let request = self.app.tab_previews[0]
            .pane
            .canvas_request
            .as_ref()
            .expect("pending request");
        wait_until_completed(request);
    }

    fn write_updated(&self) {
        std::fs::write(
            &self.path,
            RevisionCase::source(UPDATED_TEXT, UPDATED_X, UPDATED_WIDTH),
        )
        .expect("updated Canvas bytes");
    }

    fn finish(&mut self) {
        CanvasRefreshTest::finish(&mut self.app, &self.context);
    }

    fn finish_current_request(&mut self) {
        let generation = self.generation();
        let deadline = std::time::Instant::now() + super::PARSE_TIMEOUT;
        loop {
            let pane = &mut self.app.tab_previews[0].pane;
            pane.poll_canvas_request(&self.context);
            if pane.canvas_request.is_none() || pane.session_generation != generation {
                return;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "current Canvas result was not delivered"
            );
            std::thread::yield_now();
        }
    }

    fn generation(&self) -> u64 {
        self.app.tab_previews[0].pane.session_generation
    }

    fn assert_updated(&mut self) {
        RevisionCase::assert_updated_document(
            &self.app,
            UPDATED_TEXT,
            (UPDATED_X as f64, UPDATED_WIDTH as f64),
        );
        RevisionCase::assert_updated_frame(
            &mut self.app,
            &self.context,
            UPDATED_TEXT,
            ORIGINAL_TEXT,
        );
    }
}

#[test]
fn automatic_refresh_after_unpolled_completion_reads_latest_file() {
    let mut case = CoalescingCase::new();
    case.refresh(false);
    case.wait_completed();
    let generation = case.generation();
    case.write_updated();
    case.refresh(false);
    case.refresh(false);
    assert_eq!(
        case.generation(),
        generation,
        "coalesce without cancelling the current request"
    );
    case.finish();
    case.assert_updated();
    assert_eq!(case.generation(), generation + 1, "one follow-up read");
}

#[test]
fn failed_unpolled_read_still_runs_the_reserved_valid_refresh() {
    let mut case = CoalescingCase::new();
    std::fs::write(&case.path, "{invalid").expect("invalid Canvas bytes");
    case.refresh(false);
    case.wait_completed();
    case.write_updated();
    case.refresh(false);
    case.finish_current_request();
    RevisionCase::assert_failed(&mut case.app, &case.context);
    case.finish();
    case.assert_updated();
}

#[test]
fn manual_force_discards_the_automatic_refresh_reservation() {
    let mut case = CoalescingCase::new();
    case.refresh(false);
    case.wait_completed();
    case.write_updated();
    case.refresh(false);
    case.refresh(true);
    let forced_generation = case.generation();
    case.finish();
    case.assert_updated();
    assert_eq!(
        case.generation(),
        forced_generation,
        "no stale follow-up after manual force"
    );
}

#[test]
fn canvas_path_switch_discards_the_previous_refresh_reservation() {
    let mut case = CoalescingCase::new();
    case.refresh(false);
    case.wait_completed();
    case.refresh(false);
    let next = case.path.with_file_name("next.canvas");
    std::fs::write(
        &next,
        RevisionCase::source(UPDATED_TEXT, UPDATED_X, UPDATED_WIDTH),
    )
    .expect("next Canvas file");
    case.app.tab_previews[0]
        .pane
        .update_canvas_path(&next, false);
    let switched_generation = case.generation();
    case.finish();
    case.assert_updated();
    assert_eq!(
        case.generation(),
        switched_generation,
        "no stale follow-up after path switch"
    );
    assert_eq!(case.app.tab_previews[0].pane.md_file_path, next);
}

#[test]
fn unchanged_reserved_refresh_retains_the_actual_panned_view() {
    let mut case = CoalescingCase::new();
    let panned = CanvasRefreshTest::pan(&mut case.app, &case.context);
    case.refresh(false);
    case.wait_completed();
    case.refresh(false);
    case.finish();
    assert_eq!(
        CanvasRefreshTest::draw(&mut case.app, &case.context),
        panned
    );
    RevisionCase::assert_updated_frame(&mut case.app, &case.context, ORIGINAL_TEXT, UPDATED_TEXT);
}

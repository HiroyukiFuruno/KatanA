use crate::app::ActionOps;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use eframe::egui;
use std::time::{Duration, Instant};

const BROWSER_UPDATE_TIMEOUT: Duration = Duration::from_secs(10);
const HTML_BROWSER_WIDTH: u32 = 1024;
const HTML_BROWSER_HEIGHT: u32 = 768;
const RED_RGB: [u8; 3] = [u8::MAX, 0, 0];
const GREEN_RGB: [u8; 3] = [0, u8::MAX, 0];

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn active_preview_session_generation(app: &KatanaApp) -> TestResult<u64> {
    let active_path = app
        .state
        .active_path()
        .ok_or("active document is missing")?;
    app.tab_previews
        .iter()
        .find(|preview| preview.path == active_path)
        .map(|preview| preview.pane.session_generation)
        .ok_or_else(|| "active preview is missing".into())
}

fn start_pending_browser_sessions(app: &mut KatanaApp) -> TestResult {
    let viewport = katana_document_viewer::browser_session::HtmlBrowserViewport::new(
        HTML_BROWSER_WIDTH,
        HTML_BROWSER_HEIGHT,
        1.0,
    )?;
    for preview in &mut app.tab_previews {
        preview.pane.start_html_browser_for_test(viewport);
    }
    Ok(())
}

fn wait_for_preview_reload(
    app: &mut KatanaApp,
    ctx: &egui::Context,
    previous_session_generation: u64,
) -> TestResult<u64> {
    let deadline = Instant::now() + BROWSER_UPDATE_TIMEOUT;
    loop {
        for preview in &mut app.tab_previews {
            preview.pane.poll_html_browser(ctx);
        }
        let session_generation = active_preview_session_generation(app)?;
        if session_generation != previous_session_generation
            && app
                .html_browser_frame_matching_rgb_pixels_for_test(GREEN_RGB)
                .is_some_and(|pixels| pixels > 0)
        {
            return Ok(session_generation);
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "timed out waiting for preview reload from session {previous_session_generation}"
            )
            .into());
        }
        std::thread::yield_now();
    }
}

fn load_initial_preview(
    app: &mut KatanaApp,
    ctx: &egui::Context,
    path: &std::path::Path,
) -> TestResult<u64> {
    let url = url::Url::from_file_path(path).map_err(|_| "file URL")?;
    app.process_action(&ctx, AppAction::OpenUrl(url.to_string()));
    start_pending_browser_sessions(app)?;
    app.wait_for_html_browser_frame_for_test(ctx, BROWSER_UPDATE_TIMEOUT)?;
    let previous_session_generation = active_preview_session_generation(&app)?;
    assert!(
        app.html_browser_frame_matching_rgb_pixels_for_test(RED_RGB)
            .is_some_and(|pixels| pixels > 0)
    );
    Ok(previous_session_generation)
}

fn click_primary_sidebar_refresh(app: &mut KatanaApp, ctx: &egui::Context) -> AppAction {
    app.pending_action = AppAction::None;
    let refresh_action = AppAction::RefreshDocument { is_manual: true };
    let pointer = super::render_action_button_for_test(
        ctx,
        app,
        egui::RawInput::default(),
        refresh_action.clone(),
    )
    .center();
    super::render_primary_controls_for_test(ctx, app, super::pointer_input(pointer, true));
    super::render_primary_controls_for_test(ctx, app, super::pointer_input(pointer, false));
    let action = app.take_action();
    assert!(matches!(
        action,
        AppAction::RefreshDocument { is_manual: true }
    ));
    action
}

#[test]
fn primary_sidebar_refresh_reads_changed_local_html_and_starts_green_session() -> TestResult {
    let _runtime_guard = crate::preview_pane::html_browser_runtime_test_guard();
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("sidebar-reload.html");
    let red_html = "<html><body style=\"background:#ff0000\"><p>Before</p></body></html>";
    let green_html = "<html><body style=\"background:#00ff00\"><p>After</p></body></html>";
    std::fs::write(&path, red_html)?;
    let ctx = egui::Context::default();
    let mut app = super::test_app();
    app.state.workspace.data = Some(katana_core::workspace::Workspace::new(
        directory.path(),
        Vec::new(),
    ));
    let previous_session_generation = load_initial_preview(&mut app, &ctx, &path)?;
    let action = click_primary_sidebar_refresh(&mut app, &ctx);

    std::fs::write(&path, green_html)?;
    app.process_action(&ctx, action);
    let session_generation = wait_for_preview_reload(&mut app, &ctx, previous_session_generation)?;

    assert_ne!(session_generation, previous_session_generation);
    assert!(
        app.html_browser_frame_matching_rgb_pixels_for_test(GREEN_RGB)
            .is_some_and(|pixels| pixels > 0)
    );
    assert_eq!(app.state.document.open_documents[0].buffer, green_html);
    Ok(())
}

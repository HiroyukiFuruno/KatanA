use crate::preview_pane::DocumentWorkerLifecycle;
use crate::preview_pane::image_html_surface::{
    HtmlBrowserSource, HtmlBrowserSurface, HtmlBrowserViewport,
};
use std::time::{Duration, Instant};

const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);
const RETIRED_SURFACE_COUNT: usize = 3;

#[test]
fn pending_real_html_surfaces_retire_without_leaving_workers() {
    let _runtime_guard = crate::preview_pane::html_browser_runtime_test_guard();
    assert_eq!(DocumentWorkerLifecycle::live_count(), 0);
    let mut surfaces = Vec::new();
    for _ in 0..RETIRED_SURFACE_COUNT {
        let source =
            HtmlBrowserSource::new("<h1>Pending close</h1>", "https://example.test/index.html")
                .expect("valid browser source");
        let mut surface = HtmlBrowserSurface::start(source);
        assert!(
            surface
                .start_pending_session(HtmlBrowserViewport::new(320, 240, 1.0).expect("viewport"))
        );
        surfaces.push(surface);
    }
    let started = Instant::now();
    drop(surfaces);
    assert!(
        started.elapsed() < CLOSE_TIMEOUT,
        "UI retirement exceeded the existing close budget"
    );
    let deadline = started + CLOSE_TIMEOUT;
    while DocumentWorkerLifecycle::live_count() != 0 && Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert_eq!(
        DocumentWorkerLifecycle::live_count(),
        0,
        "real HTML workers must finish inside the original close budget"
    );
}

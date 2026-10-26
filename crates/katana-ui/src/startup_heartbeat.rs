use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static HEARTBEAT_FRAME: AtomicU64 = AtomicU64::new(0);

pub(crate) struct StartupHeartbeat;

pub(crate) struct StartupFrameDiagnostics {
    started_at: Instant,
    cumulative_pass_nr: u64,
    prepared_at: Option<Instant>,
    panels_at: Option<Instant>,
}

impl StartupHeartbeat {
    pub(crate) fn is_enabled() -> bool {
        std::env::var_os("KATANA_STARTUP_HEARTBEAT").is_some()
    }

    pub(crate) fn begin_frame(
        ctx: &eframe::egui::Context,
        frame: &eframe::Frame,
    ) -> Option<StartupFrameDiagnostics> {
        if !Self::is_enabled() || !crate::debug_log::DebugLog::is_enabled() {
            return None;
        }

        let viewport = ctx.viewport_rect();
        let content = ctx.content_rect();
        let (viewport_minimized, viewport_occluded, viewport_focused) = ctx.input(|input| {
            let viewport = input.viewport();
            (viewport.minimized, viewport.occluded, input.focused)
        });
        let started_at = Instant::now();
        let cumulative_pass_nr = ctx.cumulative_pass_nr();
        tracing::info!(
            event = "startup_heartbeat_frame_begin",
            cumulative_pass_nr,
            previous_cpu_usage = ?frame.info().cpu_usage,
            viewport_id = ?ctx.viewport_id(),
            viewport_width_points = viewport.width(),
            viewport_height_points = viewport.height(),
            content_width_points = content.width(),
            content_height_points = content.height(),
            pixels_per_point = ctx.pixels_per_point(),
            viewport_minimized = ?viewport_minimized,
            viewport_occluded = ?viewport_occluded,
            viewport_focused = ?viewport_focused,
            heartbeat_frame = HEARTBEAT_FRAME.load(Ordering::Relaxed) + 1,
            repaint_causes = ?ctx.repaint_causes(),
            "startup heartbeat frame begin"
        );

        Some(StartupFrameDiagnostics {
            started_at,
            cumulative_pass_nr,
            prepared_at: None,
            panels_at: None,
        })
    }

    pub(crate) fn mark_prepared(diagnostics: &mut Option<StartupFrameDiagnostics>) {
        if let Some(diagnostics) = diagnostics {
            diagnostics.prepared_at = Some(Instant::now());
        }
    }

    pub(crate) fn mark_panels(diagnostics: &mut Option<StartupFrameDiagnostics>) {
        if let Some(diagnostics) = diagnostics {
            diagnostics.panels_at = Some(Instant::now());
        }
    }

    pub(crate) fn record_completed_frame_with_diagnostics(
        diagnostics: Option<StartupFrameDiagnostics>,
    ) {
        let Some(diagnostics) = diagnostics else {
            Self::record_completed_frame();
            return;
        };

        let heartbeat_started_at = Instant::now();
        Self::record_completed_frame();
        let heartbeat_finished_at = Instant::now();
        let ui_elapsed_us = heartbeat_started_at
            .duration_since(diagnostics.started_at)
            .as_micros() as u64;
        let prepared_elapsed_us = diagnostics
            .prepared_at
            .map(|at| at.duration_since(diagnostics.started_at).as_micros() as u64);
        let panels_elapsed_us = diagnostics
            .panels_at
            .map(|at| at.duration_since(diagnostics.started_at).as_micros() as u64);
        let heartbeat_record_us = heartbeat_finished_at
            .duration_since(heartbeat_started_at)
            .as_micros() as u64;
        let frame_elapsed_us = heartbeat_finished_at
            .duration_since(diagnostics.started_at)
            .as_micros() as u64;
        let heartbeat_frame = HEARTBEAT_FRAME.load(Ordering::Relaxed);
        tracing::info!(
            event = "startup_heartbeat_frame_end",
            cumulative_pass_nr = diagnostics.cumulative_pass_nr,
            ui_elapsed_us,
            frame_elapsed_us,
            prepared_elapsed_us,
            panels_elapsed_us,
            heartbeat_record_called = true,
            heartbeat_record_us,
            heartbeat_frame,
            "startup heartbeat frame end"
        );
    }

    pub(crate) fn record_completed_frame() {
        let Ok(path) = std::env::var("KATANA_STARTUP_HEARTBEAT") else {
            return;
        };
        let frame = HEARTBEAT_FRAME.fetch_add(1, Ordering::Relaxed) + 1;
        if let Err(error) = std::fs::write(&path, Self::contents(frame)) {
            tracing::error!(path, %error, "failed to write startup heartbeat");
        }
    }

    fn contents(frame: u64) -> String {
        format!("first-frame\nframe={frame}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::StartupHeartbeat;

    #[test]
    fn heartbeat_write_failure_is_logged_by_real_subscriber_in_child_process() {
        const CHILD_SENTINEL: &str = "KATANA_STARTUP_HEARTBEAT_ERROR_CHILD_V1";
        if std::env::var_os(CHILD_SENTINEL).as_deref() == Some(std::ffi::OsStr::new("1")) {
            let heartbeat_directory = std::env::var_os("KATANA_STARTUP_HEARTBEAT")
                .expect("child heartbeat directory must be provided");
            let log_path = std::path::PathBuf::from(&heartbeat_directory).join("heartbeat.log");
            let log_file = std::fs::File::create(&log_path).expect("log file must be created");
            tracing_subscriber::fmt()
                .with_ansi(false)
                .with_writer(log_file)
                .try_init()
                .expect("child tracing subscriber must initialize");

            StartupHeartbeat::record_completed_frame();
            return;
        }

        let heartbeat_directory = tempfile::tempdir().expect("heartbeat directory must exist");
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "startup_heartbeat::tests::heartbeat_write_failure_is_logged_by_real_subscriber_in_child_process",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("KATANA_STARTUP_HEARTBEAT", heartbeat_directory.path())
            .env(CHILD_SENTINEL, "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "child heartbeat error test failed: stdout={stdout} stderr={stderr}"
        );

        let log = std::fs::read_to_string(heartbeat_directory.path().join("heartbeat.log"))
            .expect("child must capture the real tracing output");
        assert!(
            log.contains("failed to write startup heartbeat"),
            "captured tracing output did not contain the heartbeat write error: {log}"
        );
    }

    #[test]
    fn heartbeat_contains_first_frame_and_monotonic_frame() {
        assert_eq!(StartupHeartbeat::contents(1), "first-frame\nframe=1\n");
        assert_eq!(StartupHeartbeat::contents(42), "first-frame\nframe=42\n");
    }

    #[test]
    fn heartbeat_records_every_completed_frame_in_child_process() {
        const CHILD_SENTINEL: &str = "KATANA_STARTUP_HEARTBEAT_COMPLETED_FRAMES_CHILD_V1";
        if std::env::var_os(CHILD_SENTINEL).as_deref() == Some(std::ffi::OsStr::new("1")) {
            let heartbeat_path = std::env::var_os("KATANA_STARTUP_HEARTBEAT")
                .expect("child heartbeat path must be provided");

            StartupHeartbeat::record_completed_frame();
            assert_eq!(
                std::fs::read_to_string(&heartbeat_path).unwrap(),
                "first-frame\nframe=1\n"
            );
            StartupHeartbeat::record_completed_frame();
            assert_eq!(
                std::fs::read_to_string(&heartbeat_path).unwrap(),
                "first-frame\nframe=2\n"
            );
            return;
        }

        let heartbeat = tempfile::NamedTempFile::new().unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "startup_heartbeat::tests::heartbeat_records_every_completed_frame_in_child_process",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("KATANA_STARTUP_HEARTBEAT", heartbeat.path())
            .env(CHILD_SENTINEL, "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "child completed-frame heartbeat test failed: stdout={stdout} stderr={stderr}"
        );
    }
}

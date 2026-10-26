use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use std::{io::Write, path::Path};

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
        if let Err(error) = write_frame(Path::new(&path), frame) {
            tracing::error!(path, %error, "failed to write startup heartbeat");
        }
    }

    fn contents(frame: u64) -> String {
        format!("first-frame\nframe={frame}\n")
    }
}

fn write_frame(path: &Path, frame: u64) -> std::io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(StartupHeartbeat::contents(frame).as_bytes())?;
    temporary
        .persist(path)
        .map(|_| ())
        .map_err(|error| error.error)
}

#[cfg(test)]
mod tests {
    use super::StartupHeartbeat;
    use std::io::{Read, Seek, SeekFrom};
    use std::sync::Barrier;
    use std::sync::atomic::{AtomicBool, Ordering};

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
    fn heartbeat_replacement_does_not_truncate_a_reader_held_snapshot() {
        let directory = tempfile::tempdir().expect("heartbeat test directory");
        let heartbeat_path = directory.path().join("heartbeat");
        super::write_frame(&heartbeat_path, 1).expect("initial heartbeat frame");

        let mut reader = std::fs::File::open(&heartbeat_path).expect("open held snapshot");
        super::write_frame(&heartbeat_path, 2).expect("replace heartbeat frame");

        reader
            .seek(SeekFrom::Start(0))
            .expect("rewind held snapshot");
        let mut held_snapshot = String::new();
        reader
            .read_to_string(&mut held_snapshot)
            .expect("read held snapshot");
        assert_eq!(held_snapshot, StartupHeartbeat::contents(1));
        assert_eq!(
            std::fs::read_to_string(&heartbeat_path).expect("read current heartbeat"),
            StartupHeartbeat::contents(2)
        );
    }

    #[test]
    fn heartbeat_readers_observe_complete_monotonic_records_during_updates() {
        const FINAL_FRAME: u64 = 500;
        use std::sync::Arc;
        use std::thread;

        let directory = tempfile::tempdir().expect("heartbeat test directory");
        let heartbeat_path = directory.path().join("heartbeat");
        super::write_frame(&heartbeat_path, 1).expect("initial heartbeat frame");
        let done = Arc::new(AtomicBool::new(false));
        let start = Arc::new(Barrier::new(2));
        let reader_path = heartbeat_path.clone();
        let reader_done = Arc::clone(&done);
        let reader_start = Arc::clone(&start);
        let reader = thread::spawn(move || {
            observe_heartbeat_updates(&reader_path, &reader_done, &reader_start);
        });

        start.wait();
        for frame in 2..=FINAL_FRAME {
            super::write_frame(&heartbeat_path, frame).expect("publish heartbeat frame");
        }
        done.store(true, Ordering::Release);
        reader.join().expect("heartbeat reader thread");
        let final_contents =
            std::fs::read_to_string(&heartbeat_path).expect("read final heartbeat");
        assert_eq!(parse_frame(&final_contents), FINAL_FRAME);
    }

    fn observe_heartbeat_updates(path: &std::path::Path, done: &AtomicBool, start: &Barrier) {
        let mut previous = 0;
        start.wait();
        loop {
            let contents = std::fs::read_to_string(path).expect("read heartbeat during updates");
            let frame = parse_frame(&contents);
            assert!(
                frame >= previous,
                "heartbeat regressed: {previous} -> {frame}"
            );
            previous = frame;
            if done.load(Ordering::Acquire) {
                return;
            }
        }
    }

    fn parse_frame(contents: &str) -> u64 {
        let mut lines = contents.lines();
        assert_eq!(lines.next(), Some("first-frame"));
        let frame = lines
            .next()
            .and_then(|line| line.strip_prefix("frame="))
            .and_then(|frame| frame.parse().ok())
            .expect("heartbeat must contain a numeric frame line");
        assert_eq!(lines.next(), None);
        frame
    }

    #[test]
    fn heartbeat_records_every_completed_frame_in_child_process() {
        const CHILD_SENTINEL: &str = "KATANA_STARTUP_HEARTBEAT_COMPLETED_FRAMES_CHILD_V1";
        if std::env::var_os(CHILD_SENTINEL).as_deref() == Some(std::ffi::OsStr::new("1")) {
            let heartbeat_path = std::env::var_os("KATANA_STARTUP_HEARTBEAT")
                .expect("child heartbeat path must be provided");

            StartupHeartbeat::record_completed_frame();
            assert_eq!(
                std::fs::read_to_string(&heartbeat_path).expect("read relative first frame"),
                "first-frame\nframe=1\n"
            );
            StartupHeartbeat::record_completed_frame();
            assert_eq!(
                std::fs::read_to_string(&heartbeat_path).expect("read relative second frame"),
                "first-frame\nframe=2\n"
            );
            return;
        }

        let directory = tempfile::tempdir().expect("relative heartbeat directory");
        let output = std::process::Command::new(
            std::env::current_exe().expect("current test executable"),
        )
        .args([
            "--exact",
            "startup_heartbeat::tests::heartbeat_records_every_completed_frame_in_child_process",
            "--nocapture",
            "--test-threads=1",
        ])
        .current_dir(directory.path())
        .env("KATANA_STARTUP_HEARTBEAT", "relative-heartbeat")
        .env(CHILD_SENTINEL, "1")
        .output()
        .expect("run relative heartbeat child test");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "child completed-frame heartbeat test failed: stdout={stdout} stderr={stderr}"
        );
    }
}

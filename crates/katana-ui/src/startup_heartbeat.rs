use std::{
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

static HEARTBEAT_FRAME: AtomicU64 = AtomicU64::new(0);
static HEARTBEAT_EPOCH: OnceLock<Instant> = OnceLock::new();
static LAST_HEARTBEAT_MILLIS: AtomicU64 = AtomicU64::new(0);
const HEARTBEAT_INTERVAL_MILLIS: u64 = 100;

pub(crate) struct StartupHeartbeat;

impl StartupHeartbeat {
    pub(crate) fn is_enabled() -> bool {
        std::env::var_os("KATANA_STARTUP_HEARTBEAT").is_some()
    }

    pub(crate) fn record_completed_frame() {
        let Ok(path) = std::env::var("KATANA_STARTUP_HEARTBEAT") else {
            return;
        };
        let elapsed_millis = HEARTBEAT_EPOCH
            .get_or_init(Instant::now)
            .elapsed()
            .as_millis() as u64;
        let previous_millis = LAST_HEARTBEAT_MILLIS.load(Ordering::Relaxed);
        if !Self::should_write(
            HEARTBEAT_FRAME.load(Ordering::Relaxed),
            previous_millis,
            elapsed_millis,
        ) {
            return;
        }
        LAST_HEARTBEAT_MILLIS.store(elapsed_millis, Ordering::Relaxed);
        let frame = HEARTBEAT_FRAME.fetch_add(1, Ordering::Relaxed) + 1;
        if let Err(error) = std::fs::write(&path, Self::contents(frame)) {
            tracing::error!(path, %error, "failed to write startup heartbeat");
        }
    }

    fn contents(frame: u64) -> String {
        format!("first-frame\nframe={frame}\n")
    }

    fn should_write(previous_frame: u64, previous_millis: u64, elapsed_millis: u64) -> bool {
        previous_frame == 0
            || elapsed_millis.saturating_sub(previous_millis) >= HEARTBEAT_INTERVAL_MILLIS
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
    fn heartbeat_writes_first_frame_then_throttles_to_the_interval() {
        assert!(StartupHeartbeat::should_write(0, 0, 0));
        assert!(!StartupHeartbeat::should_write(1, 100, 199));
        assert!(StartupHeartbeat::should_write(1, 100, 200));
    }
}

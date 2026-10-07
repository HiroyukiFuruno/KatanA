use katana_core::system::ProcessService;
use std::ffi::OsStr;
use std::path::Path;
#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::process::Child;
use std::process::Command;
use std::time::{Duration, Instant};

const SAMPLE_TIMEOUT: Duration = Duration::from_secs(5);
const SAMPLE_DURATION_SECONDS: &str = "1";
const SAMPLE_INTERVAL_MILLISECONDS: &str = "10";
const POLL_INTERVAL: Duration = Duration::from_millis(10);

fn enabled(value: Option<&OsStr>) -> bool {
    value == Some(OsStr::new("1"))
}

fn sample_command(pid: u32, output: &Path) -> Command {
    let mut command = ProcessService::create_command("/usr/bin/sample");
    command
        .arg(pid.to_string())
        .arg(SAMPLE_DURATION_SECONDS)
        .arg(SAMPLE_INTERVAL_MILLISECONDS)
        .arg("-file")
        .arg(output);
    command
}

fn sample_timed_out(started: Instant, now: Instant) -> bool {
    now.duration_since(started) >= SAMPLE_TIMEOUT
}

#[cfg(target_os = "macos")]
pub(super) fn capture_if_enabled() -> Option<Result<PathBuf, String>> {
    if !enabled(std::env::var_os("KATANA_HTML_FAILURE_SAMPLE").as_deref()) {
        return None;
    }
    Some(capture_process_sample(
        std::process::id(),
        &sample_output_directory(),
    ))
}

#[cfg(target_os = "macos")]
fn sample_output_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/html-startup-failures")
}

#[cfg(target_os = "macos")]
fn capture_process_sample(pid: u32, directory: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("could not create sample directory: {error}"))?;
    let path = unique_sample_path(pid, directory)?;
    let mut child = ProcessService::spawn(sample_command(pid, &path))
        .map_err(|error| format!("could not start macOS sample: {error}"))?;
    wait_for_sample(&mut child)?;
    Ok(path)
}

#[cfg(target_os = "macos")]
fn unique_sample_path(pid: u32, directory: &Path) -> Result<PathBuf, String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| format!("system clock is before UNIX epoch: {error}"))?
        .as_nanos();
    let path = directory.join(format!("html-startup-{pid}-{timestamp}.sample.txt"));
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| format!("could not reserve sample output path: {error}"))?;
    Ok(path)
}

#[cfg(target_os = "macos")]
fn wait_for_sample(child: &mut Child) -> Result<(), String> {
    let started = Instant::now();
    loop {
        let status = child.try_wait().map_err(|error| {
            let kill = child.kill();
            let reap = child.wait();
            format!("could not inspect sample process: {error} (kill={kill:?}, reap={reap:?})")
        })?;
        if let Some(status) = status {
            return if status.success() {
                Ok(())
            } else {
                Err(format!("macOS sample exited with {status}"))
            };
        }
        if sample_timed_out(started, Instant::now()) {
            return terminate_sample(child);
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

#[cfg(target_os = "macos")]
fn terminate_sample(child: &mut Child) -> Result<(), String> {
    let kill = child.kill();
    let reap = child.wait();
    Err(format!(
        "macOS sample exceeded the 5-second limit (kill={kill:?}, reap={reap:?})"
    ))
}

#[cfg(test)]
mod tests {
    use super::{SAMPLE_TIMEOUT, enabled, sample_command, sample_timed_out};
    use katana_core::system::ProcessService;
    use std::ffi::OsStr;
    use std::path::Path;
    use std::time::{Duration, Instant};

    #[test]
    fn sample_opt_in_requires_exact_one() {
        assert!(!enabled(None));
        assert!(!enabled(Some(OsStr::new("0"))));
        assert!(!enabled(Some(OsStr::new("true"))));
        assert!(enabled(Some(OsStr::new("1"))));
    }

    #[test]
    fn sample_command_targets_pid_and_writes_only_to_the_requested_path() {
        let path = Path::new("target/html-startup-failures/sample.txt");
        let command = sample_command(4321, path);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(command.get_program(), "/usr/bin/sample");
        assert_eq!(
            args,
            [
                "4321",
                "1",
                "10",
                "-file",
                "target/html-startup-failures/sample.txt"
            ]
        );
    }

    #[test]
    fn sample_timeout_boundary_is_five_seconds() {
        let start = Instant::now();
        assert_eq!(SAMPLE_TIMEOUT, Duration::from_secs(5));
        assert!(!sample_timed_out(
            start,
            start + Duration::from_millis(4_999)
        ));
        assert!(sample_timed_out(start, start + SAMPLE_TIMEOUT));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn sample_captures_owned_child_pid_to_workspace_failure_directory() {
        use super::{capture_process_sample, sample_output_directory};

        /* WHY: 実プロファイラの負荷を初回描画の時間制約と競合させない。 */
        let _runtime_guard = crate::preview_pane::html_browser_runtime_test_guard();
        let mut child = ProcessService::create_command("sleep")
            .arg("10")
            .spawn()
            .expect("start owned short-lived sample target");
        let pid = child.id();
        let result = capture_process_sample(pid, &sample_output_directory());
        let _ = child.kill();
        let _ = child.wait();
        let path = result.expect("sample owned child");
        assert!(path.starts_with(sample_output_directory()));
        assert!(
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .contains(&pid.to_string())
        );
        let content = std::fs::read_to_string(path).expect("read sample report");
        assert!(content.contains(&pid.to_string()));
        assert!(content.contains("Thread"));
    }
}

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub fn screenshot(process_name: &str, output: &Path) -> Result<()> {
    if cfg!(target_os = "macos") {
        capture_macos(process_name, output)
    } else if cfg!(target_os = "linux") {
        capture_linux(output)
    } else {
        bail!("screenshot capture is not supported on this platform")
    }
}

#[cfg(target_os = "macos")]
fn capture_macos(process_name: &str, output: &Path) -> Result<()> {
    let window_id = wait_for_window_id_macos(process_name, 10)?;
    let status = Command::new("screencapture")
        .args(["-l", &window_id.to_string(), &output.display().to_string()])
        .status()?;
    anyhow::ensure!(status.success(), "screencapture exited with {status}");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn capture_macos(_process_name: &str, _output: &Path) -> Result<()> {
    bail!("macOS capture called on non-macOS platform")
}

#[cfg(target_os = "linux")]
fn capture_linux(output: &Path) -> Result<()> {
    if which("scrot") {
        let status = Command::new("scrot")
            .args(["--focused", &output.display().to_string()])
            .status()?;
        anyhow::ensure!(status.success(), "scrot exited with {status}");
    } else if which("import") {
        let status = Command::new("import")
            .args(["-window", "root", &output.display().to_string()])
            .status()?;
        anyhow::ensure!(status.success(), "import exited with {status}");
    } else {
        bail!("no screenshot tool found; install scrot or ImageMagick");
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn capture_linux(_output: &Path) -> Result<()> {
    bail!("Linux capture called on non-Linux platform")
}

#[cfg(target_os = "linux")]
fn which(cmd: &str) -> bool {
    Command::new("which").arg(cmd).output().map_or(false, |o| o.status.success())
}

fn wait_for_window_id_macos(process_name: &str, timeout_secs: u64) -> Result<u64> {
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        if let Some(id) = query_window_id_macos(process_name) {
            return Ok(id);
        }
        if std::time::Instant::now() >= deadline {
            bail!("KatanA window not found within {timeout_secs}s");
        }
        thread::sleep(Duration::from_millis(500));
    }
}

fn query_window_id_macos(process_name: &str) -> Option<u64> {
    let script = format!(
        r#"
import Quartz
wl = Quartz.CGWindowListCopyWindowInfo(
    Quartz.kCGWindowListOptionOnScreenOnly | Quartz.kCGWindowListExcludeDesktopElements,
    Quartz.kCGNullWindowID
)
for w in wl:
    if w.get('kCGWindowOwnerName') == '{process_name}' and w.get('kCGWindowLayer', 999) == 0:
        wid = w.get('kCGWindowNumber')
        bounds = w.get('kCGWindowBounds', {{}})
        if wid and bounds.get('Width', 0) > 0 and bounds.get('Height', 0) > 0:
            print(wid)
            break
"#
    );
    let out = Command::new("python3").arg("-c").arg(&script).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
    raw.parse::<u64>().ok()
}

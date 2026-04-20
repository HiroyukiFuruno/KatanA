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
    let bounds = wait_for_window_bounds_macos(process_name, 10)?;
    let region = format!("{},{},{},{}", bounds.0, bounds.1, bounds.2, bounds.3);
    let status = Command::new("screencapture")
        .args(["-R", &region, &output.display().to_string()])
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
    // Try scrot first, then import (ImageMagick) as fallback
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

fn wait_for_window_bounds_macos(
    process_name: &str,
    timeout_secs: u64,
) -> Result<(i32, i32, i32, i32)> {
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        if let Some(bounds) = query_window_bounds_macos(process_name) {
            return Ok(bounds);
        }
        if std::time::Instant::now() >= deadline {
            bail!("KatanA window not found within {timeout_secs}s");
        }
        thread::sleep(Duration::from_millis(500));
    }
}

fn query_window_bounds_macos(process_name: &str) -> Option<(i32, i32, i32, i32)> {
    let script = format!(
        r#"tell application "System Events"
    set procs to (every process whose name is "{process_name}")
    if procs is {{}} then return ""
    set proc to first item of procs
    if (count of windows of proc) is 0 then return ""
    set win to first window of proc
    set {{x, y}} to position of win
    set {{w, h}} to size of win
    return (x as string) & "," & (y as string) & "," & (w as string) & "," & (h as string)
end tell"#
    );
    let out = Command::new("osascript").arg("-e").arg(&script).output().ok()?;
    let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if raw.is_empty() {
        return None;
    }
    let parts: Vec<i32> = raw.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    if parts.len() == 4 && parts[2] > 0 && parts[3] > 0 {
        Some((parts[0], parts[1], parts[2], parts[3]))
    } else {
        None
    }
}

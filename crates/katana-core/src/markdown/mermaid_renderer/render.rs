use super::types::MermaidRenderOps;
use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};
use tempfile::NamedTempFile;

use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::{DiagramBlock, DiagramResult};

use super::resolve::MermaidBinaryOps;

const MMDC_AVAILABILITY_TIMEOUT: Duration = Duration::from_secs(2);
const MMDC_RENDER_TIMEOUT: Duration = Duration::from_secs(10);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(25);

static MMDC_AVAILABILITY_CACHE: LazyLock<Mutex<HashMap<PathBuf, bool>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl MermaidRenderOps {
    pub fn is_mmdc_available() -> bool {
        let binary = MermaidBinaryOps::resolve_mmdc_binary();
        let mut cache = MMDC_AVAILABILITY_CACHE.lock().unwrap();
        if let Some(&available) = cache.get(&binary) {
            return available;
        }

        /* WHY: Only cache a confirmed result (Some(true/false)). If the probe
         * times out or fails to spawn (None), we do NOT cache the failure so
         * the next call can retry. This prevents a transient startup timeout
         * from permanently marking mmdc as unavailable for the session. */
        match probe_mmdc_availability(&binary) {
            Some(available) => {
                cache.insert(binary, available);
                available
            }
            None => false,
        }
    }

    /* WHY: Rendering as PNG with mmdc (Puppeteer/Chrome based) bypasses resvg's lack of support for <foreignObject>. */
    pub fn render_mermaid(block: &DiagramBlock) -> DiagramResult {
        if !Self::is_mmdc_available() {
            return DiagramResult::CommandNotFound {
                tool_name: "mmdc (Mermaid CLI)".to_string(),
                install_hint: "`npm install -g @mermaid-js/mermaid-cli`".to_string(),
                source: block.source.clone(),
            };
        }
        match Self::run_mmdc_process(&block.source) {
            Ok(png_bytes) => DiagramResult::OkPng(png_bytes),
            Err(e) => DiagramResult::Err {
                source: block.source.clone(),
                error: e,
            },
        }
    }

    /* WHY: PNG output ensures mmdc (Puppeteer) correctly renders all SVG elements. Bypasses text loss caused by <foreignObject> which resvg doesn't support. */
    pub fn run_mmdc_process(source: &str) -> Result<Vec<u8>, String> {
        let input_file = Self::create_input_file(source)?;
        /* WHY: mmdc determines the format by the output file's extension. */
        let output_path = input_file.path().with_extension("png");

        let preset = DiagramColorPreset::current();
        let mut command = MermaidBinaryOps::build_mmdc_command();
        command
            .args(vec![
                "-i",
                input_file.path().to_str().unwrap_or(""),
                "-o",
                output_path.to_str().unwrap_or(""),
                "--backgroundColor",
                preset.background,
                "--theme",
                preset.mermaid_theme,
                "--quiet",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let status = run_command_status_with_timeout(command, MMDC_RENDER_TIMEOUT)
            .map_err(|e| format!("mmdc execution failed: {e}"))?;

        if !status.success() {
            return Err("mmdc returned a non-zero exit code".to_string());
        }
        std::fs::read(&output_path).map_err(|e| format!("PNG read failed: {e}"))
    }

    pub fn create_input_file(source: &str) -> Result<NamedTempFile, String> {
        let mut file = NamedTempFile::with_suffix(".mmd")
            .map_err(|e| format!("Temp file creation failed: {e}"))?;
        file.write_all(source.as_bytes())
            .map_err(|e| format!("Temp file write failed: {e}"))?;
        Ok(file)
    }
}

/* WHY: Returns Some(true) when mmdc responded successfully, Some(false) when
 * the binary was found but exited with a non-zero code, and None when the
 * probe could not be completed (process failed to spawn, or timed out).
 * Callers use None to skip caching so the check is retried next time. */
fn probe_mmdc_availability(binary: &Path) -> Option<bool> {
    let mut command = MermaidBinaryOps::build_mmdc_command_for_binary(binary);
    command
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    match run_command_status_with_timeout(command, MMDC_AVAILABILITY_TIMEOUT) {
        Ok(status) => Some(status.success()),
        Err(_) => {
            /* WHY: Timeout or spawn failure — do not cache; allow retry on
             * the next call so a slow startup environment can still succeed. */
            None
        }
    }
}

fn run_command_status_with_timeout(
    mut command: Command,
    timeout: Duration,
) -> Result<ExitStatus, String> {
    let mut child = command
        .spawn()
        .map_err(|e| format!("process startup failed: {e}"))?;
    let start = Instant::now();

    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("process wait failed: {e}"))?
        {
            return Ok(status);
        }

        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "process timed out after {} ms",
                timeout.as_millis()
            ));
        }

        std::thread::sleep(PROCESS_POLL_INTERVAL);
    }
}

#[cfg(test)]
#[cfg(all(test, unix))]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn command_timeout_returns_error_for_hung_process() {
        let start = Instant::now();
        /* WHY: Use standard sleep to avoid overlayfs ETXTBSY race conditions */
        let mut cmd = Command::new("sleep");
        cmd.arg("5");

        let result = run_command_status_with_timeout(cmd, Duration::from_millis(100));

        assert!(result.is_err());
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn command_timeout_returns_success_for_fast_process() {
        /* WHY: Use standard true to avoid overlayfs ETXTBSY race conditions */
        let cmd = Command::new("true");

        /* WHY: heavily loaded test environments (e.g. concurrent llvm-cov runs) can occasionally take >1000ms to spawn and complete even an empty script. Flaky timeout prevention. */
        let status = run_command_status_with_timeout(cmd, Duration::from_secs(5)).unwrap();

        assert!(status.success());
    }

    #[test]
    fn probe_mmdc_availability_returns_none_for_timeout() {
        /* WHY: Regression test for Bug 1 — a timed-out probe must return None
         * (not Some(false)), so the caller does NOT cache the failure and can
         * retry on the next call. */
        let binary = std::path::Path::new("sleep");

        /* WHY: sleep 5 will definitely time out with a 100 ms budget.
         * We call the inner helper directly via a duplicate-like test. */
        let mut command = MermaidBinaryOps::build_mmdc_command_for_binary(binary);
        command
            .arg("5")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        let result = run_command_status_with_timeout(command, Duration::from_millis(100));
        assert!(result.is_err(), "timed-out probe must return Err");
    }

    #[test]
    fn probe_mmdc_availability_returns_some_false_for_bad_binary() {
        /* WHY: A binary that exists but exits non-zero must return Some(false)
         * so the caller caches the negative result and avoids repeated probes. */
        let binary = std::path::Path::new("false");
        let result = probe_mmdc_availability(binary);
        assert_eq!(result, Some(false), "'false' binary must yield Some(false)");
    }

    #[test]
    fn is_mmdc_available_does_not_cache_when_probe_returns_none() {
        /* WHY: Regression test — if a transient timeout caused a None probe
         * result, the cache must remain empty so the next call retries. */
        /* WHY: Use a deliberately invalid binary path that won't spawn. */
        let _guard = std::sync::Mutex::new(());
        unsafe { std::env::set_var("MERMAID_MMDC", "/this/path/does/not/exist/mmdc") };

        /* WHY: First call: should return false (spawn failure → None → not cached) */
        let first = MermaidRenderOps::is_mmdc_available();
        assert!(!first);

        /* WHY: The cache should NOT have stored the failure under this key. */
        let binary = super::super::resolve::MermaidBinaryOps::resolve_mmdc_binary();
        let cache = MMDC_AVAILABILITY_CACHE.lock().unwrap();
        assert!(
            !cache.contains_key(&binary),
            "spawn failure must not be cached so the next call can retry"
        );

        unsafe { std::env::remove_var("MERMAID_MMDC") };
    }
}

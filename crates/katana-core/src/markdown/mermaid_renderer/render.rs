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

        let available = probe_mmdc_availability(&binary);
        cache.insert(binary, available);
        available
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

fn probe_mmdc_availability(binary: &Path) -> bool {
    let mut command = MermaidBinaryOps::build_mmdc_command_for_binary(binary);
    command
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    run_command_status_with_timeout(command, MMDC_AVAILABILITY_TIMEOUT)
        .map(|status| status.success())
        .unwrap_or(false)
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt};

    fn write_executable_script(body: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let script_path = dir.path().join("fake-mmdc.sh");
        fs::write(&script_path, body).unwrap();
        let mut permissions = fs::metadata(&script_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions).unwrap();
        dir
    }

    #[test]
    fn command_timeout_returns_error_for_hung_process() {
        let dir = write_executable_script("#!/bin/sh\nsleep 5\n");
        let script_path = dir.path().join("fake-mmdc.sh");
        let start = Instant::now();

        let result =
            run_command_status_with_timeout(Command::new(&script_path), Duration::from_millis(100));

        assert!(result.is_err());
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn command_timeout_returns_success_for_fast_process() {
        let dir = write_executable_script("#!/bin/sh\nexit 0\n");
        let script_path = dir.path().join("fake-mmdc.sh");

        let status =
            run_command_status_with_timeout(Command::new(&script_path), Duration::from_secs(1))
                .unwrap();

        assert!(status.success());
    }
}

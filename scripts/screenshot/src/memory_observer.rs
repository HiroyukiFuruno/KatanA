use anyhow::{Context, Result, bail, ensure};
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, ExitStatus, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const OBSERVER_TIMEOUT: Duration = Duration::from_secs(15);
const POLL_INTERVAL: Duration = Duration::from_millis(50);
const VMMAP: &str = "/usr/bin/vmmap";
const HEAP: &str = "/usr/bin/heap";
const MAX_OUTPUT_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Serialize)]
struct CommandManifest<'a> {
    diagnostic_only: bool,
    output_root: String,
    step_index: usize,
    phase: &'a str,
    parent_pid: u32,
    observer_started_at_ms: u128,
    observer_finished_at_ms: u128,
    command_path: &'a str,
    args: Vec<String>,
    command_started_at_ms: u128,
    command_finished_at_ms: u128,
    exit_status: String,
    timed_out: bool,
    output_within_limit: bool,
    output_limit_exceeded: bool,
}

#[derive(Debug)]
struct CommandResult {
    status: ExitStatus,
    timed_out: bool,
    output_limit_exceeded: bool,
}

struct CommandRun<'a> {
    step_index: usize,
    phase: &'a str,
    observer_started_at_ms: u128,
    args: Vec<String>,
    path: &'a str,
    command_started_at_ms: u128,
    command_finished_at_ms: u128,
    timed_out: bool,
    status: ExitStatus,
    output_within_limit: bool,
    output_limit_exceeded: bool,
}

struct CommandSpec<'a> {
    step_index: usize,
    phase: &'a str,
    observer_started_at_ms: u128,
    path: &'a str,
    option: &'a str,
    stem: &'a str,
    name: &'a str,
}

#[derive(Debug)]
pub struct MemoryObserver {
    output_dir: PathBuf,
    parent_pid: u32,
    enabled: bool,
    sequence: u32,
}

impl MemoryObserver {
    pub fn new(enabled: bool, output_dir: &Path) -> Result<Self> {
        if enabled && !cfg!(target_os = "macos") {
            bail!("--memory-diagnostics is supported only on macOS")
        }
        let output_dir = if enabled {
            output_dir.canonicalize().with_context(|| {
                format!(
                    "cannot canonicalize diagnostic output {}",
                    output_dir.display()
                )
            })?
        } else {
            output_dir.to_path_buf()
        };
        Ok(Self {
            output_dir,
            parent_pid: std::process::id(),
            enabled,
            sequence: 0,
        })
    }

    pub fn observe(&mut self, step_index: usize, phase: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        validate_phase(phase)?;
        let stem = self.next_stem(step_index, phase);
        let (vmmap, heap) = self.run_pair(step_index, phase, &stem)?;
        ensure!(
            vmmap.status.success()
                && heap.status.success()
                && !vmmap.timed_out
                && !heap.timed_out
                && !vmmap.output_limit_exceeded
                && !heap.output_limit_exceeded,
            "memory diagnostic command failed for {stem}"
        );
        Ok(())
    }

    fn next_stem(&mut self, step_index: usize, phase: &str) -> String {
        let sequence = self.sequence;
        self.sequence = self.sequence.saturating_add(1);
        format!("memory-step-{step_index:04}-{sequence:04}-{phase}")
    }

    fn run_pair(
        &self,
        step_index: usize,
        phase: &str,
        stem: &str,
    ) -> Result<(CommandResult, CommandResult)> {
        let started = epoch_ms()?;
        let vmmap = self.observe_command(CommandSpec {
            step_index,
            phase,
            observer_started_at_ms: started,
            path: VMMAP,
            option: "-summary",
            stem,
            name: "vmmap",
        })?;
        let heap = self.observe_command(CommandSpec {
            step_index,
            phase,
            observer_started_at_ms: started,
            path: HEAP,
            option: "-s",
            stem,
            name: "heap",
        })?;
        Ok((vmmap, heap))
    }

    fn observe_command(&self, spec: CommandSpec<'_>) -> Result<CommandResult> {
        let args = vec![spec.option.into(), self.parent_pid.to_string()];
        self.run_command(spec, args)
    }

    fn run_command(&self, spec: CommandSpec<'_>, args: Vec<String>) -> Result<CommandResult> {
        let (stdout, stderr) = self.open_output_files(spec.stem, spec.name)?;
        let command_started_at_ms = epoch_ms()?;
        let stdout_path = self
            .output_dir
            .join(format!("{}-{}.stdout", spec.stem, spec.name));
        let stderr_path = self
            .output_dir
            .join(format!("{}-{}.stderr", spec.stem, spec.name));
        let status = execute_command(spec.path, &args, stdout, stderr, &stdout_path, &stderr_path)?;
        let command_finished_at_ms = epoch_ms()?;
        let output_within_limit = !status.output_limit_exceeded;
        self.write_manifest(
            CommandRun {
                step_index: spec.step_index,
                phase: spec.phase,
                observer_started_at_ms: spec.observer_started_at_ms,
                args,
                path: spec.path,
                command_started_at_ms,
                command_finished_at_ms,
                timed_out: status.timed_out,
                status: status.status,
                output_within_limit,
                output_limit_exceeded: status.output_limit_exceeded,
            },
            spec.stem,
            spec.name,
        )?;
        ensure!(
            output_within_limit,
            "memory diagnostic output exceeded 8 MiB for {}-{}",
            spec.stem,
            spec.name
        );
        Ok(status)
    }

    fn write_manifest(&self, run: CommandRun<'_>, stem: &str, name: &str) -> Result<()> {
        let manifest = CommandManifest {
            diagnostic_only: true,
            output_root: self.output_dir.display().to_string(),
            step_index: run.step_index,
            phase: run.phase,
            parent_pid: self.parent_pid,
            observer_started_at_ms: run.observer_started_at_ms,
            observer_finished_at_ms: run.command_finished_at_ms,
            command_path: run.path,
            args: run.args,
            command_started_at_ms: run.command_started_at_ms,
            command_finished_at_ms: run.command_finished_at_ms,
            exit_status: format!("{:?}", run.status),
            timed_out: run.timed_out,
            output_within_limit: run.output_within_limit,
            output_limit_exceeded: run.output_limit_exceeded,
        };
        let manifest_path = self.output_dir.join(format!("{stem}-{name}.json"));
        let mut file = new_output_file(&manifest_path)?;
        file.write_all(&serde_json::to_vec_pretty(&manifest)?)
            .with_context(|| format!("cannot write {}", manifest_path.display()))?;
        Ok(())
    }

    fn open_output_files(&self, stem: &str, name: &str) -> Result<(File, File)> {
        let stdout_path = self.output_dir.join(format!("{stem}-{name}.stdout"));
        let stderr_path = self.output_dir.join(format!("{stem}-{name}.stderr"));
        let stdout = new_output_file(&stdout_path)?;
        let stderr = new_output_file(&stderr_path)?;
        Ok((stdout, stderr))
    }
}

fn epoch_ms() -> Result<u128> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX epoch")?
        .as_millis())
}

fn validate_phase(phase: &str) -> Result<()> {
    ensure!(
        !phase.is_empty() && phase.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
        "invalid memory diagnostic phase"
    );
    Ok(())
}

fn spawn_command(path: &str, args: &[String], stdout: File, stderr: File) -> Result<Child> {
    katana_core::system::ProcessService::create_command(path)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .with_context(|| format!("failed to start {path}"))
}

fn execute_command(
    path: &str,
    args: &[String],
    stdout: File,
    stderr: File,
    stdout_path: &Path,
    stderr_path: &Path,
) -> Result<CommandResult> {
    let mut child = spawn_command(path, args, stdout, stderr)?;
    wait_bounded(&mut child, stdout_path, stderr_path)
}

fn new_output_file(path: &Path) -> Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("cannot create diagnostic output {}", path.display()))
}

fn wait_bounded(
    child: &mut Child,
    stdout_path: &Path,
    stderr_path: &Path,
) -> Result<CommandResult> {
    let deadline = Instant::now() + OBSERVER_TIMEOUT;
    loop {
        let over_limit = match output_limit_exceeded(stdout_path, stderr_path) {
            Ok(value) => value,
            Err(error) => return Err(reap_after_error(child, error)),
        };
        if over_limit {
            let status = stop_and_reap(child)?;
            return Ok(CommandResult {
                status,
                timed_out: false,
                output_limit_exceeded: true,
            });
        }
        let waited = match child.try_wait() {
            Ok(status) => status,
            Err(error) => return Err(reap_after_error(child, error.into())),
        };
        if let Some(status) = waited {
            return Ok(CommandResult {
                status,
                timed_out: false,
                output_limit_exceeded: false,
            });
        }
        if Instant::now() >= deadline {
            let status = stop_and_reap(child)?;
            return Ok(CommandResult {
                status,
                timed_out: true,
                output_limit_exceeded: false,
            });
        }
        sleep(POLL_INTERVAL);
    }
}

fn output_limit_exceeded(stdout: &Path, stderr: &Path) -> Result<bool> {
    let stdout_bytes = std::fs::metadata(stdout)?.len();
    let stderr_bytes = std::fs::metadata(stderr)?.len();
    Ok(stdout_bytes > MAX_OUTPUT_BYTES
        || stderr_bytes > MAX_OUTPUT_BYTES
        || stdout_bytes + stderr_bytes > MAX_OUTPUT_BYTES)
}

fn stop_and_reap(child: &mut Child) -> Result<ExitStatus> {
    match child.kill() {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).context("failed to stop memory diagnostic command"),
    }
    child
        .wait()
        .context("failed to reap memory diagnostic command")
}

fn reap_after_error(child: &mut Child, error: anyhow::Error) -> anyhow::Error {
    match stop_and_reap(child) {
        Ok(_) => anyhow::anyhow!("memory diagnostic wait failed: {error}"),
        Err(reap_error) => {
            anyhow::anyhow!("memory diagnostic wait failed: {error}; reap failed: {reap_error}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_paths_and_arguments_are_fixed() {
        assert_eq!(VMMAP, "/usr/bin/vmmap");
        assert_eq!(HEAP, "/usr/bin/heap");
        assert_eq!(OBSERVER_TIMEOUT, Duration::from_secs(15));
    }

    #[test]
    fn disabled_observer_does_not_spawn_or_write() {
        let dir = tempfile::tempdir().expect("temporary output directory");
        let mut observer = MemoryObserver::new(false, dir.path()).expect("observer setup");
        observer
            .observe(1, "record_runtime_snapshot")
            .expect("disabled observer is a no-op");
        assert_eq!(std::fs::read_dir(dir.path()).expect("directory").count(), 0);
    }

    #[test]
    fn phase_rejects_path_injection() {
        let error = validate_phase("close/active").expect_err("phase must be filename-safe");
        assert!(
            error
                .to_string()
                .contains("invalid memory diagnostic phase")
        );
    }

    #[test]
    fn output_files_are_create_new() {
        let dir = tempfile::tempdir().expect("temporary output directory");
        let path = dir.path().join("raw.stdout");
        new_output_file(&path).expect("first output file");
        let error = new_output_file(&path).expect_err("existing output must not be replaced");
        assert!(
            error
                .to_string()
                .contains("cannot create diagnostic output")
        );
    }

    #[cfg(unix)]
    #[test]
    fn output_symlink_is_not_followed() {
        let dir = tempfile::tempdir().expect("temporary output directory");
        let target = dir.path().join("target");
        std::fs::write(&target, b"keep").expect("target");
        let link = dir.path().join("raw.stdout");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        assert!(new_output_file(&link).is_err());
        assert_eq!(std::fs::read(&target).expect("target contents"), b"keep");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn real_owned_process_probe_records_fixed_arguments() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("katana-memory-observer-owned-probe-{nonce}"));
        std::fs::create_dir_all(&dir).expect("diagnostic output");
        let mut observer = MemoryObserver::new(true, &dir).expect("macOS observer");
        observer
            .observe(7, "owned_probe")
            .expect("vmmap and heap probe");
        let manifests = std::fs::read_dir(&dir)
            .expect("diagnostic output")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"));
        let mut seen = 0;
        for entry in manifests {
            let manifest_path = entry.path();
            let manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&manifest_path).expect("manifest"))
                    .expect("valid manifest");
            assert_eq!(manifest["diagnostic_only"], true);
            assert_eq!(manifest["parent_pid"], std::process::id());
            assert_eq!(manifest["phase"], "owned_probe");
            assert_eq!(manifest["args"][1], std::process::id().to_string());
            assert!(
                manifest["exit_status"]
                    .as_str()
                    .is_some_and(|status| status.contains("0"))
            );
            assert!(
                manifest["command_path"]
                    .as_str()
                    .is_some_and(|path| path == VMMAP || path == HEAP)
            );
            assert!(manifest_path.with_extension("stdout").exists());
            seen += 1;
        }
        assert_eq!(seen, 2);
        eprintln!("owned memory diagnostic raw output: {}", dir.display());
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn enabled_observer_rejects_unsupported_os() {
        let dir = tempfile::tempdir().expect("temporary output directory");
        let error = MemoryObserver::new(true, dir.path()).expect_err("non-macOS must reject");
        assert!(error.to_string().contains("only on macOS"));
    }
}

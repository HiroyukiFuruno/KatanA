use super::resource_snapshot::UiResourceSnapshot;
use crate::debug_log::DebugLog;
use serde::Serialize;
use std::{
    ffi::OsString,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

const RECEIPT_DIRECTORY_ENV: &str = "KATANA_DOCUMENT_RESOURCE_RECEIPT_DIR";
const RUN_ID_ENV: &str = "KATANA_DOCUMENT_RESOURCE_RUN_ID";
const RECEIPT_INTERVAL: Duration = Duration::from_millis(100);
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(crate) struct ResourceReceiptState {
    directory: PathBuf,
    run_id: String,
    sequence: u64,
    last_capture_at: Option<Instant>,
    last_snapshot: Option<UiResourceSnapshot>,
    failed: bool,
}

#[derive(Serialize)]
struct Receipt<'a> {
    schema: &'static str,
    run_id: &'a str,
    pid: u32,
    seq: u64,
    #[serde(flatten)]
    snapshot: &'a UiResourceSnapshot,
    snapshot_elapsed_us: u64,
}

impl ResourceReceiptState {
    pub(crate) fn from_environment() -> Option<Self> {
        match Self::configuration(
            DebugLog::is_enabled(),
            std::env::var_os(RECEIPT_DIRECTORY_ENV).map(PathBuf::from),
            std::env::var_os(RUN_ID_ENV),
        ) {
            Ok(state) => state,
            Err(error) => {
                DebugLog::write("document_resource_receipt_error", format_args!("{error}"));
                None
            }
        }
    }

    pub(crate) fn configuration(
        debug_enabled: bool,
        directory: Option<PathBuf>,
        run_id: Option<OsString>,
    ) -> Result<Option<Self>, String> {
        if !debug_enabled {
            return Ok(None);
        }
        let (directory, run_id) = match (directory, run_id) {
            (None, None) => return Ok(None),
            (Some(directory), Some(run_id)) => (directory, run_id),
            _ => return Err("receipt directory and run ID must be set together".to_owned()),
        };
        if !directory.is_absolute() {
            return Err("receipt directory must be absolute".to_owned());
        }
        let metadata = fs::symlink_metadata(&directory)
            .map_err(|error| format!("receipt directory metadata failed: {error}"))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err("receipt directory must be an existing non-symlink directory".to_owned());
        }
        let run_id = run_id
            .into_string()
            .map_err(|_| "receipt run ID must be ASCII".to_owned())?;
        if run_id.is_empty()
            || run_id.len() > 64
            || !run_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(
                "receipt run ID must be 1-64 ASCII letters, digits, dot, dash, or underscore"
                    .to_owned(),
            );
        }
        Ok(Some(Self {
            directory,
            run_id,
            sequence: 0,
            last_capture_at: None,
            last_snapshot: None,
            failed: false,
        }))
    }

    pub(crate) fn record(&mut self, snapshot: UiResourceSnapshot, snapshot_elapsed_us: u64) {
        if self.failed || self.last_snapshot.as_ref() == Some(&snapshot) {
            return;
        }
        let Some(sequence) = self.sequence.checked_add(1) else {
            self.fail("receipt sequence overflow".to_owned());
            return;
        };
        let serialized_at = Instant::now();
        let payload = match serde_json::to_vec(&Receipt {
            schema: "katana-completed-frame-resource-receipt-v1",
            run_id: &self.run_id,
            pid: std::process::id(),
            seq: sequence,
            snapshot: &snapshot,
            snapshot_elapsed_us,
        }) {
            Ok(payload) => payload,
            Err(error) => return self.fail(format!("receipt serialization failed: {error}")),
        };
        let serialize_us = serialized_at.elapsed().as_micros();
        let written_at = Instant::now();
        if let Err(error) = self.publish(sequence, &payload) {
            self.fail(error);
            return;
        }
        self.sequence = sequence;
        self.last_snapshot = Some(snapshot);
        DebugLog::write(
            "document_resource_receipt",
            format_args!(
                "seq={sequence} snapshot_us={snapshot_elapsed_us} serialize_us={serialize_us} write_us={}",
                written_at.elapsed().as_micros()
            ),
        );
    }

    pub(crate) fn begin_capture(&mut self) -> bool {
        if self.failed {
            return false;
        }
        let now = Instant::now();
        if self
            .last_capture_at
            .is_some_and(|previous| now.duration_since(previous) < RECEIPT_INTERVAL)
        {
            return false;
        }
        self.last_capture_at = Some(now);
        true
    }

    fn publish(&self, sequence: u64, payload: &[u8]) -> Result<(), String> {
        let nonce = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = self.directory.join(format!(
            ".receipt-{}-{sequence}-{nonce}.tmp",
            std::process::id()
        ));
        let latest = self.directory.join("latest.json");
        if fs::symlink_metadata(&latest).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err("receipt latest path must not be a symlink".to_owned());
        }
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options
            .open(&temporary)
            .map_err(|error| format!("receipt temporary create failed: {error}"))?;
        file.write_all(payload)
            .map_err(|error| format!("receipt temporary write failed: {error}"))?;
        file.write_all(b"\n")
            .map_err(|error| format!("receipt temporary newline failed: {error}"))?;
        drop(file);
        /* WHY: Windows の読者が latest.json を保持中なら置換を強行せず、計測失敗として外部受入を拒否する。 */
        fs::rename(&temporary, latest)
            .map_err(|error| format!("receipt atomic replace failed: {error}"))
    }

    fn fail(&mut self, error: String) {
        self.failed = true;
        DebugLog::write("document_resource_receipt_error", format_args!("{error}"));
    }
}

#[cfg(test)]
#[path = "resource_receipt_tests.rs"]
mod tests;

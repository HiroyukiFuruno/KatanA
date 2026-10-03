use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use super::types::{FONT_READ_CHUNK_BYTES, FontFaceResolutionDiagnostic, MAX_FONT_PAYLOAD_BYTES};

pub(super) fn read_candidate(
    path: &Path,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, FontFaceResolutionDiagnostic> {
    let (file, size) = open_candidate(path, cancelled)?;
    read_payload(file, path, size, cancelled)
}

fn open_candidate(
    path: &Path,
    cancelled: &AtomicBool,
) -> Result<(File, u64), FontFaceResolutionDiagnostic> {
    check_cancelled(cancelled)?;
    let metadata = fs::metadata(path).map_err(|error| unreadable(path, error))?;
    check_cancelled(cancelled)?;
    if !metadata.is_file() {
        return Err(FontFaceResolutionDiagnostic::NonRegularFile {
            path: path.to_owned(),
        });
    }
    check_size(path, metadata.len())?;
    check_cancelled(cancelled)?;
    let file = File::open(path).map_err(|error| unreadable(path, error))?;
    Ok((file, metadata.len()))
}

fn read_payload(
    mut file: File,
    path: &Path,
    size: u64,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, FontFaceResolutionDiagnostic> {
    let mut payload = Vec::with_capacity(size as usize);
    let mut chunk = vec![0; FONT_READ_CHUNK_BYTES];
    loop {
        check_cancelled(cancelled)?;
        if payload.len() as u64 == MAX_FONT_PAYLOAD_BYTES {
            if has_extra_byte(&mut file, path)? {
                return Err(FontFaceResolutionDiagnostic::FileTooLarge {
                    path: path.to_owned(),
                    bytes: MAX_FONT_PAYLOAD_BYTES + 1,
                    limit: MAX_FONT_PAYLOAD_BYTES,
                });
            }
            break;
        }
        let remaining = (MAX_FONT_PAYLOAD_BYTES - payload.len() as u64) as usize;
        let read_limit = chunk.len().min(remaining);
        let read = file
            .read(&mut chunk[..read_limit])
            .map_err(|error| unreadable(path, error))?;
        if read == 0 {
            break;
        }
        payload.extend_from_slice(&chunk[..read]);
    }
    Ok(payload)
}

fn has_extra_byte(file: &mut File, path: &Path) -> Result<bool, FontFaceResolutionDiagnostic> {
    let mut extra = [0_u8; 1];
    file.read(&mut extra)
        .map(|count| count > 0)
        .map_err(|error| unreadable(path, error))
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<(), FontFaceResolutionDiagnostic> {
    if cancelled.load(Ordering::Acquire) {
        Err(FontFaceResolutionDiagnostic::Cancelled)
    } else {
        Ok(())
    }
}

fn check_size(path: &Path, bytes: u64) -> Result<(), FontFaceResolutionDiagnostic> {
    if bytes > MAX_FONT_PAYLOAD_BYTES {
        return Err(FontFaceResolutionDiagnostic::FileTooLarge {
            path: path.to_owned(),
            bytes,
            limit: MAX_FONT_PAYLOAD_BYTES,
        });
    }
    Ok(())
}

fn unreadable(path: &Path, error: std::io::Error) -> FontFaceResolutionDiagnostic {
    FontFaceResolutionDiagnostic::FileUnreadable {
        path: PathBuf::from(path),
        kind: error.kind(),
    }
}

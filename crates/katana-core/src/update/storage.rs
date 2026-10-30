use super::super::CheckUpdateError;
use std::path::Path;

pub(in crate::update) struct UpdateStorageOps;

impl UpdateStorageOps {
    const PREPARATION_RESERVE_BYTES: u64 = 64 * 1024 * 1024;
    const STAGING_AND_COPY_COUNT: u64 = 2;

    pub(in crate::update) fn check_download(destination: &Path, bytes: u64) -> anyhow::Result<()> {
        let directory = destination
            .parent()
            .ok_or_else(|| anyhow::anyhow!("update download has no parent directory"))?;
        Self::ensure_capacity(directory, Self::with_reserve(bytes)?)
    }

    pub(in crate::update) fn check_archive(archive: &Path, target: &Path) -> anyhow::Result<()> {
        let expanded = Self::expanded_bytes(archive)?;
        /* WHY: 展開済みファイルと置換時のコピーが同時に残る場合も容量予算へ含める。 */
        let staging = expanded
            .checked_mul(Self::STAGING_AND_COPY_COUNT)
            .ok_or_else(|| anyhow::anyhow!("update storage size overflow"))?;
        let directory = archive
            .parent()
            .ok_or_else(|| anyhow::anyhow!("update archive has no parent directory"))?;
        let archive_bytes = std::fs::metadata(archive)?.len();
        Self::ensure_capacity(directory, Self::with_reserve(staging)?)
            .map_err(|error| Self::include_released_archive(error, archive_bytes))?;
        let target_directory = target
            .parent()
            .ok_or_else(|| anyhow::anyhow!("update target has no parent directory"))?;
        Self::ensure_capacity(target_directory, Self::with_reserve(expanded)?)
    }

    fn with_reserve(bytes: u64) -> anyhow::Result<u64> {
        bytes
            .checked_add(Self::PREPARATION_RESERVE_BYTES)
            .ok_or_else(|| anyhow::anyhow!("update storage size overflow"))
    }

    fn include_released_archive(error: anyhow::Error, archive_bytes: u64) -> anyhow::Error {
        let Some(CheckUpdateError::InsufficientStorage {
            required_bytes,
            available_bytes,
        }) = error.downcast_ref::<CheckUpdateError>()
        else {
            return error;
        };
        /* WHY: 失敗後に一時アーカイブが消えるため、再試行前と同じ容量基準で案内する。 */
        let (Some(required_bytes), Some(available_bytes)) = (
            required_bytes.checked_add(archive_bytes),
            available_bytes.checked_add(archive_bytes),
        ) else {
            return anyhow::anyhow!("update storage size overflow");
        };
        CheckUpdateError::InsufficientStorage {
            required_bytes,
            available_bytes,
        }
        .into()
    }

    fn ensure_capacity(directory: &Path, required_bytes: u64) -> anyhow::Result<()> {
        let available_bytes = fs4::available_space(directory)?;
        if available_bytes < required_bytes {
            return Err(CheckUpdateError::InsufficientStorage {
                required_bytes,
                available_bytes,
            }
            .into());
        }
        Ok(())
    }

    fn expanded_bytes(path: &Path) -> anyhow::Result<u64> {
        if path.to_string_lossy().ends_with(".tar.gz") {
            let reader = flate2::read::GzDecoder::new(std::fs::File::open(path)?);
            let mut archive = tar::Archive::new(reader);
            let mut total = 0_u64;
            for entry in archive.entries()? {
                total = total
                    .checked_add(entry?.size())
                    .ok_or_else(|| anyhow::anyhow!("update storage size overflow"))?;
            }
            return Ok(total);
        }
        let mut archive = zip::ZipArchive::new(std::fs::File::open(path)?)?;
        let mut total = 0_u64;
        for index in 0..archive.len() {
            total = total
                .checked_add(archive.by_index(index)?.size())
                .ok_or_else(|| anyhow::anyhow!("update storage size overflow"))?;
        }
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_check_uses_real_filesystem_and_preserves_numbers() {
        let directory = tempfile::tempdir().unwrap();
        let error = UpdateStorageOps::ensure_capacity(directory.path(), u64::MAX).unwrap_err();
        assert!(
            matches!(error.downcast_ref::<CheckUpdateError>(), Some(CheckUpdateError::InsufficientStorage { required_bytes: u64::MAX, available_bytes }) if *available_bytes < u64::MAX)
        );
        assert!(UpdateStorageOps::ensure_capacity(directory.path(), 0).is_ok());
    }

    #[test]
    fn required_capacity_cannot_wrap() {
        assert!(UpdateStorageOps::with_reserve(u64::MAX).is_err());
    }

    #[test]
    fn download_preflight_rejects_capacity_before_creating_destination() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("update.zip");
        let error = UpdateStorageOps::check_download(
            &path,
            u64::MAX - UpdateStorageOps::PREPARATION_RESERVE_BYTES,
        )
        .unwrap_err();
        assert!(matches!(
            error.downcast_ref::<CheckUpdateError>(),
            Some(CheckUpdateError::InsufficientStorage {
                required_bytes: u64::MAX,
                ..
            })
        ));
        assert!(!path.exists());
    }

    #[test]
    fn staging_error_reports_capacity_before_the_retry_download() {
        let error = CheckUpdateError::InsufficientStorage {
            required_bytes: 1200,
            available_bytes: 800,
        };
        let actual = UpdateStorageOps::include_released_archive(error.into(), 500);
        assert_eq!(
            actual.downcast_ref::<CheckUpdateError>(),
            Some(&CheckUpdateError::InsufficientStorage {
                required_bytes: 1700,
                available_bytes: 1300
            })
        );
    }

    #[test]
    fn zip_budget_uses_uncompressed_payload_size() {
        use std::io::Write;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("update.zip");
        let mut writer = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        writer
            .start_file("app", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"application").unwrap();
        writer
            .start_file("worker", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"sidecar").unwrap();
        writer.finish().unwrap();
        assert_eq!(UpdateStorageOps::expanded_bytes(&path).unwrap(), 18);
    }

    #[test]
    fn tar_gzip_budget_uses_uncompressed_payload_size() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("update.tar.gz");
        let encoder = flate2::write::GzEncoder::new(
            std::fs::File::create(&path).unwrap(),
            flate2::Compression::default(),
        );
        let mut writer = tar::Builder::new(encoder);
        let data = b"application";
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        writer.append_data(&mut header, "app", &data[..]).unwrap();
        writer.into_inner().unwrap().finish().unwrap();
        assert_eq!(
            UpdateStorageOps::expanded_bytes(&path).unwrap(),
            data.len() as u64
        );
    }
}

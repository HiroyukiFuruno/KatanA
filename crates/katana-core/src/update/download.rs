use super::types::UpdateDownloadOps;
use crate::update::UpdateProgress;

#[path = "download_archive_security.rs"]
mod download_archive_security;

impl UpdateDownloadOps {
    pub fn download_update<P: AsRef<std::path::Path>, F>(
        url: &str,
        dest_path: P,
        mut on_progress: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(UpdateProgress),
    {
        let response = ureq::get(url)
            .header("User-Agent", concat!("KatanA/", env!("CARGO_PKG_VERSION")))
            .call()?;

        let total_size = response
            .headers()
            .get("Content-Length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        let mut reader = response.into_body().into_reader();
        let mut out_file = std::fs::File::create(dest_path)?;

        const DOWNLOAD_BUFFER_SIZE: usize = 65536;
        let mut buffer = [0; DOWNLOAD_BUFFER_SIZE];
        let mut downloaded = 0;

        loop {
            use std::io::Read;
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            use std::io::Write;
            out_file.write_all(&buffer[..bytes_read])?;
            downloaded += bytes_read as u64;
            on_progress(UpdateProgress::Downloading {
                downloaded,
                total: total_size,
            });
        }

        Ok(())
    }

    pub fn extract_update<P: AsRef<std::path::Path>, D: AsRef<std::path::Path>, F>(
        archive_path: P,
        extract_to_dir: D,
        on_progress: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(UpdateProgress),
    {
        let path = archive_path.as_ref();
        if is_tar_gz(path) {
            return Self::extract_tar_gz(path, extract_to_dir, on_progress);
        }
        Self::extract_zip(path, extract_to_dir, on_progress)
    }

    fn extract_tar_gz<P: AsRef<std::path::Path>, D: AsRef<std::path::Path>, F>(
        path: P,
        extract_to_dir: D,
        mut on_progress: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(UpdateProgress),
    {
        let tar_gz = std::fs::File::open(path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);

        /* WHY: tar crate doesn't provide a direct total count without reading the whole stream.
        We skip counting to maintain performance and avoid double-pass. */
        archive.unpack(extract_to_dir)?;

        /* WHY: Notify 100% progress after full extraction since we can meassure it incrementally without overhead. */
        on_progress(UpdateProgress::Extracting {
            current: 1,
            total: 1,
        });
        Ok(())
    }

    fn extract_zip<P: AsRef<std::path::Path>, D: AsRef<std::path::Path>, F>(
        path: P,
        extract_to_dir: D,
        mut on_progress: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(UpdateProgress),
    {
        let mut archive = zip::ZipArchive::new(std::fs::File::open(path)?)?;
        let total = archive.len();

        for i in 0..total {
            let mut file = archive.by_index(i)?;
            let entry_name = file.name()?.into_owned();
            if download_archive_security::is_zip_symlink(&file) {
                anyhow::bail!(
                    "ZIP archive contains unsupported symlink entry: {}",
                    entry_name
                );
            }
            let Some(rel_path) = file.enclosed_name() else {
                continue;
            };
            let rel_path = rel_path.to_path_buf();
            let is_directory = entry_name.ends_with('/');
            extract_zip_entry(&mut file, is_directory, &rel_path, extract_to_dir.as_ref())?;

            on_progress(UpdateProgress::Extracting {
                current: i + 1,
                total,
            });
        }
        Ok(())
    }
}

fn extract_zip_entry(
    file: &mut zip::read::ZipFile<'_, impl std::io::Read + ?Sized>,
    is_directory: bool,
    rel_path: &std::path::Path,
    extract_to_dir: &std::path::Path,
) -> anyhow::Result<()> {
    let outpath = extract_to_dir.join(rel_path);
    if is_directory {
        download_archive_security::ensure_output_directory(&outpath, extract_to_dir)?;
    } else {
        if let Some(parent) = outpath.parent() {
            download_archive_security::ensure_output_directory(parent, extract_to_dir)?;
        }
        download_archive_security::reject_existing_symlink(&outpath)?;
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&outpath)?;
        std::io::copy(file, &mut output)?;
    }
    download_archive_security::apply_unix_permissions_zip(file, &outpath)
}

fn is_tar_gz(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.ends_with(".tar.gz"))
        .unwrap_or(false)
}

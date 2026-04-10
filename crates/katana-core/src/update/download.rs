use super::types::UpdateDownloadOps;
use crate::update::UpdateProgress;

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
        mut on_progress: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(UpdateProgress),
    {
        let path = archive_path.as_ref();
        let is_tar_gz = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.ends_with(".tar.gz"))
            .unwrap_or(false);

        if is_tar_gz {
            let tar_gz = std::fs::File::open(path)?;
            let tar = flate2::read::GzDecoder::new(tar_gz);
            let mut archive = tar::Archive::new(tar);

            /* WHY: tar crate doesn't provide a direct total count without reading the whole stream.
               We can estimate progress or just report 'Extracting...'. For now, we'll just extract. */
            archive.unpack(extract_to_dir)?;
            
            // Progress report (100% since we can't easily count indexed entries in tar without double-pass)
            on_progress(UpdateProgress::Extracting {
                current: 1,
                total: 1,
            });
        } else {
            let mut archive = zip::ZipArchive::new(std::fs::File::open(path)?)?;
            let total = archive.len();

            for i in 0..total {
                let mut file = archive.by_index(i)?;
                let Some(path) = file.enclosed_name() else {
                    continue;
                };
                let outpath = extract_to_dir.as_ref().join(path);

                if (*file.name()).ends_with('/') {
                    std::fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(p) = outpath.parent() {
                        std::fs::create_dir_all(p)?;
                    }
                    std::io::copy(&mut file, &mut std::fs::File::create(&outpath)?)?;
                }

                #[cfg(unix)]
                apply_unix_permissions_zip(&file, &outpath)?;
                on_progress(UpdateProgress::Extracting {
                    current: i + 1,
                    total,
                });
            }
        }
        Ok(())
    }
}

#[cfg(unix)]
fn apply_unix_permissions_zip(
    file: &zip::read::ZipFile<'_, impl std::io::Read + ?Sized>,
    outpath: &std::path::Path,
) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if let Some(mode) = file.unix_mode() {
        std::fs::set_permissions(outpath, std::fs::Permissions::from_mode(mode))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_update() {
        use std::io::Write;
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{}/update.zip", port);

        thread::spawn(move || {
            use std::io::Read;
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buf = [0; 1024];
            let _ = stream.read(&mut buf);

            let body = b"mock zip payload";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.write_all(body);
        });

        let temp_dir = tempfile::tempdir().unwrap();
        let dest = temp_dir.path().join("update.zip");

        UpdateDownloadOps::download_update(&url, &dest, |_| {}).unwrap();
        assert!(dest.exists());
        assert_eq!(std::fs::read(&dest).unwrap(), b"mock zip payload");
    }

    #[test]
    fn test_extract_update() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let zip_path = temp_dir.path().join("test.zip");
        let extract_dir = temp_dir.path().join("extracted");

        {
            let file = std::fs::File::create(&zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            zip.start_file("hello.txt", options).unwrap();
            zip.write_all(b"Hello from ZIP").unwrap();
            zip.add_directory("somedir/", options).unwrap();
            zip.start_file("../outside.txt", options).unwrap();
            zip.write_all(b"Should be skipped").unwrap();
            zip.finish().unwrap();
        }

        UpdateDownloadOps::extract_update(&zip_path, &extract_dir, |_| {}).unwrap();

        let extracted_file = extract_dir.join("hello.txt");
        assert!(extracted_file.exists());
        assert_eq!(std::fs::read(&extracted_file).unwrap(), b"Hello from ZIP");
        assert!(extract_dir.join("somedir").is_dir());
    }
}

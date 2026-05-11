use std::path::{Path, PathBuf};

#[cfg(unix)]
const EXECUTABLE_PERMISSION_BITS: u32 = 0o111;
#[cfg(any(target_os = "windows", test))]
pub(super) const WINDOWS_EXECUTABLE_NAME: &str = "KatanA.exe";
#[cfg(any(target_os = "linux", all(test, unix)))]
pub(super) const LINUX_EXECUTABLE_NAME: &str = "KatanA";

#[derive(Clone, Copy)]
pub(super) enum ExtractedFileFallback {
    #[cfg(any(target_os = "windows", test))]
    RegularFile,
    #[cfg(any(target_os = "linux", all(test, unix)))]
    ExecutableRegularFile,
}

impl ExtractedFileFallback {
    fn requires_executable_bit(self) -> bool {
        match self {
            #[cfg(any(target_os = "windows", test))]
            Self::RegularFile => false,
            #[cfg(any(target_os = "linux", all(test, unix)))]
            Self::ExecutableRegularFile => true,
        }
    }
}

pub(super) fn resolve_extracted_file(
    extract_dir: &Path,
    expected_file_name: &str,
    fallback: ExtractedFileFallback,
) -> anyhow::Result<PathBuf> {
    let expected_path = extract_dir.join(expected_file_name);
    if is_regular_file(&expected_path)? {
        return Ok(expected_path);
    }

    if expected_path.exists() {
        anyhow::bail!(
            "Extracted update expected {expected_file_name} to be a regular file. Directory entries: {}",
            describe_directory_entries(extract_dir)?
        );
    }

    let candidates = extracted_file_candidates(extract_dir, fallback)?;
    match candidates.as_slice() {
        [candidate] => Ok(candidate.clone()),
        [] => anyhow::bail!(
            "Extracted update does not contain a valid executable named {expected_file_name}. Directory entries: {}",
            describe_directory_entries(extract_dir)?
        ),
        _ => anyhow::bail!(
            "Extracted update contains multiple executable candidates for {expected_file_name}. Directory entries: {}",
            describe_directory_entries(extract_dir)?
        ),
    }
}

fn extracted_file_candidates(
    extract_dir: &Path,
    fallback: ExtractedFileFallback,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut candidates = Vec::new();
    for entry in std::fs::read_dir(extract_dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_file() {
            continue;
        }

        let path = entry.path();
        if fallback.requires_executable_bit() && !has_executable_bit(&path)? {
            continue;
        }
        candidates.push(path);
    }
    candidates.sort();
    Ok(candidates)
}

fn is_regular_file(path: &Path) -> anyhow::Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.file_type().is_file()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn has_executable_bit(path: &Path) -> anyhow::Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        Ok(std::fs::metadata(path)?.permissions().mode() & EXECUTABLE_PERMISSION_BITS != 0)
    }

    #[cfg(not(unix))]
    {
        let _path = path;
        Ok(true)
    }
}

fn describe_directory_entries(dir: &Path) -> anyhow::Result<String> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let kind = describe_file_type(&file_type);
        entries.push(format!("{} ({kind})", entry.file_name().to_string_lossy()));
    }
    entries.sort();

    if entries.is_empty() {
        return Ok("<empty>".to_owned());
    }

    Ok(entries.join(", "))
}

fn describe_file_type(file_type: &std::fs::FileType) -> &'static str {
    if file_type.is_dir() {
        return "dir";
    }
    if file_type.is_file() {
        return "file";
    }
    if file_type.is_symlink() {
        return "symlink";
    }

    "other"
}

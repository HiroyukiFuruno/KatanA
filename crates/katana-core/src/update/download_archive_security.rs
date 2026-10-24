use std::path::Path;

const ZIP_FILE_TYPE_MASK: u32 = 0o170000;
const ZIP_SYMLINK_TYPE: u32 = 0o120000;
#[cfg(unix)]
const UNIX_PERMISSION_MASK: u32 = 0o777;

pub(super) fn is_zip_symlink(file: &zip::read::ZipFile<'_, impl std::io::Read + ?Sized>) -> bool {
    file.unix_mode()
        .is_some_and(|mode| mode & ZIP_FILE_TYPE_MASK == ZIP_SYMLINK_TYPE)
}

pub(super) fn ensure_output_directory(path: &Path, root: &Path) -> anyhow::Result<()> {
    if !root.exists() {
        std::fs::create_dir_all(root)?;
    }
    reject_symlink_directory(root)?;
    if path == root {
        return Ok(());
    }

    let relative = path
        .strip_prefix(root)
        .map_err(|_| anyhow::anyhow!("ZIP output path escapes extraction root"))?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let std::path::Component::Normal(name) = component else {
            anyhow::bail!("ZIP output path contains an unsafe component");
        };
        current.push(name);
        if !current.exists() {
            std::fs::create_dir(&current)?;
        }
        reject_symlink_directory(&current)?;
    }
    Ok(())
}

fn reject_symlink_directory(path: &Path) -> anyhow::Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        anyhow::bail!(
            "ZIP extraction refuses symlink output component: {}",
            path.display()
        );
    }
    if !metadata.is_dir() {
        anyhow::bail!(
            "ZIP extraction output component is not a directory: {}",
            path.display()
        );
    }
    Ok(())
}

pub(super) fn reject_existing_symlink(path: &Path) -> anyhow::Result<()> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() {
        anyhow::bail!(
            "ZIP extraction refuses symlink output path: {}",
            path.display()
        );
    }
    if !metadata.is_file() {
        anyhow::bail!(
            "ZIP extraction output path is not a regular file: {}",
            path.display()
        );
    }
    Ok(())
}

#[cfg(unix)]
pub(super) fn apply_unix_permissions_zip(
    file: &zip::read::ZipFile<'_, impl std::io::Read + ?Sized>,
    outpath: &Path,
) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if let Some(mode) = file.unix_mode() {
        std::fs::set_permissions(
            outpath,
            std::fs::Permissions::from_mode(mode & UNIX_PERMISSION_MASK),
        )?;
    }
    Ok(())
}

#[cfg(not(unix))]
pub(super) fn apply_unix_permissions_zip(
    _file: &zip::read::ZipFile<'_, impl std::io::Read + ?Sized>,
    _outpath: &Path,
) -> anyhow::Result<()> {
    Ok(())
}

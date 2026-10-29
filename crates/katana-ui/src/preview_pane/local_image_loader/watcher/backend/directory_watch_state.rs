use notify::event::RemoveKind;
use notify::{Event, EventKind};
use std::fs::Metadata;
use std::io;
use std::path::Path;

pub(super) struct DirectoryMetadata;

impl DirectoryMetadata {
    pub(super) fn validate(path: &Path) -> notify::Result<()> {
        let metadata = std::fs::metadata(path)
            .map_err(|error| notify::Error::io(error).add_path(path.to_path_buf()))?;
        if !metadata.is_dir() {
            return Err(
                notify::Error::io(io::Error::from(io::ErrorKind::NotADirectory))
                    .add_path(path.to_path_buf()),
            );
        }
        Ok(())
    }

    pub(super) fn sample(path: &Path) -> Option<notify::Result<Event>> {
        /* WHY: 画像bytesはContentWatchが監視するため、親や子を列挙しない。 */
        Self::classify(path, std::fs::metadata(path))
    }

    pub(super) fn classify(
        path: &Path,
        metadata: io::Result<Metadata>,
    ) -> Option<notify::Result<Event>> {
        match metadata {
            Ok(metadata) if metadata.is_dir() => None,
            Ok(_) => Some(Ok(Self::removed(path))),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) =>
            {
                Some(Ok(Self::removed(path)))
            }
            Err(error) => Some(Err(notify::Error::io(error).add_path(path.to_path_buf()))),
        }
    }

    fn removed(path: &Path) -> Event {
        Event::new(EventKind::Remove(RemoveKind::Folder)).add_path(path.to_path_buf())
    }
}

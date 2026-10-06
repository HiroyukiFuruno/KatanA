use super::{Inner, WatchEvent, WatchRequest};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Weak;

pub(super) struct WatchTarget {
    pub(super) owner: Weak<Inner>,
    pub(super) generation: u64,
    pub(super) requested_path: PathBuf,
}

pub(super) type Targets = HashMap<PathBuf, Vec<WatchTarget>>;

pub(super) struct Registration;

impl Registration {
    pub(super) fn paths(path: &Path) -> Result<Vec<PathBuf>, String> {
        let name = path.file_name().ok_or("image path has no file name")?;
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty());
        let parent = std::fs::canonicalize(parent.unwrap_or_else(|| Path::new(".")))
            .map_err(|error| format!("image watch parent unavailable: {error}"))?;
        let alias = parent.join(name);
        match std::fs::canonicalize(&alias) {
            Ok(resolved) if resolved != alias => Ok(vec![alias, resolved]),
            Ok(_) => Ok(vec![alias]),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(vec![alias]),
            Err(error) => Err(format!("image watch path unavailable: {error}")),
        }
    }

    pub(super) fn register(
        request: WatchRequest,
        watcher: &mut RecommendedWatcher,
        watched_dirs: &mut HashSet<PathBuf>,
        targets: &mut Targets,
    ) {
        let Some(owner) = request.owner.upgrade() else {
            return;
        };
        if owner.generation.load(std::sync::atomic::Ordering::Acquire) != request.generation {
            return;
        }
        let result = Self::install(&request, watcher, watched_dirs, targets);
        let event = match result {
            Ok(()) => WatchEvent::Registered(request.path, request.generation),
            Err(error) => WatchEvent::Failed(request.path, error, request.generation),
        };
        super::events::send_event(&owner, event);
    }

    fn install(
        request: &WatchRequest,
        watcher: &mut RecommendedWatcher,
        watched_dirs: &mut HashSet<PathBuf>,
        targets: &mut Targets,
    ) -> Result<(), String> {
        let paths = Self::paths(&request.path)?;
        for path in &paths {
            let parent = path.parent().ok_or("image watch path has no parent")?;
            if watched_dirs.contains(parent) {
                continue;
            }
            Self::watch_directory(watcher, parent, targets)?;
            watched_dirs.insert(parent.to_path_buf());
        }
        Self::remove_previous(request, targets);
        for path in paths {
            targets.entry(path).or_default().push(WatchTarget {
                owner: request.owner.clone(),
                generation: request.generation,
                requested_path: request.path.clone(),
            });
        }
        super::events::unwatch_unused_dirs(watcher, watched_dirs, targets);
        Ok(())
    }

    fn watch_directory(
        watcher: &mut RecommendedWatcher,
        parent: &Path,
        targets: &Targets,
    ) -> Result<(), String> {
        let result = watcher.watch(parent, RecursiveMode::NonRecursive);
        /* WHY: 監視先変更時にbackendが破棄した通知を補うため、既存画像を非同期で再確認する。 */
        super::events::notify_overflow(targets);
        result.map_err(|error| format!("image watch registration failed: {error}"))
    }

    fn remove_previous(request: &WatchRequest, targets: &mut Targets) {
        targets.retain(|_, owners| {
            owners.retain(|target| {
                !target.owner.ptr_eq(&request.owner) || target.requested_path != request.path
            });
            !owners.is_empty()
        });
    }
}

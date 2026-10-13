use katana_core::workspace::TreeEntry;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(crate) struct ScanContext<'a> {
    pub ignored_directories: &'a [String],
    pub max_depth: usize,
    pub visible_extensions: &'a [String],
    pub extensionless_excludes: &'a [String],
    pub cancel_token: &'a std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub in_memory_dirs: &'a std::collections::HashSet<PathBuf>,
}

pub(crate) struct ScannerOps;

impl ScannerOps {
    fn process_file(path: PathBuf, file_name: &str, ctx: ScanContext<'_>) -> Option<TreeEntry> {
        let is_visible = match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => ctx
                .visible_extensions
                .iter()
                .any(|v| v.eq_ignore_ascii_case(ext)),
            None => {
                let no_ext_enabled = ctx.visible_extensions.iter().any(|v| v.is_empty());
                no_ext_enabled
                    && !ctx
                        .extensionless_excludes
                        .iter()
                        .any(|excl| excl == file_name)
            }
        };
        if is_visible {
            Some(TreeEntry::File { path })
        } else {
            None
        }
    }

    fn process_dir(path: PathBuf, current_depth: usize, ctx: ScanContext<'_>) -> Option<TreeEntry> {
        let children =
            Self::scan_directory_internal(&path, ctx, current_depth + 1).unwrap_or_default();
        if Self::has_any_visible(&children, ctx.visible_extensions)
            || ctx.in_memory_dirs.contains(&path)
        {
            Some(TreeEntry::Directory { path, children })
        } else {
            None
        }
    }

    fn process_entry(
        entry: &std::fs::DirEntry,
        current_depth: usize,
        ctx: ScanContext<'_>,
    ) -> Option<TreeEntry> {
        if ctx.cancel_token.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }
        let path = entry.path();
        let file_name_os = entry.file_name();
        let file_name = file_name_os.to_str()?;
        if ctx
            .ignored_directories
            .iter()
            .any(|ignored| ignored == file_name)
        {
            return None;
        }
        if path.is_dir() {
            Self::process_dir(path, current_depth, ctx)
        } else {
            Self::process_file(path, file_name, ctx)
        }
    }

    fn scan_directory_internal(
        dir: &Path,
        ctx: ScanContext<'_>,
        current_depth: usize,
    ) -> std::io::Result<Vec<TreeEntry>> {
        if current_depth >= ctx.max_depth
            || ctx.cancel_token.load(std::sync::atomic::Ordering::Relaxed)
        {
            return Ok(Vec::new());
        }
        use rayon::prelude::*;
        let iter = std::fs::read_dir(dir)?;
        let child_entries: Vec<_> = iter.filter_map(Result::ok).collect();
        let mut entries: Vec<TreeEntry> = child_entries
            .into_par_iter()
            .filter_map(|entry| Self::process_entry(&entry, current_depth, ctx))
            .collect();
        entries.sort_by(Self::compare_entries);
        Ok(entries)
    }

    pub(crate) fn compare_entries(a: &TreeEntry, b: &TreeEntry) -> std::cmp::Ordering {
        match (a, b) {
            (TreeEntry::Directory { .. }, TreeEntry::File { .. }) => std::cmp::Ordering::Less,
            (TreeEntry::File { .. }, TreeEntry::Directory { .. }) => std::cmp::Ordering::Greater,
            (a, b) => {
                compare_path_natural(&a.path().to_string_lossy(), &b.path().to_string_lossy())
            }
        }
    }

    /* WHY: Recursively and in parallel scans a directory, returning a tree containing only visible files. */
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn scan_directory(
        dir: &Path,
        ignored_directories: &[String],
        max_depth: usize,
        current_depth: usize,
        visible_extensions: &[String],
        extensionless_excludes: &[String],
        cancel_token: &std::sync::Arc<std::sync::atomic::AtomicBool>,
        in_memory_dirs: &std::collections::HashSet<PathBuf>,
    ) -> std::io::Result<Vec<TreeEntry>> {
        let ctx = ScanContext {
            ignored_directories,
            max_depth,
            visible_extensions,
            extensionless_excludes,
            cancel_token,
            in_memory_dirs,
        };
        Self::scan_directory_internal(dir, ctx, current_depth)
    }

    /* WHY: Recursively checks if there is at least one visible file in the tree. */
    pub(crate) fn has_any_visible(entries: &[TreeEntry], visible_extensions: &[String]) -> bool {
        entries.iter().any(|e| match e {
            TreeEntry::File { path } => match path.extension().and_then(|ext| ext.to_str()) {
                Some(ext) => visible_extensions
                    .iter()
                    .any(|v| v.eq_ignore_ascii_case(ext)),
                None => visible_extensions.iter().any(|v| v.is_empty()),
            },
            TreeEntry::Directory { children, .. } => {
                Self::has_any_visible(children, visible_extensions)
            }
        })
    }
}

fn compare_path_natural(a_path: &str, b_path: &str) -> std::cmp::Ordering {
    let mut a_chars = a_path.chars().peekable();
    let mut b_chars = b_path.chars().peekable();
    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (Some(&ca), Some(&cb)) => {
                if ca.is_ascii_digit() && cb.is_ascii_digit() {
                    let a_num = parse_leading_num(&mut a_chars);
                    let b_num = parse_leading_num(&mut b_chars);
                    if a_num != b_num {
                        break a_num.cmp(&b_num);
                    }
                } else {
                    if ca != cb {
                        break ca.cmp(&cb);
                    }
                    a_chars.next();
                    b_chars.next();
                }
            }
            (Some(_), None) => break std::cmp::Ordering::Greater,
            (None, Some(_)) => break std::cmp::Ordering::Less,
            (None, None) => break std::cmp::Ordering::Equal,
        }
    }
}

fn parse_leading_num(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> u64 {
    const DECIMAL_BASE: u64 = 10;
    let mut num = 0u64;
    while let Some(&c) = chars.peek() {
        if !c.is_ascii_digit() {
            break;
        }
        num = num * DECIMAL_BASE + (c as u64 - '0' as u64);
        chars.next();
    }
    num
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_has_any_visible_with_empty_extension() {
        let file_with_no_ext = TreeEntry::File {
            path: PathBuf::from("no_extension_file"),
        };
        let file_with_ext = TreeEntry::File {
            path: PathBuf::from("file.md"),
        };

        let visible_exts_without_empty = vec!["md".to_string()];
        assert!(!ScannerOps::has_any_visible(
            std::slice::from_ref(&file_with_no_ext),
            &visible_exts_without_empty
        ));
        assert!(ScannerOps::has_any_visible(
            std::slice::from_ref(&file_with_ext),
            &visible_exts_without_empty
        ));

        let visible_exts_with_empty = vec!["md".to_string(), "".to_string()];
        assert!(ScannerOps::has_any_visible(
            std::slice::from_ref(&file_with_no_ext),
            &visible_exts_with_empty
        ));

        let dir = TreeEntry::Directory {
            path: PathBuf::from("dir"),
            children: vec![file_with_no_ext],
        };
        assert!(!ScannerOps::has_any_visible(
            std::slice::from_ref(&dir),
            &visible_exts_without_empty
        ));
        assert!(ScannerOps::has_any_visible(
            &[dir],
            &visible_exts_with_empty
        ));
    }

    #[test]
    fn test_scan_directory_empty_extension() {
        use std::fs;
        use std::sync::Arc;
        use std::sync::atomic::AtomicBool;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("no_ext_file");
        fs::write(&file_path, "test").unwrap();

        let cancel_token = Arc::new(AtomicBool::new(false));
        let in_memory_dirs = std::collections::HashSet::new();

        let tree_with_empty = ScannerOps::scan_directory(
            dir.path(),
            &[],
            10,
            0,
            &["".to_string()],
            &[],
            &cancel_token,
            &in_memory_dirs,
        )
        .unwrap();

        assert_eq!(tree_with_empty.len(), 1);
        if let TreeEntry::File { path } = &tree_with_empty[0] {
            assert_eq!(path.file_name().unwrap(), "no_ext_file");
        } else {
            panic!("Expected file entry");
        }

        let tree_without_empty = ScannerOps::scan_directory(
            dir.path(),
            &[],
            10,
            0,
            &["md".to_string()],
            &[],
            &cancel_token,
            &in_memory_dirs,
        )
        .unwrap();
        assert!(tree_without_empty.is_empty());
    }

    #[test]
    fn test_natural_sort_ordering() {
        use std::path::PathBuf;
        let mut entries = [
            TreeEntry::File {
                path: PathBuf::from("v0-1-0"),
            },
            TreeEntry::File {
                path: PathBuf::from("v0-10-0"),
            },
            TreeEntry::File {
                path: PathBuf::from("v0-2-0"),
            },
            TreeEntry::File {
                path: PathBuf::from("a"),
            },
            TreeEntry::File {
                path: PathBuf::from("a1"),
            },
            TreeEntry::File {
                path: PathBuf::from("a10"),
            },
            TreeEntry::File {
                path: PathBuf::from("a2"),
            },
        ];
        entries.sort_by(ScannerOps::compare_entries);

        let names: Vec<String> = entries
            .iter()
            .map(|e| e.path().to_string_lossy().to_string())
            .collect();
        assert_eq!(
            names,
            vec!["a", "a1", "a2", "a10", "v0-1-0", "v0-2-0", "v0-10-0"]
        );
    }

    #[test]
    fn test_natural_sort_edge_cases() {
        use std::cmp::Ordering;
        use std::path::PathBuf;

        let a = TreeEntry::File {
            path: PathBuf::from("a"),
        };
        let a_duplicate = TreeEntry::File {
            path: PathBuf::from("a"),
        };
        assert_eq!(
            ScannerOps::compare_entries(&a, &a_duplicate),
            Ordering::Equal
        );

        let a10 = TreeEntry::File {
            path: PathBuf::from("a10"),
        };
        let a1 = TreeEntry::File {
            path: PathBuf::from("a1"),
        };
        assert_eq!(ScannerOps::compare_entries(&a10, &a1), Ordering::Greater);
        assert_eq!(ScannerOps::compare_entries(&a1, &a10), Ordering::Less);

        let ab = TreeEntry::File {
            path: PathBuf::from("ab"),
        };
        assert_eq!(ScannerOps::compare_entries(&a, &ab), Ordering::Less);
        assert_eq!(ScannerOps::compare_entries(&ab, &a), Ordering::Greater);
    }
}

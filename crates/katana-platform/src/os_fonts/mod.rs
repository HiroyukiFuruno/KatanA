use std::fs;
use std::path::Path;
use std::sync::OnceLock;

/* WHY: Service for discovering installed macOS fonts. */

mod types;
pub use types::OsFontScanner;
static SYSTEM_FONTS: OnceLock<Vec<(String, String)>> = OnceLock::new();

impl OsFontScanner {
    /* WHY: Returns a cached list of system fonts to avoid filesystem IO on every frame. */
    pub fn cached_fonts() -> &'static [(String, String)] {
        SYSTEM_FONTS.get_or_init(Self::scan_fonts).as_slice()
    }

    /* WHY: Scans standard macOS font directories for TTF, TTC, and OTF files.

    Returns a list of `(font_name, file_path)`. */
    pub fn scan_fonts() -> Vec<(String, String)> {
        let mut fonts = Vec::new();
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default();
        let mut all_dirs = Vec::new();

        #[cfg(target_os = "macos")]
        {
            all_dirs.push("/System/Library/Fonts".to_string());
            all_dirs.push("/System/Library/Fonts/Supplemental".to_string());
            all_dirs.push("/Library/Fonts".to_string());
            if !home_dir.is_empty() {
                all_dirs.push(format!("{home_dir}/Library/Fonts"));
            }
        }

        #[cfg(target_os = "windows")]
        {
            let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            all_dirs.push(format!("{windir}\\Fonts"));
            let local_app_data = std::env::var("LOCALAPPDATA")
                .unwrap_or_else(|_| format!("{home_dir}\\AppData\\Local"));
            if !home_dir.is_empty() {
                all_dirs.push(format!("{local_app_data}\\Microsoft\\Windows\\Fonts"));
            }
        }

        #[cfg(target_os = "linux")]
        {
            all_dirs.push("/usr/share/fonts".to_string());
            all_dirs.push("/usr/local/share/fonts".to_string());
            if !home_dir.is_empty() {
                all_dirs.push(format!("{home_dir}/.local/share/fonts"));
                all_dirs.push(format!("{home_dir}/.fonts"));
            }
        }

        for dir in all_dirs {
            Self::scan_directory(Path::new(&dir), &mut fonts);
        }

        fonts.sort_by(|a, b| a.0.cmp(&b.0));
        fonts.dedup_by(|a, b| a.0 == b.0);
        fonts
    }

    /* WHY: Recursively unnested scan to adhere to maximum nest rules (2 levels focus). */
    pub fn scan_directory(dir: &Path, fonts: &mut Vec<(String, String)>) {
        let Ok(entries) = fs::read_dir(dir) else {
            /* WHY: Skip directories that do not exist or are not accessible. */
            return;
        };

        for entry in entries.flatten() {
            Self::scan_entry(entry, fonts);
        }
    }

    fn scan_entry(entry: fs::DirEntry, fonts: &mut Vec<(String, String)>) {
        let Ok(file_type) = entry.file_type() else {
            return;
        };
        let path = entry.path();
        if file_type.is_dir() {
            Self::scan_directory(&path, fonts);
        } else if file_type.is_file() {
            Self::append_font_path(&path, fonts);
        } else if file_type.is_symlink() {
            Self::process_entry(&path, fonts);
        }
    }

    /* WHY: Extracted per-entry logic to ensure max 30 lines and 2 block nest rule. */
    pub fn process_entry(path: &Path, fonts: &mut Vec<(String, String)>) {
        if !path.is_file() {
            return;
        }
        Self::append_font_path(path, fonts);
    }

    fn append_font_path(path: &Path, fonts: &mut Vec<(String, String)>) {
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if ext != "ttf" && ext != "ttc" && ext != "otf" {
            /* WHY: Skip files with unsupported extensions. */
            return;
        }

        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let path_str = path.to_string_lossy().to_string();
        fonts.push((name, path_str));
    }
}

#[cfg(test)]
mod tests {
    use super::OsFontScanner;

    #[test]
    fn discovers_fonts_in_nested_directories() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("truetype").join("family");
        std::fs::create_dir_all(&nested).unwrap();
        let font = nested.join("InstalledFont.ttf");
        std::fs::write(&font, []).unwrap();
        std::fs::write(nested.join("readme.txt"), []).unwrap();
        let mut fonts = Vec::new();
        OsFontScanner::scan_directory(root.path(), &mut fonts);
        assert_eq!(
            fonts,
            vec![(
                "InstalledFont".to_owned(),
                font.to_string_lossy().into_owned()
            )]
        );
    }

    #[test]
    #[cfg(unix)]
    fn directory_symlinks_do_not_create_recursive_scan_cycles() {
        let root = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(root.path(), root.path().join("cycle")).unwrap();
        let mut fonts = Vec::new();
        OsFontScanner::scan_directory(root.path(), &mut fonts);
        assert!(fonts.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn file_symlinks_keep_their_candidate_path() {
        let root = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        let font = source.path().join("Source.otf");
        std::fs::write(&font, []).unwrap();
        let alias = root.path().join("Linked.ttf");
        std::os::unix::fs::symlink(&font, &alias).unwrap();
        let mut fonts = Vec::new();
        OsFontScanner::scan_directory(root.path(), &mut fonts);
        assert_eq!(
            fonts,
            vec![("Linked".to_owned(), alias.to_string_lossy().into_owned())]
        );
    }

    #[test]
    #[cfg(unix)]
    fn broken_file_symlinks_are_not_font_candidates() {
        let root = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(
            root.path().join("Missing.otf"),
            root.path().join("Broken.ttf"),
        )
        .unwrap();
        let mut fonts = Vec::new();
        OsFontScanner::scan_directory(root.path(), &mut fonts);
        assert!(fonts.is_empty());
    }
}

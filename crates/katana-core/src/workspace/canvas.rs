use std::path::Path;

use super::TreeEntry;

const CANVAS_EXTENSIONS: &[&str] = &["canvas"];

impl TreeEntry {
    pub fn path_is_canvas(path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("canvas"))
    }

    pub fn is_canvas(&self) -> bool {
        self.is_file() && Self::path_is_canvas(self.path())
    }

    pub fn canvas_extensions() -> &'static [&'static str] {
        CANVAS_EXTENSIONS
    }
}

use std::path::PathBuf;

pub struct MermaidBinaryOps;

impl MermaidBinaryOps {
    pub fn default_install_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".local").join("katana").join("mermaid.min.js"))
    }

    pub fn resolve_mermaid_js() -> PathBuf {
        /* WHY: environment override is useful in CI and local debugging scenarios. */
        #[allow(clippy::single_match)]
        match std::env::var("MERMAID_JS") {
            Ok(path) => return PathBuf::from(path),
            Err(_) => {}
        }

        Self::default_install_path().unwrap_or_else(|| PathBuf::from("mermaid.min.js"))
    }

    pub fn find_mermaid_js() -> Option<PathBuf> {
        let path = Self::resolve_mermaid_js();
        if path.exists() { Some(path) } else { None }
    }
}

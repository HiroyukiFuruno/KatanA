pub(crate) struct MissingRendererAssetsOps;

impl MissingRendererAssetsOps {
    pub(crate) fn with<ResultValue>(action: impl FnOnce() -> ResultValue) -> ResultValue {
        let temp_dir = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("MERMAID_JS", temp_dir.path().join("missing-mermaid.min.js")) };
        unsafe { std::env::set_var("DRAWIO_JS", temp_dir.path().join("missing-drawio.min.js")) };
        unsafe { std::env::set_var("PLANTUML_JAR", temp_dir.path().join("missing-plantuml.jar")) };
        let result = action();
        unsafe { std::env::remove_var("MERMAID_JS") };
        unsafe { std::env::remove_var("DRAWIO_JS") };
        unsafe { std::env::remove_var("PLANTUML_JAR") };
        result
    }
}

use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone)]
pub struct AssetDownloadRequest {
    pub url: String,
    pub dest: PathBuf,
}

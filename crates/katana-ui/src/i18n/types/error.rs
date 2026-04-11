use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessages {
    pub missing_dependency: String,
    pub curl_launch_failed: String,
    pub download_failed: String,
    pub render_error: String,
}

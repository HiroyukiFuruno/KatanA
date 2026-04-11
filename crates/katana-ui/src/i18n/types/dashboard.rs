use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMessages {
    pub welcome_title: String,
    pub welcome_subtitle: String,
    pub start_section: String,
    pub recent_section: String,
    pub no_recent_files: String,
    pub welcome_screen: String,
    pub user_guide: String,
}

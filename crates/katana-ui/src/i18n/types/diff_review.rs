use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReviewMessages {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_split")]
    pub split: String,
    #[serde(default = "default_inline")]
    pub inline: String,
    #[serde(default = "default_before")]
    pub before: String,
    #[serde(default = "default_after")]
    pub after: String,
    #[serde(default = "default_cancel")]
    pub cancel: String,
    #[serde(default = "default_reject_all")]
    pub reject_all: String,
    #[serde(default = "default_apply_fix")]
    pub apply_fix: String,
    #[serde(default = "default_previous_file")]
    pub previous_file: String,
    #[serde(default = "default_next_file")]
    pub next_file: String,
    #[serde(default = "default_file_counter")]
    pub file_counter: String,
    #[serde(default = "default_collapsed_lines")]
    pub collapsed_lines: String,
    #[serde(default = "default_switch_to_split")]
    pub switch_to_split: String,
    #[serde(default = "default_switch_to_inline")]
    pub switch_to_inline: String,
    #[serde(default = "default_content_changed")]
    pub content_changed: String,
    #[serde(default = "default_enter_fullscreen")]
    pub enter_fullscreen: String,
    #[serde(default = "default_exit_fullscreen")]
    pub exit_fullscreen: String,
}

impl Default for DiffReviewMessages {
    fn default() -> Self {
        Self {
            title: default_title(),
            split: default_split(),
            inline: default_inline(),
            before: default_before(),
            after: default_after(),
            cancel: default_cancel(),
            reject_all: default_reject_all(),
            apply_fix: default_apply_fix(),
            previous_file: default_previous_file(),
            next_file: default_next_file(),
            file_counter: default_file_counter(),
            collapsed_lines: default_collapsed_lines(),
            switch_to_split: default_switch_to_split(),
            switch_to_inline: default_switch_to_inline(),
            content_changed: default_content_changed(),
            enter_fullscreen: default_enter_fullscreen(),
            exit_fullscreen: default_exit_fullscreen(),
        }
    }
}

fn default_title() -> String {
    "Review Changes".to_string()
}

fn default_split() -> String {
    "Split".to_string()
}

fn default_inline() -> String {
    "Inline".to_string()
}

fn default_before() -> String {
    "Before".to_string()
}

fn default_after() -> String {
    "After".to_string()
}

fn default_cancel() -> String {
    "Cancel".to_string()
}

fn default_reject_all() -> String {
    "Cancel all".to_string()
}

fn default_apply_fix() -> String {
    "Apply Fix".to_string()
}

fn default_previous_file() -> String {
    "Previous file".to_string()
}

fn default_next_file() -> String {
    "Next file".to_string()
}

fn default_file_counter() -> String {
    "{current}/{total}".to_string()
}

fn default_collapsed_lines() -> String {
    "{count} hidden lines".to_string()
}

fn default_switch_to_split() -> String {
    "Show split diff".to_string()
}

fn default_switch_to_inline() -> String {
    "Show inline diff".to_string()
}

fn default_content_changed() -> String {
    "The file changed after the diff was prepared. Re-run the fix to review the latest content."
        .to_string()
}

fn default_enter_fullscreen() -> String {
    "Enter fullscreen".to_string()
}

fn default_exit_fullscreen() -> String {
    "Exit fullscreen".to_string()
}

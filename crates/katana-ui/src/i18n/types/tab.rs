use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabMessages {
    pub nav_prev: String,
    pub nav_next: String,
    pub close: String,
    pub close_others: String,
    pub close_all: String,
    pub close_right: String,
    pub close_left: String,
    pub pin: String,
    pub unpin: String,
    pub restore_closed: String,
    pub group_name_placeholder: String,
    pub create_group_button: String,
    pub close_group: String,
    pub ungroup: String,
    pub add_tab_to_group: String,
    pub remove_from_group: String,
    pub create_new_group: String,
    pub add_to_group: String,
}

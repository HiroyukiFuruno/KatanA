pub mod app;
pub mod changelog;
pub mod editor;
pub mod fixtures;
pub mod foreground_surface_isolation;
pub mod harness_utils;
pub mod lifecycle;
pub mod overlap_checker;
pub mod preview_pane;
pub mod search;
pub mod settings;
pub mod shell_logic;
pub mod tabs;
pub mod toc;
pub mod toc_split_mode_sync;
pub mod tree_layout;
pub mod workspace;

use std::sync::Mutex;
use std::sync::OnceLock;

/// Global mutex for tests that require serial execution (e.g., environment variable manipulation).
pub static SERIAL_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

#[allow(dead_code)]
pub fn get_serial_test_mutex() -> &'static Mutex<()> {
    SERIAL_TEST_MUTEX.get_or_init(|| Mutex::new(()))
}

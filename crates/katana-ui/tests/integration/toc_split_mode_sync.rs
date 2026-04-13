use eframe::egui;
use katana_core::document::Document;
use std::sync::atomic::AtomicBool;
use katana_ui::app_state::AppAction;
use std::time::Instant;

// We will test that when TOC sets scroll_to_line, the preview tracks correctly.

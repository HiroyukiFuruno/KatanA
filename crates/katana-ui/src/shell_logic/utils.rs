use chrono::{DateTime, Local};

pub struct ShellUtils;

impl ShellUtils {
    pub fn format_modified_time(time: std::time::SystemTime) -> String {
        let dt: DateTime<Local> = time.into();
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    const SPLASH_FADE_START: f32 = 0.8;
    const SPLASH_FADE_DURATION: f32 = 0.2;
    const SPLASH_TOTAL_DURATION_SECS: f32 = 1.5;

    pub fn calculate_splash_opacity(progress: f32) -> f32 {
        if progress < Self::SPLASH_FADE_START {
            1.0
        } else {
            (1.0 - progress) / Self::SPLASH_FADE_DURATION
        }
    }

    pub fn calculate_splash_progress(elapsed: f32, is_loading: bool) -> f32 {
        let mut p = elapsed / Self::SPLASH_TOTAL_DURATION_SECS;
        const PROGRESS_FREEZE_POINT: f32 = 0.79;
        if is_loading && p > PROGRESS_FREEZE_POINT {
            /* WHY: Keep the splash screen active and freeze progress until loading finishes */
            p = PROGRESS_FREEZE_POINT;
        }
        p.clamp(0.0, 1.0)
    }

    pub fn prev_tab_index(idx: usize, count: usize) -> usize {
        if count == 0 {
            0
        } else {
            (idx + count - 1) % count
        }
    }

    pub fn next_tab_index(idx: usize, count: usize) -> usize {
        if count == 0 { 0 } else { (idx + 1) % count }
    }

    const KB_UNIT: u64 = 1024;
    const MB_UNIT: u64 = 1024 * 1024;
    const GB_UNIT: u64 = 1024 * 1024 * 1024;

    pub fn format_file_size(bytes: u64) -> String {
        if bytes < Self::KB_UNIT {
            format!("{} B", bytes)
        } else if bytes < Self::MB_UNIT {
            format!("{:.1} KB", bytes as f64 / Self::KB_UNIT as f64)
        } else if bytes < Self::GB_UNIT {
            format!("{:.1} MB", bytes as f64 / Self::MB_UNIT as f64)
        } else {
            format!("{:.1} GB", bytes as f64 / Self::GB_UNIT as f64)
        }
    }
}

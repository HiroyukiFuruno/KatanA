pub(crate) struct DebugLog;

impl DebugLog {
    pub(crate) fn write(event: &str, details: std::fmt::Arguments<'_>) {
        if enabled_from(std::env::var("DEBUG").ok().as_deref()) {
            eprintln!("[KATANA_DEBUG] event={event} {details}");
        }
    }
}

fn enabled_from(value: Option<&str>) -> bool {
    matches!(value, Some("true"))
}

#[cfg(test)]
mod tests {
    use super::{DebugLog, enabled_from};

    #[test]
    fn debug_enabled_process_emits_formatted_event() {
        const CHILD: &str = "KATANA_DEBUG_LOG_TEST_CHILD";
        if std::env::var_os(CHILD).is_some() {
            DebugLog::write("coverage_probe", format_args!("value={}", 42));
            return;
        }
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "debug_log::tests::debug_enabled_process_emits_formatted_event",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .env("DEBUG", "true")
            .output()
            .unwrap();
        assert!(output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("[KATANA_DEBUG] event=coverage_probe value=42"));
    }

    #[test]
    fn debug_output_requires_the_exact_true_value() {
        assert!(enabled_from(Some("true")));
        for value in [None, Some("TRUE"), Some("1"), Some("false"), Some("")] {
            assert!(!enabled_from(value));
        }
    }
}

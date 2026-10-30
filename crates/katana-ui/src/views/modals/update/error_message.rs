pub(crate) struct UpdateErrorMessageOps;

impl UpdateErrorMessageOps {
    pub(crate) fn for_phase<'a>(
        error: &katana_core::update::CheckUpdateError,
        messages: &'a crate::i18n::UpdateMessages,
        installing: bool,
    ) -> (String, &'a str) {
        if installing {
            (
                Self::localize_install(error, messages),
                &messages.failed_to_install,
            )
        } else {
            (Self::localize(error, messages), &messages.failed_to_check)
        }
    }

    pub(crate) fn localize_install(
        error: &katana_core::update::CheckUpdateError,
        messages: &crate::i18n::UpdateMessages,
    ) -> String {
        match error {
            katana_core::update::CheckUpdateError::InsufficientStorage {
                required_bytes,
                available_bytes,
            } => {
                const BYTES_PER_MEGABYTE: u64 = 1_000_000;
                messages
                    .update_install_error_insufficient_storage
                    .replace(
                        "{required_mb}",
                        &required_bytes.div_ceil(BYTES_PER_MEGABYTE).to_string(),
                    )
                    .replace(
                        "{available_mb}",
                        &(available_bytes / BYTES_PER_MEGABYTE).to_string(),
                    )
            }
            katana_core::update::CheckUpdateError::Other(_) => {
                messages.update_install_error_unknown.clone()
            }
            _ => Self::localize(error, messages),
        }
    }

    pub(crate) fn localize(
        error: &katana_core::update::CheckUpdateError,
        messages: &crate::i18n::UpdateMessages,
    ) -> String {
        let template = Self::template_for(error.i18n_key(), messages);
        match error {
            katana_core::update::CheckUpdateError::ServerStatus(status) => {
                template.replace("{status}", &status.to_string())
            }
            _ => template.to_string(),
        }
    }

    fn template_for<'a>(key: &str, messages: &'a crate::i18n::UpdateMessages) -> &'a str {
        match key {
            "update_check_error_network_unreachable" => {
                &messages.update_check_error_network_unreachable
            }
            "update_check_error_network_timed_out" => {
                &messages.update_check_error_network_timed_out
            }
            "update_check_error_server_status" => &messages.update_check_error_server_status,
            "update_check_error_proxy_failed" => &messages.update_check_error_proxy_failed,
            "update_check_error_invalid_payload" => &messages.update_check_error_invalid_payload,
            "update_check_error_unknown" => &messages.update_check_error_unknown,
            "update_install_error_storage_full" => &messages.update_install_error_storage_full,
            "update_install_error_permission_denied" => {
                &messages.update_install_error_permission_denied
            }
            _ => &messages.update_check_error_unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preparation_and_check_failures_have_distinct_localized_headings() {
        let messages = japanese_messages();
        let error = katana_core::update::CheckUpdateError::Other("technical detail".into());
        let (install_message, install_heading) =
            UpdateErrorMessageOps::for_phase(&error, &messages.update, true);
        let (check_message, check_heading) =
            UpdateErrorMessageOps::for_phase(&error, &messages.update, false);
        assert_eq!(install_heading, messages.update.failed_to_install);
        assert_eq!(check_heading, messages.update.failed_to_check);
        assert_eq!(
            install_message,
            messages.update.update_install_error_unknown
        );
        assert_eq!(check_message, messages.update.update_check_error_unknown);
        assert_ne!(install_heading, check_heading);
    }

    #[test]
    fn install_storage_failure_explains_space_in_japanese() {
        let messages = japanese_messages();
        let text = UpdateErrorMessageOps::localize_install(
            &katana_core::update::CheckUpdateError::StorageFull,
            &messages.update,
        );
        assert_eq!(text, messages.update.update_install_error_storage_full);
        assert!(!text.contains("不明"));
    }

    #[test]
    fn unknown_install_failure_does_not_claim_update_check_failure() {
        let messages = japanese_messages();
        let text = UpdateErrorMessageOps::localize_install(
            &katana_core::update::CheckUpdateError::Other("private technical detail".into()),
            &messages.update,
        );
        assert_eq!(text, messages.update.update_install_error_unknown);
        assert!(!text.contains("private technical detail"));
        assert!(!text.contains("更新確認"));
    }

    #[test]
    fn install_storage_budget_displays_rounded_required_and_available_mb() {
        let messages = japanese_messages();
        let text = UpdateErrorMessageOps::localize_install(
            &katana_core::update::CheckUpdateError::InsufficientStorage {
                required_bytes: 808_738_757,
                available_bytes: 396_361_728,
            },
            &messages.update,
        );
        assert!(text.contains("809 MB"));
        assert!(text.contains("396 MB"));
        assert!(!text.contains("{required_mb}"));
        assert!(!text.contains("{available_mb}"));
    }

    #[test]
    fn localizes_server_status_with_placeholder() {
        let messages = english_messages();

        let text = UpdateErrorMessageOps::localize(
            &katana_core::update::CheckUpdateError::ServerStatus(429),
            &messages.update,
        );

        assert!(text.contains("429"));
        assert!(!text.contains("{status}"));
    }

    #[test]
    fn localizes_network_error_in_japanese() {
        let messages = japanese_messages();

        let text = UpdateErrorMessageOps::localize(
            &katana_core::update::CheckUpdateError::NetworkUnreachable,
            &messages.update,
        );

        assert_eq!(text, messages.update.update_check_error_network_unreachable);
        assert!(!text.contains("io:"));
    }

    #[test]
    fn localizes_all_non_status_variants() {
        let messages = english_messages();
        let cases = [
            (
                katana_core::update::CheckUpdateError::NetworkTimedOut,
                messages
                    .update
                    .update_check_error_network_timed_out
                    .as_str(),
            ),
            (
                katana_core::update::CheckUpdateError::ProxyFailed,
                messages.update.update_check_error_proxy_failed.as_str(),
            ),
            (
                katana_core::update::CheckUpdateError::InvalidPayload,
                messages.update.update_check_error_invalid_payload.as_str(),
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(
                UpdateErrorMessageOps::localize(&error, &messages.update),
                expected
            );
        }
    }

    #[test]
    fn unknown_error_uses_localized_phrase_without_raw_detail() {
        let messages = english_messages();

        let text = UpdateErrorMessageOps::localize(
            &katana_core::update::CheckUpdateError::Other("io: Connection refused".to_string()),
            &messages.update,
        );

        assert_eq!(text, messages.update.update_check_error_unknown);
        assert!(!text.contains("Connection refused"));
    }

    fn english_messages() -> crate::i18n::I18nMessages {
        serde_json::from_str(include_str!("../../../../locales/en.json")).expect("Test requirement")
    }

    fn japanese_messages() -> crate::i18n::I18nMessages {
        serde_json::from_str(include_str!("../../../../locales/ja.json")).expect("Test requirement")
    }
}

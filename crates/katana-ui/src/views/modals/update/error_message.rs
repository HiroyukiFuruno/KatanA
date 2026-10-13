pub(crate) struct UpdateErrorMessageOps;

impl UpdateErrorMessageOps {
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
            _ => &messages.update_check_error_unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

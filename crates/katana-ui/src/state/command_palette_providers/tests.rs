use super::{AppCommandProvider, DocumentOperationSettings, ViewMode};
use crate::app_state::AppAction;
use crate::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteProvider, CommandPaletteResultKind,
};

const CHILD_ENV: &str = "KATANA_PALETTE_PROVIDER_CONTRACT_CHILD";
const TEST_NAME: &str =
    "state::command_palette_providers::tests::app_command_provider_returns_system_actions";
const SUCCESS_MARKER: &str = "KATANA_PALETTE_PROVIDER_CONTRACT_OK";

struct ProviderContractFixture;

impl ProviderContractFixture {
    fn run_in_child() -> std::io::Result<bool> {
        if Self::is_child() {
            return Ok(false);
        }
        /* WHY: 言語を変更する他テストと同時実行しても候補契約が干渉しないよう、同じ実テストだけを別プロセスで実行する。 */
        let executable = std::env::current_exe()?;
        let program = executable.to_str().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "non-UTF-8 test executable")
        })?;
        let output = katana_core::system::ProcessService::create_command(program)
            .args(["--exact", TEST_NAME, "--test-threads=1", "--nocapture"])
            .env(CHILD_ENV, "1")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "{stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            stdout.contains(SUCCESS_MARKER),
            "child contract did not execute: {stdout}"
        );
        Ok(true)
    }

    fn is_child() -> bool {
        let exact = std::env::args_os().any(|argument| argument == "--exact");
        let selected = std::env::args_os().any(|argument| argument == TEST_NAME);
        std::env::var_os(CHILD_ENV).as_deref() == Some(std::ffi::OsStr::new("1"))
            && exact
            && selected
    }

    fn verify_queries() {
        let provider = AppCommandProvider {
            operation_settings: DocumentOperationSettings::evaluate(None, None),
            current_mode: ViewMode::PreviewOnly,
        };
        let results = provider.search("", None, None);
        assert!(!results.is_empty());
        assert_eq!(results[0].kind, CommandPaletteResultKind::RecentOrCommon);
        let results = provider.search("Settings", None, None);
        assert!(!results.is_empty());
        assert_eq!(results[0].kind, CommandPaletteResultKind::Action);
        assert_eq!(results[0].label, "Settings");
        assert!(matches!(
            &results[0].execute_payload,
            CommandPaletteExecutePayload::DispatchAppAction(AppAction::ToggleSettings)
        ));
    }
}

#[test]
fn app_command_provider_returns_system_actions() -> std::io::Result<()> {
    if ProviderContractFixture::run_in_child()? {
        return Ok(());
    }
    crate::i18n::I18nOps::set_language("en");
    ProviderContractFixture::verify_queries();
    println!("{SUCCESS_MARKER}");
    Ok(())
}

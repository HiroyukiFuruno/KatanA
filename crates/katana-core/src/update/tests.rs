use super::*;

#[test]
fn test_is_newer_version() {
    use crate::update::types::UpdateOps;

    assert!(!UpdateOps::is_newer_version("0.22.1", "0.22.1"));

    /* WHY: Verify that Katana's custom '-X' hotfix suffix is treated properly (Base is checked first, then hotfix value) */
    assert!(!UpdateOps::is_newer_version("v0.22.1-1", "v0.22.1"));
    assert!(UpdateOps::is_newer_version("0.22.1", "0.22.1-1"));
    assert!(UpdateOps::is_newer_version("v0.22.1-1", "v0.22.1-2"));
    assert!(!UpdateOps::is_newer_version("0.22.1-2", "v0.22.1-1"));

    /* WHY: Verify standard SemVer bumps are preferred over matching base verions */
    assert!(UpdateOps::is_newer_version("0.22.1", "0.22.2"));
    assert!(!UpdateOps::is_newer_version("0.22.2", "0.22.1"));

    /* WHY: Verify edge cases where hotfix suffix does not override a base patch update */
    assert!(UpdateOps::is_newer_version("0.22.1-5", "0.22.2"));
}

#[test]
fn test_update_manager_and_state() {
    let target = std::path::PathBuf::from("/Applications/KatanA.app");
    let mut manager = UpdateManager::new("0.6.4".to_string(), target.clone());

    assert_eq!(manager.current_version, "0.6.4");
    assert_eq!(manager.target_app_path, target);
    assert!(matches!(manager.state, UpdateState::Idle));

    assert!(manager.should_check_for_updates());

    manager.set_api_url_override("http://localhost".to_string());
    assert_eq!(
        manager.api_url_override.as_deref(),
        Some("http://localhost")
    );

    manager.set_check_interval(std::time::Duration::from_secs(3600));
    assert_eq!(manager.check_interval, std::time::Duration::from_secs(3600));

    manager.transition_to(UpdateState::Checking);
    assert!(matches!(manager.state, UpdateState::Checking));
    assert!(manager.last_checked.is_some());

    assert!(!manager.should_check_for_updates());

    manager.check_interval = std::time::Duration::from_secs(0);
    manager.last_checked = Some(std::time::Instant::now());
    assert!(manager.should_check_for_updates());

    manager.transition_to(UpdateState::Error("dummy error".to_string()));
    assert!(matches!(manager.state, UpdateState::Error(_)));

    let default_state = UpdateState::default();
    assert!(matches!(default_state, UpdateState::Idle));
}

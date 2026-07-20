use super::*;

#[test]
fn test_parse_changelog_marks_new_versions_open() {
    let md = "# Changelog\n\n## [0.8.0] - 2026-03-28\n### Added\n- Feature A\n\n## [0.7.0] - 2026-02-01\n### Fixed\n- Bug B";

    let sections = ChangelogOps::parse_changelog(md, "0.8.0", Some("0.7.0"));

    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].version, "0.8.0");
    assert!(sections[0].default_open);
    assert_eq!(sections[1].version, "0.7.0");
    assert!(!sections[1].default_open);
}

#[test]
fn test_parse_changelog_unreleased_is_closed() {
    let md = "# Changelog\n\n## [Unreleased]\n### Added\n- Feature\n\n## [v0.8.0] - DATE";
    let sections = ChangelogOps::parse_changelog(md, "0.8.0", Some("0.7.0"));
    assert_eq!(sections[0].version, "Unreleased");
    assert!(!sections[0].default_open);
}

#[test]
fn test_parse_changelog_no_previous_opens_all_up_to_current() {
    let md = "# Changelog\n\n## [0.8.0] - DATE\n### Added\n- A\n\n## [0.7.0] - DATE\n### Fixed\n- B\n\n## [0.6.0] - DATE\n### Changed\n- C";
    let sections = ChangelogOps::parse_changelog(md, "0.8.0", None);

    assert!(sections[0].default_open);
    assert!(sections[1].default_open);
    assert!(sections[2].default_open);
}

#[test]
fn test_parse_changelog_body_extraction() {
    let md = "# Changelog\n\n## [0.8.0] - 2026-03-28\n### Added\n- Feature A\n- Feature B";
    let sections = ChangelogOps::parse_changelog(md, "0.8.0", Some("0.7.0"));
    assert!(sections[0].body.contains("### Added"));
    assert!(sections[0].body.contains("- Feature A"));
    assert!(sections[0].body.contains("- Feature B"));
}

#[test]
fn test_compare_versions() {
    use katana_core::update::types::UpdateOps;
    assert_eq!(UpdateOps::compare_versions("0.8.0", "0.7.0"), 1);
    assert_eq!(UpdateOps::compare_versions("0.7.0", "0.8.0"), -1);
    assert_eq!(UpdateOps::compare_versions("0.8.0", "0.8.0"), 0);
    assert_eq!(UpdateOps::compare_versions("1.0.0", "0.9.9"), 1);
    assert_eq!(UpdateOps::compare_versions("0.8.0.1", "0.8.0"), 1);
}

#[test]
fn test_compare_versions_with_hyphen() {
    use katana_core::update::types::UpdateOps;
    /* WHY: Reproduce issue: "0.8.8-1" should be considered > "0.8.8" in KatanA's versioning (patch increment). */
    assert_eq!(UpdateOps::compare_versions("0.8.8-1", "0.8.8"), 1);
    assert_eq!(UpdateOps::compare_versions("0.8.8", "0.8.8-1"), -1);
    assert_eq!(UpdateOps::compare_versions("0.8.8-2", "0.8.8-1"), 1);
}

#[test]
fn test_is_newer_or_equal() {
    assert!(ChangelogOps::is_newer_or_equal("v0.8.0", "0.7.0"));
    assert!(ChangelogOps::is_newer_or_equal("v0.8.0", "v0.8.0"));
    assert!(!ChangelogOps::is_newer_or_equal("0.7.0", "v0.8.0"));
}

#[test]
fn test_is_older() {
    assert!(ChangelogOps::is_older("0.7.0", "v0.8.0"));
    assert!(!ChangelogOps::is_older("v0.8.0", "v0.8.0"));
    assert!(!ChangelogOps::is_older("0.8.0", "0.7.0"));
}

#[test]
fn test_fetch_changelog_uses_embedded_current_release_notes() {
    let current_version = env!("CARGO_PKG_VERSION");
    let (tx, rx) = std::sync::mpsc::channel();

    ChangelogOps::fetch_changelog("en", current_version.to_string(), None, tx.clone());
    match rx.recv().unwrap() {
        ChangelogEvent::Success(sections) => {
            assert_eq!(sections.first().unwrap().version, current_version);
        }
        ChangelogEvent::Error(error) => panic!("unexpected changelog error: {error}"),
    }

    ChangelogOps::fetch_changelog("ja", current_version.to_string(), None, tx);
    match rx.recv().unwrap() {
        ChangelogEvent::Success(sections) => {
            assert_eq!(sections.first().unwrap().version, current_version);
        }
        ChangelogEvent::Error(error) => panic!("unexpected changelog error: {error}"),
    }
}

#[test]
fn test_render_release_notes_tab_ui() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let sections = vec![ChangelogSection {
                version: "0.8.0".to_string(),
                heading: "v0.8.0".to_string(),
                body: "# Test\n- Item".to_string(),
                default_open: true,
            }];

            ChangelogOps::render_release_notes_tab(ui, &[], true, false);

            ChangelogOps::render_release_notes_tab(ui, &sections, false, false);

            ChangelogOps::render_release_notes_tab(ui, &[], false, false);
        });
    });
}

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
    assert_eq!(ChangelogOps::compare_versions("0.8.0", "0.7.0"), 1);
    assert_eq!(ChangelogOps::compare_versions("0.7.0", "0.8.0"), -1);
    assert_eq!(ChangelogOps::compare_versions("0.8.0", "0.8.0"), 0);
    assert_eq!(ChangelogOps::compare_versions("1.0.0", "0.9.9"), 1);
    assert_eq!(ChangelogOps::compare_versions("0.8.0.1", "0.8.0"), 1);
}

#[test]
fn test_compare_versions_with_hyphen() {
    /* WHY: Reproduce issue: "0.8.8-1" should be considered > "0.8.8" in KatanA's versioning (patch increment). */
    assert_eq!(ChangelogOps::compare_versions("0.8.8-1", "0.8.8"), 1);
    assert_eq!(ChangelogOps::compare_versions("0.8.8", "0.8.8-1"), -1);
    assert_eq!(ChangelogOps::compare_versions("0.8.8-2", "0.8.8-1"), 1);
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
fn test_fetch_changelog_coverage() {
    let (tx, _rx) = std::sync::mpsc::channel();
    ChangelogOps::fetch_changelog("en", "0.8.0".to_string(), None, tx.clone());
    ChangelogOps::fetch_changelog("ja", "0.8.0".to_string(), None, tx);
}

#[test]
fn test_render_release_notes_tab_ui() {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
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

#[test]
fn test_handle_fetch_result_network_error() {
    let (tx, rx) = std::sync::mpsc::channel();
    ChangelogOps::handle_fetch_result(Err("Offline".to_string()), &tx, "0.1.0", None);
    match rx.try_recv().unwrap() {
        ChangelogEvent::Error(e) => assert_eq!(e, "Offline"),
        _ => panic!("Expected Error event"),
    }
}

#[test]
fn test_handle_fetch_result_http_error_with_text() {
    let (tx, rx) = std::sync::mpsc::channel();
    let response = ehttp::Response {
        url: "https://example.com".to_string(),
        ok: false,
        status: 404,
        status_text: "Not Found".to_string(),
        bytes: b"Not Found Data".to_vec(),
        headers: ehttp::Headers::new(&[]),
    };
    ChangelogOps::handle_fetch_result(Ok(response), &tx, "0.1.0", None);
    match rx.try_recv().unwrap() {
        ChangelogEvent::Error(e) => assert_eq!(e, "HTTP error 404: Not Found Data"),
        _ => panic!("Expected Error"),
    }
}

#[test]
fn test_handle_fetch_result_ok_response_decode_error() {
    let (tx, rx) = std::sync::mpsc::channel();
    let response = ehttp::Response {
        url: "https://example.com".to_string(),
        ok: true,
        status: 200,
        status_text: "OK".to_string(),
        bytes: vec![0xFF, 0xFE, 0xFD],
        headers: ehttp::Headers::new(&[]),
    };
    ChangelogOps::handle_fetch_result(Ok(response), &tx, "0.1.0", None);
    match rx.try_recv().unwrap() {
        ChangelogEvent::Error(e) => assert_eq!(e, "Failed to decode response text"),
        _ => panic!("Expected Error"),
    }
}

#[test]
fn test_handle_fetch_result_failure_response_decode_error() {
    let (tx, rx) = std::sync::mpsc::channel();
    let response = ehttp::Response {
        url: "https://example.com".to_string(),
        ok: false,
        status: 500,
        status_text: "Server Error".to_string(),
        bytes: vec![0xFF, 0xFE, 0xFD],
        headers: ehttp::Headers::new(&[]),
    };
    ChangelogOps::handle_fetch_result(Ok(response), &tx, "0.1.0", None);
    match rx.try_recv().unwrap() {
        ChangelogEvent::Error(e) => assert_eq!(e, "HTTP error: 500"),
        _ => panic!("Expected Error"),
    }
}

#[test]
fn test_handle_fetch_result_success() {
    let (tx, rx) = std::sync::mpsc::channel();
    let md = "# Changelog\n## [0.8.0]\n### Added\n- Ok!";
    let response = ehttp::Response {
        url: "https://example.com".to_string(),
        ok: true,
        status: 200,
        status_text: "OK".to_string(),
        bytes: md.as_bytes().to_vec(),
        headers: ehttp::Headers::new(&[]),
    };
    ChangelogOps::handle_fetch_result(Ok(response), &tx, "0.8.0", None);
    match rx.try_recv().unwrap() {
        ChangelogEvent::Success(sections) => {
            assert_eq!(sections.len(), 1);
            assert_eq!(sections[0].version, "0.8.0");
        }
        _ => panic!("Expected Success"),
    }
}

#[test]
fn test_get_changelog_url_cache_busting() {
    let url_en = ChangelogOps::get_changelog_url("en", "0.8.0");
    assert!(
        url_en.starts_with("https://raw.githubusercontent.com/HiroyukiFuruno/KatanA/refs/heads/master/CHANGELOG.md?v=0.8.0&t="),
        "URL {} does not contain expected prefix", url_en
    );

    let url_ja = ChangelogOps::get_changelog_url("ja", "0.8.1-beta");
    assert!(
        url_ja.starts_with("https://raw.githubusercontent.com/HiroyukiFuruno/KatanA/refs/heads/master/CHANGELOG.ja.md?v=0.8.1-beta&t="),
        "URL {} does not contain expected prefix", url_ja
    );

    let url_unknown = ChangelogOps::get_changelog_url("it", "1.0.0");
    assert!(
        url_unknown.starts_with("https://raw.githubusercontent.com/HiroyukiFuruno/KatanA/refs/heads/master/CHANGELOG.md?v=1.0.0&t="),
        "URL {} does not contain expected prefix", url_unknown
    );
}

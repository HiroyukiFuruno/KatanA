use super::*;

#[test]
fn list_uncovered_lines_test() {
    let keys_with_trailing = vec![
        PersistentKey::WorkspaceTabs {
            workspace_path: std::path::PathBuf::from("/a/b/c/"),
        },
        PersistentKey::WorkspaceTabs {
            workspace_path: std::path::PathBuf::from("\\a\\b\\c\\"),
        },
    ];

    for key in keys_with_trailing {
        let raw = key.to_raw_key().unwrap();
        let expected_base = if raw.contains('\\') {
            "\\a\\b\\c"
        } else {
            "/a/b/c"
        };
        assert_eq!(raw, format!("workspace_tabs:{}", expected_base));
        let decoded = PersistentKey::from_raw_key(&raw).unwrap();
        match decoded {
            PersistentKey::WorkspaceTabs { workspace_path } => {
                assert_eq!(workspace_path.to_str().unwrap(), expected_base);
            }
            _ => panic!("Wrong type"),
        }
        assert!(
            key.target_filename()
                .unwrap()
                .starts_with("workspace_tabs_")
        );
    }

    let key = PersistentKey::Diagram {
        document_path: std::path::PathBuf::from("/a/b/c.md"),
        diagram_kind: "mermaid".to_string(),
        theme: "dark".to_string(),
        source_hash: "123".to_string(),
    };
    let raw = key.to_raw_key().unwrap();
    assert_eq!(raw, "diagram:/a/b/c.md:mermaid:dark:123");
    let decoded = PersistentKey::from_raw_key(&raw).unwrap();
    match decoded {
        PersistentKey::Diagram {
            document_path,
            diagram_kind,
            theme,
            source_hash,
        } => {
            assert_eq!(document_path.to_str().unwrap(), "/a/b/c.md");
            assert_eq!(diagram_kind, "mermaid");
            assert_eq!(theme, "dark");
            assert_eq!(source_hash, "123");
        }
        _ => panic!("Wrong type"),
    }

    let fname = key.target_filename().unwrap();
    assert!(fname.starts_with("diagram_"));
    assert!(fname.ends_with(".json"));

    assert_eq!(PersistentKey::Unknown.to_raw_key(), None);
    assert_eq!(PersistentKey::Unknown.target_filename(), None);
    assert!(matches!(
        PersistentKey::from_raw_key("invalid:format:string"),
        None
    ));
    assert!(matches!(PersistentKey::from_raw_key("invalid"), None));

    // Test trailing slash in from_raw_key directly
    let decoded = PersistentKey::from_raw_key("workspace_tabs:/a/b/c/").unwrap();
    match decoded {
        PersistentKey::WorkspaceTabs { workspace_path } => {
            assert_eq!(workspace_path.to_str().unwrap(), "/a/b/c");
        }
        _ => panic!("Wrong type"),
    }
}

#[test]
fn test_cache_facade_default_method() {
    struct MockFacade;
    impl CacheFacade for MockFacade {
        fn get_memory(&self, _k: &str) -> Option<String> {
            None
        }
        fn set_memory(&self, _k: &str, _v: String) {}
        fn get_persistent(&self, _k: &str) -> Option<String> {
            None
        }
        fn set_persistent(&self, _k: &str, _v: String) -> anyhow::Result<()> {
            Ok(())
        }
    }

    let m = MockFacade;
    assert_eq!(m.get_memory(""), None);
    m.set_memory("", String::new());
    assert_eq!(m.get_persistent(""), None);
    let _ = m.set_persistent("", String::new());
}

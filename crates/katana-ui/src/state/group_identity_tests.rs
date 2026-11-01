use super::*;

fn group(id: &str, name: &str, color: &str) -> TabGroup {
    TabGroup {
        id: id.into(),
        name: name.into(),
        color_hex: color.into(),
        collapsed: false,
        members: vec![],
    }
}

#[test]
fn name_and_color_pair_controls_conflicts() {
    let groups = [group("one", "", "#4A90D9"), group("two", "Work", "#D94A4A")];
    for (name, color, expected) in [
        ("", "#4a90d9", true),
        ("  ", "#4A90D9", true),
        ("", "#D94A4A", false),
        ("Work", "#D94A4A", true),
        ("Work", "#4A90D9", false),
        ("Other", "#D94A4A", false),
        ("work", "#D94A4A", false),
        (" Work", "#D94A4A", false),
    ] {
        assert_eq!(
            GroupIdentity::conflicts(&groups, None, name, color),
            expected,
            "{name:?} {color}"
        );
    }
    assert!(!GroupIdentity::conflicts(
        &groups,
        Some("two"),
        "Work",
        "#D94A4A"
    ));
    assert!(GroupIdentity::same_identity(
        "",
        "#4a90d9",
        "  ",
        "#4A90D9FF"
    ));
    assert!(GroupIdentity::same_identity("x", "invalid", "x", "invalid"));
    assert!(!GroupIdentity::same_identity(
        "x", "invalid", "x", "INVALID"
    ));
    assert!(!GroupIdentity::same_identity(
        "x",
        "#ff000000",
        "x",
        "#00000000"
    ));
}

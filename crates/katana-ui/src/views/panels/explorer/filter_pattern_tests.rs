use super::FilterPattern;

#[test]
fn unix_wildcard_keeps_backslash_inside_a_path_segment() {
    let regex =
        regex::Regex::new(&FilterPattern::normal_mode_pattern("a*.png", '/', false)).unwrap();

    assert!(regex.is_match(r"a\b.png"));
    assert!(!regex.is_match(r"b\a.png"));
}

#[test]
fn windows_wildcards_accept_both_separator_spellings() {
    let basename = regex::Regex::new(&FilterPattern::normal_mode_pattern(
        "icon*.png",
        '\\',
        false,
    ))
    .unwrap();
    assert!(basename.is_match(r"assets\icon.png"));

    let relative = regex::Regex::new(&FilterPattern::normal_mode_pattern(
        "assets/*.png",
        '\\',
        false,
    ))
    .unwrap();
    assert!(relative.is_match(r"assets\icon.png"));
    assert!(!relative.is_match(r"assets\nested\icon.png"));

    let backslash_query = regex::Regex::new(&FilterPattern::normal_mode_pattern(
        r"assets\*.png",
        '\\',
        false,
    ))
    .unwrap();
    assert!(backslash_query.is_match(r"assets\icon.png"));
    assert!(!backslash_query.is_match(r"assets\nested\icon.png"));
}

#[test]
fn wildcard_word_boundary_is_applied_after_the_path_prefix() {
    let unix = regex::Regex::new(&FilterPattern::normal_mode_pattern("*.png", '/', true)).unwrap();
    assert!(unix.is_match("assets (copy)/icon.png"));
    assert!(unix.is_match("assets-/icon.png"));
    assert!(!unix.is_match("assets-/icon.png.bak"));

    let windows =
        regex::Regex::new(&FilterPattern::normal_mode_pattern("*.png", '\\', true)).unwrap();
    assert!(windows.is_match(r"assets (copy)\icon.png"));
    assert!(windows.is_match(r"assets-\icon.png"));
    assert!(!windows.is_match(r"assets-\icon.png.bak"));
}

#![cfg(feature = "better_syntax_highlighting")]

use super::{CodeBlock, CommonMarkCache, CommonMarkOptions};
use egui::Ui;
use std::sync::OnceLock;
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

const CUSTOM_SYNTAX: &str = r#"%YAML 1.2
---
name: Codex Lazy Cache Test
file_extensions: [codexlazy]
scope: source.codexlazy
contexts:
  main:
    - match: "\\bSPECIAL\\b"
      scope: keyword.control.codexlazy
"#;
const RUST_SOURCE: &str = "fn main() { let value = 42; }\n";

const CUSTOM_THEME: &[u8] = br##"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
  <key>name</key><string>Codex Lazy Cache Test</string>
  <key>settings</key><array><dict><key>settings</key><dict>
    <key>background</key><string>#123456</string>
    <key>foreground</key><string>#FFFFFF</string>
  </dict></dict></array>
</dict>
</plist>"##;

fn with_test_ui<R>(mut f: impl FnMut(&mut Ui) -> R) -> R {
    let mut result = None;
    egui::__run_test_ui(|ui| {
        result = Some(f(ui));
    });
    result.expect("test context should invoke the panel contents")
}
fn render_highlight(
    cache: &CommonMarkCache,
    options: &CommonMarkOptions<'_>,
    extension: &str,
    source: &str,
) -> egui::text::LayoutJob {
    with_test_ui(|ui| {
        CodeBlock {
            lang: Some(extension.to_owned()),
            content: source.to_owned(),
        }
        .syntax_highlighting(cache, options, extension, ui, source)
    })
}
fn render_pre_syntax_highlighting(
    cache: &mut CommonMarkCache,
    options: &CommonMarkOptions<'_>,
) -> egui::Color32 {
    with_test_ui(|ui| {
        CodeBlock::pre_syntax_highlighting(cache, options, ui);
        ui.visuals().extreme_bg_color
    })
}
fn assert_default_and_custom_entries(cache: &CommonMarkCache) {
    let syntaxes = cache.ps.get().expect("syntax set should be initialized");
    assert!(syntaxes.find_syntax_by_extension("rs").is_some());
    assert!(syntaxes.find_syntax_by_extension("codexlazy").is_some());

    let themes = cache.ts.get().expect("theme set should be initialized");
    assert!(themes.themes.contains_key("base16-ocean.light"));
    assert!(themes.themes.contains_key("base16-ocean.dark"));
    assert!(themes.themes.contains_key("codex-lazy-test"));
}
#[test]
fn default_cache_defers_sets_until_the_highlighting_path_needs_them() {
    let mut cache = CommonMarkCache::default();
    assert!(cache.ps.get().is_none());
    assert!(cache.ts.get().is_none());

    let options = CommonMarkOptions::default();
    let _ = render_pre_syntax_highlighting(&mut cache, &options);
    assert!(cache.ps.get().is_none());
    assert!(cache.ts.get().is_some());

    let first = render_highlight(&cache, &options, "rs", RUST_SOURCE);
    let initialized_syntaxes = cache.ps.get().expect("highlighting initializes syntax set");
    let initialized_themes = cache
        .ts
        .get()
        .expect("pre-highlighting initializes theme set");
    let second = render_highlight(&cache, &options, "rs", RUST_SOURCE);

    assert_eq!(first, second);
    assert!(std::ptr::eq(
        initialized_syntaxes,
        cache.ps.get().expect("syntax set is reused")
    ));
    assert!(std::ptr::eq(
        initialized_themes,
        cache.ts.get().expect("theme set is reused")
    ));
}
#[test]
fn default_rust_highlighting_matches_an_eager_syntect_reference() {
    let lazy = CommonMarkCache::default();
    let eager = CommonMarkCache {
        ps: OnceLock::from(SyntaxSet::load_defaults_newlines()),
        ts: OnceLock::from(ThemeSet::load_defaults()),
        ..CommonMarkCache::default()
    };
    let options = CommonMarkOptions::default();

    let lazy_output = render_highlight(&lazy, &options, "rs", RUST_SOURCE);
    let eager_output = render_highlight(&eager, &options, "rs", RUST_SOURCE);

    assert_eq!(lazy_output, eager_output);
    assert!(lazy.ps.get().is_some());
    assert!(lazy.ts.get().is_some());
}
#[test]
fn custom_syntax_and_theme_can_be_added_before_or_after_highlighting() {
    let options = CommonMarkOptions::default();

    let mut before = CommonMarkCache::default();
    assert!(before.ps.get().is_none());
    assert!(before.ts.get().is_none());
    before
        .add_syntax_from_str(CUSTOM_SYNTAX, Some("Codex Lazy Test"))
        .expect("custom syntax should parse");
    before
        .add_syntax_theme_from_bytes("codex-lazy-test", CUSTOM_THEME)
        .expect("custom theme should parse");
    assert_default_and_custom_entries(&before);
    let before_output = render_highlight(&before, &options, "codexlazy", "SPECIAL\n");
    let mut custom_options = CommonMarkOptions::default();
    custom_options.theme_light = "codex-lazy-test".to_owned();
    custom_options.theme_dark = "codex-lazy-test".to_owned();
    assert_eq!(
        render_pre_syntax_highlighting(&mut before, &custom_options),
        egui::Color32::from_rgb(0x12, 0x34, 0x56)
    );
    assert_eq!(before_output.text, "SPECIAL\n");

    let mut after = CommonMarkCache::default();
    let _ = render_highlight(&after, &options, "rs", RUST_SOURCE);
    after
        .add_syntax_from_str(CUSTOM_SYNTAX, Some("Codex Lazy Test"))
        .expect("custom syntax should parse after initialization");
    after
        .add_syntax_theme_from_bytes("codex-lazy-test", CUSTOM_THEME)
        .expect("custom theme should parse after initialization");
    assert_default_and_custom_entries(&after);
    let after_output = render_highlight(&after, &options, "codexlazy", "SPECIAL\n");
    assert_eq!(
        render_pre_syntax_highlighting(&mut after, &custom_options),
        egui::Color32::from_rgb(0x12, 0x34, 0x56)
    );
    assert_eq!(after_output.text, "SPECIAL\n");

    assert_eq!(before_output, after_output);
}
#[test]
fn invalid_custom_inputs_preserve_default_and_preexisting_entries() {
    let mut cache = CommonMarkCache::default();
    cache
        .add_syntax_from_str(CUSTOM_SYNTAX, Some("Codex Lazy Test"))
        .expect("custom syntax should parse");
    cache
        .add_syntax_theme_from_bytes("codex-lazy-test", CUSTOM_THEME)
        .expect("custom theme should parse");

    assert!(
        cache
            .add_syntax_from_str("%YAML 1.2\n---\ncontexts: [\n", Some("Invalid"))
            .is_err()
    );
    assert!(
        cache
            .add_syntax_theme_from_bytes("invalid-theme", b"not a theme")
            .is_err()
    );

    let missing_folder = std::env::temp_dir().join(format!(
        "egui-commonmark-missing-syntax-{}",
        std::process::id()
    ));
    assert!(!missing_folder.exists());
    cache.add_syntax_from_folder(
        missing_folder
            .to_str()
            .expect("temporary path should be valid UTF-8"),
    );
    assert!(
        cache
            .add_syntax_themes_from_folder(&missing_folder)
            .is_err()
    );

    assert_default_and_custom_entries(&cache);
}

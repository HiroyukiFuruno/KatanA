/* WHY: Compile-time embedded demo assets.

All demo Markdown files from `assets/feature/` are embedded into the binary via `include_str!`.
This eliminates runtime filesystem dependency, making the demo work regardless of
the current working directory or installation method. */

/// A single embedded demo asset with metadata for locale resolution and display.
pub(super) struct DemoAsset {
    /// Virtual path displayed in the tab bar (e.g. "Katana://Demo/welcome.md").
    pub virtual_path: &'static str,
    /// The embedded file content.
    pub content: &'static str,
    /// Whether this is a reference (read-only) document.
    pub is_reference: bool,
}

/* WHY: English Markdown assets */
const WELCOME_EN: &str = include_str!("../../../../../assets/feature/welcome.md");
const RENDERING_FEATURES_EN: &str =
    include_str!("../../../../../assets/feature/rendering_features.md");

/* WHY: Japanese Markdown assets */
const WELCOME_JA: &str = include_str!("../../../../../assets/feature/welcome.ja.md");
const RENDERING_FEATURES_JA: &str =
    include_str!("../../../../../assets/feature/rendering_features.ja.md");

/* WHY: Guide assets */
const GUIDE_JA: &str = include_str!("../../../resources/guide_ja.md");
const GUIDE_EN: &str = include_str!("../../../resources/guide_en.md");

/* WHY: App Welcome assets. */
const APP_WELCOME_JA: &str = include_str!("../../../resources/welcome_ja.md");
const APP_WELCOME_EN: &str = include_str!("../../../resources/welcome_en.md");

/// Resolve single embedded demo asset or help asset by name.
pub(super) fn resolve_single_asset(lang: &str, filename: &str) -> Option<DemoAsset> {
    match filename {
        "welcome.md" => {
            let content = if lang.starts_with("ja") {
                APP_WELCOME_JA
            } else {
                APP_WELCOME_EN
            };
            Some(DemoAsset {
                virtual_path: Box::leak("Katana://Welcome.md".to_string().into_boxed_str()),
                content,
                is_reference: true,
            })
        }
        "guide.md" => {
            let content = if lang.starts_with("ja") {
                GUIDE_JA
            } else {
                GUIDE_EN
            };
            Some(DemoAsset {
                virtual_path: Box::leak("Katana://Guide.md".to_string().into_boxed_str()),
                content,
                is_reference: true,
            })
        }
        _ => None,
    }
}

/// Resolve the embedded demo bundle based on the current language.
///
/// Resolution rules:
/// - Only Markdown files are included (non-Markdown assets are excluded).
/// - Prefer Japanese variant when `lang == "ja"`, fall back to English.
/// - Welcome document is always returned first.
pub(super) fn resolve_demo_bundle(lang: &str) -> Vec<DemoAsset> {
    let (welcome, rendering) = if lang.starts_with("ja") {
        (WELCOME_JA, RENDERING_FEATURES_JA)
    } else {
        (WELCOME_EN, RENDERING_FEATURES_EN)
    };

    let welcome_filename = "welcome.md";
    let rendering_filename = "rendering_features.md";

    vec![
        /* WHY: Welcome is always first */
        DemoAsset {
            virtual_path: demo_virtual_path(welcome_filename),
            content: welcome,
            is_reference: true,
        },
        DemoAsset {
            virtual_path: demo_virtual_path(rendering_filename),
            content: rendering,
            is_reference: true,
        },
    ]
}

/// Generate a `Katana://Demo/<filename>` virtual path string.
///
/// The `Katana://` prefix is already used by ChangeLog and ensures:
/// - auto-refresh skips these paths (see `handle_action_refresh_document`)
/// - save operations are blocked for reference documents
fn demo_virtual_path(filename: &str) -> &'static str {
    /* WHY: We leak the string to get a 'static lifetime.
    This is acceptable because the demo bundle is a fixed, small set of paths
    created at most once per app session. */
    Box::leak(format!("Katana://Demo/{filename}").into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_demo_bundle_returns_two_assets_en() {
        let bundle = resolve_demo_bundle("en");
        assert_eq!(bundle.len(), 2);
        assert!(bundle[0].virtual_path.contains("welcome.md"));
        assert!(bundle[1].virtual_path.contains("rendering_features.md"));
    }

    #[test]
    fn resolve_demo_bundle_returns_two_assets_ja() {
        let bundle = resolve_demo_bundle("ja");
        assert_eq!(bundle.len(), 2);
        assert!(bundle[0].virtual_path.contains("welcome.md"));
        assert!(bundle[1].virtual_path.contains("rendering_features.md"));
    }

    #[test]
    fn all_demo_assets_are_reference() {
        for asset in resolve_demo_bundle("en") {
            assert!(
                asset.is_reference,
                "{} should be reference",
                asset.virtual_path
            );
        }
    }

    #[test]
    fn welcome_is_always_first() {
        for lang in &["en", "ja", "zh-CN", "ko", "fr"] {
            let bundle = resolve_demo_bundle(lang);
            assert!(
                bundle[0].virtual_path.contains("welcome"),
                "First asset for lang={lang} should be welcome, got: {}",
                bundle[0].virtual_path
            );
        }
    }

    #[test]
    fn virtual_paths_use_katana_prefix() {
        for asset in resolve_demo_bundle("en") {
            assert!(
                asset.virtual_path.starts_with("Katana://Demo/"),
                "Expected Katana://Demo/ prefix, got: {}",
                asset.virtual_path
            );
        }
    }

    #[test]
    fn embedded_content_is_not_empty() {
        for asset in resolve_demo_bundle("en") {
            assert!(
                !asset.content.is_empty(),
                "Content for {} should not be empty",
                asset.virtual_path
            );
        }
    }

    #[test]
    fn only_markdown_files_included() {
        for asset in resolve_demo_bundle("en") {
            assert!(
                asset.virtual_path.ends_with(".md"),
                "Only .md files should be in demo bundle, got: {}",
                asset.virtual_path
            );
        }
    }
}

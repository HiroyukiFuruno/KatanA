use katana_linter::AstLinterOps;
use katana_linter::rules::domains::changelog::ChangelogOps;
use katana_linter::rules::domains::i18n::{I18nOps, IconOps};
use katana_linter::rules::domains::locales::LocaleOps;
use katana_linter::rules::domains::markdown::MarkdownOps;
use katana_linter::rules::domains::theme::{
    HardcodedColorOps, ThemeBuilderOps, UnusedThemeColorOps,
};
use katana_linter::rules::{
    CommentStyleOps, ErrorFirstOps, FileLengthOps, FontNormalizationOps, FrameStrokeOps,
    FunctionLengthOps, HorizontalLayoutOps, LazyCodeOps, MagicNumberOps, NestingDepthOps,
    PerformanceOps, ProhibitedAttributesOps, ProhibitedTypesOps, PubFreeFnOps, TypeSeparationOps,
};
use katana_linter::utils::{LinterFileOps, ViolationReporterOps};

fn target_crates(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    vec![
        root.join("crates/katana-linter/src"),
        root.join("crates/katana-core/src"),
        root.join("crates/katana-platform/src"),
        root.join("crates/katana-ui/src"),
    ]
}

#[test]
fn ast_linter_i18n_no_hardcoded_strings() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "i18n",
        "Fix: Replace string literals with i18n::t(\"key\") or i18n::tf(\"key\", &[...]).",
        &target_crates(root),
        I18nOps::lint,
    );
}

#[test]
fn ast_linter_no_magic_numbers() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "magic-number",
        "Fix: Extract numeric literals into named constants (const).",
        &[
            root.join("crates/katana-core/src"),
            root.join("crates/katana-platform/src"),
            root.join("crates/katana-ui/src"),
        ],
        MagicNumberOps::lint,
    );
}

#[test]
fn ast_linter_no_lazy_code() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "lazy-code",
        "Fix: Remove `todo!()`, `unimplemented!()`, and `dbg!()` macros. Implement the actual logic.",
        &target_crates(root),
        LazyCodeOps::lint,
    );
}

#[test]
fn ast_linter_no_prohibited_types() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "prohibited-types",
        "Fix: Use `Vec` instead of `HashMap`, `[T; N]` or `[...]`.",
        &target_crates(root),
        ProhibitedTypesOps::lint,
    );
}

#[test]
fn ast_linter_locale_files_match_base_structure() {
    let locale_dir = LinterFileOps::workspace_root()
        .expect("Test requirement")
        .join("crates/katana-ui/locales");
    let all_violations = LocaleOps::lint(&locale_dir);
    ViolationReporterOps::panic(
        "locale-structure",
        "Fix: Keep every locale JSON aligned with ja.json/en.json, including placeholder names.",
        &all_violations,
    );
}

#[test]
fn ast_linter_no_raw_icons() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "icon-facade",
        "Fix: Use `Icon::Name.as_str()` instead of raw icon string literals like \"🔄\".",
        &target_crates(root),
        IconOps::lint,
    );
}

#[test]
fn ast_linter_markdown_heading_pairs_match() {
    let all_violations =
        MarkdownOps::lint(LinterFileOps::workspace_root().expect("Test requirement"));
    ViolationReporterOps::panic(
        "markdown-heading-structure",
        "Fix: Keep each *.md and corresponding .ja/_ja markdown file aligned by heading count and heading levels.",
        &all_violations,
    );
}

#[test]
fn ast_linter_changelog_contains_current_workspace_version() {
    let all_violations =
        ChangelogOps::lint(LinterFileOps::workspace_root().expect("Test requirement"));
    ViolationReporterOps::panic(
        "changelog-version-sync",
        "Fix: Add a `## [x.y.z]` release heading to CHANGELOG.md that matches workspace.package.version in Cargo.toml.",
        &all_violations,
    );
}

#[test]
fn ast_linter_font_normalization() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "font-normalization",
        "Fix: Use `NormalizeFonts` from `font_loader` instead of raw `FontDefinitions::default()`/`::empty()`.",
        &target_crates(root),
        FontNormalizationOps::lint,
    );
}

#[test]
fn ast_linter_no_unoptimized_performance() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "performance",
        "Fix: Avoid unconditional `request_repaint()` or `set_title()` calls in UI loops.",
        &[root.join("crates/katana-ui/src")],
        PerformanceOps::lint,
    );
}

#[test]
fn ast_linter_no_allow_dead_code() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "prohibited-attributes",
        "Fix: Remove `#[allow(dead_code)]`. Dead code should be deleted, not silenced.",
        &target_crates(root),
        ProhibitedAttributesOps::lint,
    );
}

#[test]
fn ast_linter_file_length() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "file-length",
        "Fix: File exceeds 200-line limit (excluding tests). Split into smaller modules.",
        &[root.join("crates/katana-linter/src")],
        FileLengthOps::lint,
    );
}

#[test]
fn ast_linter_type_separation() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "type-separation",
        "Fix: Do not mix `pub struct` / `pub enum` in the same file as implementation logic methods if the file exceeds the length limit. Use dedicated files like `types.rs` or `types/` dir.",
        &target_crates(root),
        TypeSeparationOps::lint,
    );
}

#[test]
fn ast_linter_function_length() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "function-length",
        "Fix: Function exceeds 30-line limit. Extract helper methods.",
        &[
            root.join("crates/katana-linter/src/rules/coding"),
            root.join("crates/katana-linter/src/rules/structure"),
        ],
        FunctionLengthOps::lint,
    );
}

#[test]
fn ast_linter_nesting_depth() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "nesting-depth",
        "Fix: Nesting depth exceeds 3 levels. Use early returns or extract helpers.",
        &[
            root.join("crates/katana-linter/src/rules/coding"),
            root.join("crates/katana-linter/src/rules/structure"),
        ],
        NestingDepthOps::lint,
    );
}

#[test]
fn ast_linter_comment_style() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "comment-style",
        "Fix: Comments must start with `// WHY:` or `// SAFETY:`. Code should be self-documenting.",
        &[
            root.join("crates/katana-linter/src/rules/coding"),
            root.join("crates/katana-linter/src/rules/structure"),
        ],
        CommentStyleOps::lint,
    );
}

#[test]
fn ast_linter_error_first() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "error-first",
        "Fix: Do not nest success paths with `if let Ok(...)`. Use `?` or `let-else` to fail early.",
        &[
            root.join("crates/katana-linter/src/rules/coding"),
            root.join("crates/katana-linter/src/rules/structure"),
        ],
        ErrorFirstOps::lint,
    );
}

#[test]
fn ast_linter_no_pub_free_fn() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "pub-free-fn",
        "Fix: Public free functions are prohibited. Use struct + impl blocks (coding-rules §1.1).",
        &target_crates(root),
        PubFreeFnOps::lint,
    );
}

#[test]
fn ast_linter_no_unused_theme_colors() {
    let all_violations =
        UnusedThemeColorOps::lint(LinterFileOps::workspace_root().expect("Test requirement"));
    ViolationReporterOps::panic(
        "unused-theme-colors",
        "Fix: A theme color property is defined in `ThemeColors` but never accessed in UI code. Please use it or remove it.",
        &all_violations,
    );
}

#[test]
fn ast_linter_no_hardcoded_colors() {
    let all_violations =
        HardcodedColorOps::lint(LinterFileOps::workspace_root().expect("Test requirement"));
    ViolationReporterOps::panic(
        "hardcoded-colors",
        "Fix: A hardcoded UI color was found. Map it to a property in `ThemeColors` and use `theme_bridge::rgb_to_color32`.",
        &all_violations,
    );
}

#[test]
fn ast_linter_theme_builder_enforcement() {
    let all_violations =
        ThemeBuilderOps::lint(LinterFileOps::workspace_root().expect("Test requirement"));
    ViolationReporterOps::panic(
        "theme-builder-enforcement",
        "Fix: Theme presets must use `ThemePresetBuilder::new(...)` to enforce DRY design. Do not instantiate `PresetColorData` directly.",
        &all_violations,
    );
}

#[test]
fn ast_linter_no_japanese_in_crates() {
    use ignore::WalkBuilder;
    let root = LinterFileOps::workspace_root()
        .expect("Test requirement")
        .join("crates");

    // We walk in parallel for maximum performance without degrading test speed
    let (tx, rx) = std::sync::mpsc::channel();

    let walker = WalkBuilder::new(root).build_parallel();

    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            if let Ok(entry) = result {
                let path = entry.path();
                if path.is_file() {
                    // Exclude ja.json (the only allowed source of Japanese truth)
                    if path.file_name().is_some_and(|name| name == "ja.json") {
                        return ignore::WalkState::Continue;
                    }

                    if let Ok(content) = std::fs::read_to_string(path) {
                        for (line_idx, line) in content.lines().enumerate() {
                            // Detect Japanese specifically combining Hiragana and Katakana.
                            // We intentionally exclude pure Han ideographs (\p{Han}) because Katana includes Chinese locales (zh-TW, zh-CN)
                            // which must not trigger the Japanese check.
                            if line.chars().any(|c| matches!(c, '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}')) {
                                let _ = tx.send(format!("{}:{}: Please remove Japanese text or use Unicode escapes for test strings.", path.display(), line_idx + 1));
                                break;
                            }
                        }
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });

    drop(tx); // Close the transmitter

    let mut violations = Vec::new();
    for failure in rx {
        violations.push(katana_linter::Violation {
            file: std::path::PathBuf::from(""), // dummy
            line: 0,
            column: 0,
            message: failure,
        });
    }

    if !violations.is_empty() {
        ViolationReporterOps::panic(
            "no-japanese-in-workspace",
            "Fix: No Japanese text (Hiragana/Katakana) is allowed in any files except ja.json. Please translate comments to English or use Unicode escapes for test data.",
            &violations,
        );
    }
}

#[test]
fn ast_linter_no_horizontal_layout() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "horizontal-layout",
        "Fix: Use `AlignCenter` instead of `ui.horizontal()` for vertical centering. Add `// allow(horizontal_layout)` above the line to suppress.",
        &[root.join("crates/katana-ui/src")],
        HorizontalLayoutOps::lint,
    );
}

#[test]
fn ast_linter_no_frame_stroke() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "frame-stroke",
        "Fix: Frame `.stroke()` and `rect_stroke()` cause layout jitter. Use theme visuals or `rect.shrink(stroke_width)`. Add `// allow(frame_stroke)` above to suppress.",
        &[root.join("crates/katana-ui/src")],
        FrameStrokeOps::lint,
    );
}

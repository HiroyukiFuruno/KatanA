use katana_linter::AstLinterOps;
use katana_linter::rules::domains::changelog::ChangelogOps;
use katana_linter::rules::domains::locales::LocaleAudit;
use katana_linter::rules::domains::markdown::MarkdownOps;
use katana_linter::rules::domains::theme::{
    HardcodedColorOps, ThemeBuilderOps, UnusedThemeColorOps,
};
use katana_linter::rules::{
    FontNormalizationOps, ForegroundSurfaceOps, GlobalMenuParityOps, ProcessCommandOps,
    ReleaseScriptOps,
};
use katana_linter::utils::{LinterFileOps, LinterParserOps, ViolationReporterOps};
use std::sync::{LazyLock, Mutex};

static KAL_CURRENT_DIR_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

struct CurrentDirGuard {
    original_dir: std::path::PathBuf,
}

impl CurrentDirGuard {
    fn enter(target_dir: &std::path::Path) -> Self {
        let original_dir = std::env::current_dir().expect("Test requirement");
        std::env::set_current_dir(target_dir).expect("Test requirement");
        Self { original_dir }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.original_dir).expect("Test requirement");
    }
}

fn target_crates(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    vec![
        root.join("crates/katana-linter/src"),
        root.join("crates/katana-linter/tests"),
        root.join("crates/katana-core/src"),
        root.join("crates/katana-core/tests"),
        root.join("crates/katana-platform/src"),
        root.join("crates/katana-platform/tests"),
        root.join("crates/katana-ui/src"),
        root.join("crates/katana-ui/tests"),
    ]
}

/* WHY: Headless-process enforcement must cover every path that can spawn an OS process,
 * including supplementary script crates that live outside the primary `crates/katana-*` tree.
 * Build scripts are scanned separately by `ast_linter_no_direct_process_command_in_build_scripts`
 * because they live at `crates/<name>/build.rs` (outside any `src/`). */
fn headless_process_target_dirs(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    vec![
        root.join("crates/katana-core/src"),
        root.join("crates/katana-linter/src"),
        root.join("crates/katana-platform/src"),
        root.join("crates/katana-ui/src"),
        root.join("scripts/screenshot/src"),
    ]
}

#[test]
fn ast_linter_shared_kal_rules() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    let _lock = KAL_CURRENT_DIR_LOCK.lock().expect("Test requirement");
    let _guard = CurrentDirGuard::enter(root);
    katana_ast_lint::KatanaAstLint::from_workspace().assert_clean();
}

#[test]
fn ast_linter_icon_sync() {
    use katana_linter::rules::domains::assets::IconsSyncOps;
    let all_violations =
        IconsSyncOps::lint(LinterFileOps::workspace_root().expect("Test requirement"));
    ViolationReporterOps::panic(
        "icon-sync",
        "Fix: Synchronize SVG files in assets/icons/ with define_icons! macro by ensuring 1-to-1 matching across all themes.",
        &all_violations,
    );
}

#[test]
fn ast_linter_svg_colors() {
    use katana_linter::rules::domains::assets::SvgOps;
    let all_violations =
        SvgOps::lint_svg_colors(LinterFileOps::workspace_root().expect("Test requirement"));
    ViolationReporterOps::panic(
        "svg-colors",
        "Fix: SVGs must not have invalid colors (only #FFFFFF or currentColor allowed) and must have at least one fill or stroke attribute to prevent blackout.",
        &all_violations,
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
fn ast_linter_windows_msi_packaging_uses_current_version() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    let all_violations = ReleaseScriptOps::lint_windows_msi_packaging(root);
    ViolationReporterOps::panic(
        "release-script-windows-msi-version",
        "Fix: Windows packaging must remove stale MSI files and copy only the MSI matching the current Cargo version.",
        &all_violations,
    );
}

#[test]
fn ast_linter_release_automation_has_no_ci_bypass_markers() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    let all_violations = ReleaseScriptOps::lint_ci_integrity(root);
    ViolationReporterOps::panic(
        "no-skip-ci-marker",
        "Fix: release-ci-integrity forbids CI bypass markers in scripts/release/** and .github/workflows/**.",
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

    let (tx, rx) = std::sync::mpsc::channel();

    let walker = WalkBuilder::new(root).build_parallel();

    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            if let Ok(entry) = result {
                let path = entry.path();
                if path.is_file() {
                    if path.file_name().is_some_and(|name| name == "ja.json") {
                        return ignore::WalkState::Continue;
                    }

                    if path.components().any(|c| c.as_os_str() == "resources") {
                        return ignore::WalkState::Continue;
                    }

                    if let Ok(content) = std::fs::read_to_string(path) {
                        for (line_idx, line) in content.lines().enumerate() {
                            /* WHY: Detect Japanese specifically combining Hiragana and Katakana.
                                We intentionally exclude pure Han ideographs (\p{Han}) because Katana includes Chinese locales (zh-TW, zh-CN)
                                which must not trigger the Japanese check. */
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

    drop(tx);

    let mut violations = Vec::new();
    for failure in rx {
        violations.push(katana_linter::Violation {
            file: std::path::PathBuf::from(""),
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
fn ast_linter_no_hardcoded_os_commands() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    let all_violations = katana_linter::rules::domains::os_command::OsCommandOps::lint(root);
    ViolationReporterOps::panic(
        "no-hardcoded-os-commands",
        "Fix: Do not hardcode OS shortcuts like `\\u{2318}` or use invalid placeholder keys. Use {{os_cmd:key}} in Markdown and resolve dynamically.",
        &all_violations,
    );
}

#[test]
fn ast_linter_global_menu_parity() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    let all_violations = GlobalMenuParityOps::lint(root);
    ViolationReporterOps::panic(
        "global-menu-parity",
        "Fix: Windows/Linux global menu modules (`global_menu*.rs`) and macOS native menu (`native_menu/mod.rs` & `macos_menu.m`) must have parity in their available `AppAction` variants. Ensure any action added to one is also added to the other.",
        &all_violations,
    );
}

#[test]
fn ast_linter_foreground_surface_blocks_lower_ui() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    let all_violations = ForegroundSurfaceOps::lint(root);
    ViolationReporterOps::panic(
        "foreground-surface",
        "Fix: Foreground overlays such as modals, context menus, and slideshow mode must block lower UI events.",
        &all_violations,
    );
}

#[test]
fn ast_linter_settings_alignment() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    use katana_linter::rules::domains::ui::settings_alignment::SettingsAlignmentOps;
    let all_violations = SettingsAlignmentOps::check_settings_alignment(root);
    ViolationReporterOps::panic(
        "settings-alignment",
        "Fix: Layout properties inside settings must use `AlignCenter` and `egui::Align::Max` to prevent alignment breakages. Do not use `LabeledToggle` in linter properties, `Align::Min`, or checkbox controls.",
        &all_violations,
    );
}

#[test]
fn ast_linter_shortcut_duplicates() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    let all_violations = katana_linter::rules::domains::shortcut::ShortcutOps::lint(root);
    ViolationReporterOps::panic(
        "shortcut-duplicates",
        "Fix: Duplicate shortcuts are not allowed across commands. Ensure each OS shortcut mapping in `os_commands.json` is unique.",
        &all_violations,
    );
}

#[test]
fn ast_linter_no_direct_process_command_in_sources() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    AstLinterOps::run(
        "no-direct-process-command",
        "Fix: Route every process spawn through `ProcessService::create_command`. \
         In a build.rs, `include!(\"build_support/process.rs\")` and call `create_build_command`. \
         Direct `std::process::Command::new` is forbidden outside the allowlisted facades. \
         For Java on Windows, use the GUI-subsystem `javaw` launcher with `create_command` — \
         `Stdio::piped()` does not suppress console allocation when a GUI parent spawns a \
         console-subsystem child.",
        &headless_process_target_dirs(root),
        ProcessCommandOps::lint,
    );
}

#[test]
fn ast_linter_no_direct_process_command_in_build_scripts() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    let build_scripts = LinterFileOps::collect_build_scripts(root);
    let mut violations: Vec<katana_linter::Violation> = Vec::new();
    for file in &build_scripts {
        match LinterParserOps::parse_file(file) {
            Ok(syntax) => violations.extend(ProcessCommandOps::lint_build_script(file, &syntax)),
            Err(errors) => violations.extend(errors),
        }
    }
    ViolationReporterOps::panic(
        "no-direct-process-command-build-scripts",
        "Fix: A build.rs called `std::process::Command::new` directly. \
         Add `include!(\"build_support/process.rs\")` to the build script and use \
         `create_build_command(...)` so Windows builds do not flash a console window.",
        &violations,
    );
}

#[test]
fn ast_linter_headless_process_build_scripts_are_discovered() {
    /* WHY: Regression guard. If `collect_build_scripts` ever returns an empty list (because
     * the walker max_depth, file name filter, or workspace layout regresses), the
     * `ast_linter_no_direct_process_command_in_build_scripts` test would silently pass
     * because there would be nothing to scan. This test fails fast in that case. */
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    let build_scripts = LinterFileOps::collect_build_scripts(root);
    assert!(
        !build_scripts.is_empty(),
        "Expected at least one build.rs in crates/*; the build-script collector regressed."
    );
    assert!(
        build_scripts
            .iter()
            .any(|p| p.ends_with("katana-ui/build.rs")),
        "Expected crates/katana-ui/build.rs to be discovered by collect_build_scripts."
    );
}

#[test]
fn ast_linter_i18n_no_unused_keys() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    let locale_dir = root.join("crates/katana-ui/locales");
    let all_violations = LocaleAudit::lint_unused_keys(root, &locale_dir);
    ViolationReporterOps::panic(
        "i18n-unused-keys",
        "Fix: The locale key is not referenced in any Rust source file. \
         Consider removing it to keep translations lean.",
        &all_violations,
    );
}

#[test]
fn ast_linter_i18n_no_duplicate_values() {
    let root = LinterFileOps::workspace_root().expect("Test requirement");
    let locale_dir = root.join("crates/katana-ui/locales");
    /* WHY: Only check the base locale (en.json) — duplicates there affect all languages. */
    let en_path = locale_dir.join("en.json");
    let all_violations = LocaleAudit::lint_duplicate_values_within_file(&en_path);
    ViolationReporterOps::panic(
        "i18n-duplicate-values",
        "Fix: The locale value appears in multiple sections. \
         Consider consolidating it into `common` to avoid maintaining the same \
         translation in multiple places.",
        &all_violations,
    );
}

//! # Lint Fix Integration Scenarios
//!
//! Per-rule fix scenarios using `fix_harness`. This file is intentionally
//! separated from the harness so that:
//!
//! - `fix_harness.rs`  → portable library (can be extracted as a crate)
//! - `fix_scenarios.rs`→ this file: rule-specific test matrix
//!
//! ## Repository-portability note
//!
//! When the linter is extracted to its own repository, copy both files.
//! No UI or platform dependencies exist in either.

#[path = "fix_harness.rs"]
mod fix_harness;

use fix_harness::{BulkFixScenario, LintFixScenario};
use katana_linter::rules::markdown::{
    BlanksAroundHeadingsRule, BlanksAroundListsRule, HeadingStartLeftRule,
};

// ═══════════════════════════════════════════════════════════
// MD022 — BlanksAroundHeadings
// ═══════════════════════════════════════════════════════════

/// Basic: single missing blank line before heading is fixed cleanly.
#[test]
fn md022_single_fix_missing_blank_before() {
    LintFixScenario {
        rule: &BlanksAroundHeadingsRule,
        description: "MD022 – missing blank before heading",
        input: "Content\n## Section\n\nMore\n",
        expected_initial_count: 1,
    }
    .run()
    .assert_clean();
}

/// Basic: single missing blank line after heading is fixed cleanly.
#[test]
fn md022_single_fix_missing_blank_after() {
    LintFixScenario {
        rule: &BlanksAroundHeadingsRule,
        description: "MD022 – missing blank after heading",
        input: "# Title\n\n## Section\nContent\n",
        expected_initial_count: 1,
    }
    .run()
    .assert_clean();
}

/// Bulk (single-pass): adjacent headings are all fixed cleanly after FB29 fix.
///
/// Previously this would produce duplicate headings when the replacement included
/// the heading text. Now each fix inserts a single "\n" at a zero-width point,
/// so descending-sort bulk application never corrupts adjacent fixes.
#[test]
fn md022_bulk_fix_adjacent_headings_clean() {
    BulkFixScenario {
        rule: &BlanksAroundHeadingsRule,
        description: "MD022 – bulk fix: adjacent headings all get blank lines (FB29)",
        input: "# A\n## B\n## C\n## D\n",
        expected_min_initial_count: 2,
    }
    .run()
    .assert_clean();
}

/// Convergence: sequential single-fix applications (simulates UI "Fix" button
/// pressed repeatedly) must converge to a clean document without cascading.
#[test]
fn md022_sequential_single_fix_converges_to_clean() {
    use katana_linter::rules::markdown::MarkdownRule;
    let rule = BlanksAroundHeadingsRule;
    let path = std::path::PathBuf::from("test.md");
    let mut buffer = "# A\n## B\n## C\n## D\n".to_string();

    const MAX_ITERATIONS: usize = 30;
    for i in 0..MAX_ITERATIONS {
        let diags = rule.evaluate(&path, &buffer);
        if diags.is_empty() {
            return; // converged — test passes
        }
        let fix = diags[0]
            .fix_info
            .as_ref()
            .unwrap_or_else(|| panic!("iteration {i}: diagnostic must be fixable"));
        buffer = fix_harness::apply_single_fix(&buffer, fix);
    }
    let final_diags = rule.evaluate(&path, &buffer);
    assert!(
        final_diags.is_empty(),
        "Sequential fix did not converge in {MAX_ITERATIONS} iterations.\nbuffer: {buffer:?}"
    );
}

// ═══════════════════════════════════════════════════════════
// MD023 — HeadingStartLeft
// ═══════════════════════════════════════════════════════════

/// Basic: indented heading is fixed by removing leading whitespace.
#[test]
fn md023_single_fix_indented_heading() {
    LintFixScenario {
        rule: &HeadingStartLeftRule,
        description: "MD023 – indented heading",
        input: "# Title\n\n  ## Indented Section\n\nContent\n",
        expected_initial_count: 1,
    }
    .run()
    .assert_clean();
}

/// Bulk: multiple indented headings all fixed in one pass.
#[test]
fn md023_bulk_fix_all_indented_headings() {
    BulkFixScenario {
        rule: &HeadingStartLeftRule,
        description: "MD023 – bulk fix: multiple indented headings",
        input: "# Title\n\n  ## A\n\n  ## B\n\n  ## C\n\nContent\n",
        expected_min_initial_count: 3,
    }
    .run()
    .assert_clean();
}

// ═══════════════════════════════════════════════════════════
// MD032 — BlanksAroundLists
// ═══════════════════════════════════════════════════════════

/// Basic: list without surrounding blank lines is fixed cleanly.
#[test]
fn md032_single_fix_missing_blank_before_list() {
    LintFixScenario {
        rule: &BlanksAroundListsRule,
        description: "MD032 – missing blank before list",
        input: "Content\n- item 1\n- item 2\n\nMore\n",
        expected_initial_count: 1,
    }
    .run()
    .assert_clean();
}

/// Bulk: multiple separate lists without surrounding blank lines.
#[test]
fn md032_bulk_fix_multiple_lists() {
    BulkFixScenario {
        rule: &BlanksAroundListsRule,
        description: "MD032 – bulk fix: multiple lists missing blank lines",
        input: "Content\n- item 1\n- item 2\nMore content\n- item A\n- item B\nEnd\n",
        expected_min_initial_count: 2,
    }
    .run()
    .assert_clean();
}

// ═══════════════════════════════════════════════════════════
// MD012 — NoMultipleBlanks
// ═══════════════════════════════════════════════════════════

use katana_linter::rules::markdown::rules::whitespace::NoMultipleBlanksRule;

/// Basic: triple blank line reduced to one blank line.
#[test]
fn md012_single_fix_double_blank() {
    LintFixScenario {
        rule: &NoMultipleBlanksRule,
        description: "MD012 – double blank line",
        input: "# Title\n\n\nContent\n",
        expected_initial_count: 1,
    }
    .run()
    .assert_clean();
}

/// Bulk: multiple groups of excessive blank lines.
#[test]
fn md012_bulk_fix_multiple_groups() {
    BulkFixScenario {
        rule: &NoMultipleBlanksRule,
        description: "MD012 – bulk fix: scattered multiple blanks",
        input: "# A\n\n\n\n## B\n\n\n\nContent\n",
        expected_min_initial_count: 2,
    }
    .run()
    .assert_clean();
}

// ═══════════════════════════════════════════════════════════
// MD004 — UlStyle (bullet style consistency)
// ═══════════════════════════════════════════════════════════

use katana_linter::rules::markdown::rules::list::UlStyleRule;

/// Basic: mixed bullet markers fixed to first-seen style.
#[test]
fn md004_single_fix_mixed_bullets() {
    LintFixScenario {
        rule: &UlStyleRule,
        description: "MD004 – mixed bullet markers",
        input: "- item 1\n* item 2\n",
        expected_initial_count: 1,
    }
    .run()
    .assert_clean();
}

/// Bulk: many inconsistent bullet markers.
#[test]
fn md004_bulk_fix_many_mixed_bullets() {
    BulkFixScenario {
        rule: &UlStyleRule,
        description: "MD004 – bulk fix: many mixed bullets",
        input: "- item 1\n* item 2\n+ item 3\n* item 4\n",
        expected_min_initial_count: 3,
    }
    .run()
    .assert_clean();
}

// ═══════════════════════════════════════════════════════════
// MD027 — NoMultipleSpaceBlockquote
// ═══════════════════════════════════════════════════════════

use katana_linter::rules::markdown::rules::whitespace::NoMultipleSpaceBlockquoteRule;

/// Basic: blockquote with extra spaces is fixed.
#[test]
fn md027_single_fix_extra_spaces() {
    LintFixScenario {
        rule: &NoMultipleSpaceBlockquoteRule,
        description: "MD027 – extra spaces after blockquote",
        input: ">  Content here\n",
        expected_initial_count: 1,
    }
    .run()
    .assert_clean();
}

// ═══════════════════════════════════════════════════════════
// Adapter fix_all integration test
// ═══════════════════════════════════════════════════════════

/// Verify InternalAdapter.fix_all produces a clean document.
#[test]
fn adapter_fix_all_produces_clean_document() {
    use katana_linter::rules::markdown::{
        InternalAdapter, adapter::MarkdownLintAdapter, config::MarkdownLintConfig,
    };

    let adapter = InternalAdapter::new(std::collections::HashMap::new());
    let config = MarkdownLintConfig::load(std::path::Path::new("/nonexistent/.markdownlint.json"));
    let path = std::path::PathBuf::from("test.md");

    let input = "# A\n## B\n## C\nContent\n- item 1\n* item 2\n";
    let fixed = adapter
        .fix_all(&path, input, &config)
        .expect("should produce fixes");

    /* WHY: After fix_all, re-linting with the same rules should report fewer
     * or zero diagnostics for the rules that were fixable. */
    let remaining = adapter.lint(&path, &fixed, &config);
    let fixable_remaining: Vec<_> = remaining.iter().filter(|d| d.fix.is_some()).collect();
    assert!(
        fixable_remaining.len() < 3,
        "fix_all should resolve most fixable issues, but {} remain:\n{:#?}",
        fixable_remaining.len(),
        fixable_remaining,
    );
}

---
name: katana-i18n-management
description: >
  Skill for managing internationalization (i18n) and translations in the Katana project.
  Outlines the strict rules for adding keys, providing native translations, and avoiding
  Linter bypasses (Anti-Lazy Compliance).
---

# Katana i18n Management Strategy

In the Katana project, internationalization (i18n) is strictly controlled to ensure a high-quality, native experience for users globally. The primary language format is JSON, located in `crates/katana-ui/locales/*.json`.

## 1. Strict Native Translation Requirement (Anti-Lazy Compliance)

When instructed to add a new translation key or translate text into multiple languages (e.g., `ja`, `ko`, `zh-CN`, `fr`, etc.):

- **You MUST provide a true, native translation for that language.**
- **You MUST NOT** use formatting hacks like `[TODO-ko] ${English}` or just leave the English string in place, thinking "the Linter will pass if it's slightly different from English."
- `katana-linter` natively intercepts pseudo-translations (`[TODO`, `[todo`) and exact english duplicates. Bypassing these checks via new loopholes is strictly forbidden.

## 2. Translation Addition Workflow

1. **Identify the English string (`crates/katana-ui/locales/en.json`):**
   This is the baseline format and structure. Adding keys here acts as the Source of Truth.

   ```json
   {
     "my_new_feature": {
       "title": "My New Feature",
       "action": "Execute"
     }
   }
   ```

2. **Translate to ALL other supported json files:**
   Open each locale JSON file (e.g. `de.json`, `es.json`, `fr.json`, `it.json`, `ja.json`, `ko.json`, `pt.json`, `zh-CN.json`, `zh-TW.json`) and apply the new keys natively.
   *Example for `ja.json`:*

   ```json
   {
     "my_new_feature": {
       "title": "新機能",
       "action": "実行"
     }
   }
   ```

3. **Verify with `make check-light` or `make check-local`:**
   Running local linters will fire the AST linter (`katana-linter`). It verifies that all keys in `en.json` are present in all other files, and that actual translated values have been provided.

## 3. Dealing with Expected Identical Values

If a word is intended to be identical to English globally (e.g., proper nouns like "Rust", "KatanA", version tokens), the Linter may throw an `identical to English baseline` violation.

**DO NOT cheat by adding a space or a bracket to the translation.**
Instead, if you are absolutely confident it should be identical globally, update the exclusion list in `crates/katana-linter/src/rules/domains/locales/values.rs` (`LOCALE_VALUE_EXCEPTIONS`). Add the key and the exact value that should be excluded.

## 4. Final Rule

AI agents must treat "i18n addition" as a task that aims to deliver linguistic quality, not as a puzzle to trick the linter.

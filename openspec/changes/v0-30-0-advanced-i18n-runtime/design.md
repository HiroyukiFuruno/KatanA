## Context

The current i18n implementation loads JSON into generated Rust structs and exposes `I18nOps::get()` plus `I18nOps::tf(template, params)`. This is fast and type-friendly for static labels, but it has two problems that matter for a desktop app:

1. Language selection can still panic when the selected language is not represented by the embedded dictionary.
2. Formatting is plain string replacement, so messages such as result counts or problem counts cannot express locale-aware plural forms.

PR #236 also mentioned `task_todo` translations, but current locale files already contain localized values such as `Aufgabe [ ]`, `Tarea [ ]`, `Tarefa [ ]`, and `Attività [ ]`. That item is stale and should not drive a new task.

## Goals

- Prevent unsupported language values from crashing application startup or UI rendering.
- Add an i18n engine adapter so KatanA UI call sites do not depend directly on Fluent, ICU, or JSON internals.
- Preserve the existing static JSON locale path while richer formatting is introduced.
- Provide a structured formatting API for plural and parameterized messages.
- Add tests and linter coverage that prove locale quality against current locale files.

## Non-Goals

- Rewriting every locale file to FTL in a single task.
- Removing all generated `I18nMessages` structs immediately.
- Re-translating all existing copy as part of this runtime migration.
- Using local LLM translation overlay for static UI strings; that belongs to v0.25.0.

## Decisions

### I18n Engine Adapter

Introduce an adapter such as `I18nEngine` / `MessageFormatter` that owns language fallback, message lookup, and formatting. UI code may continue to use convenient wrappers, but those wrappers should call the adapter instead of performing formatting directly.

### Safe Language Fallback

Unsupported language codes fall back to the configured default language, currently English. Embedded locale corruption can still fail tests or startup because it is a build-quality problem, but user-provided language values must not panic the application.

### Structured Formatting Before Full File Migration

Add a typed formatting API before converting all locale storage. The initial implementation may map current JSON strings into the adapter and only route selected plural-sensitive messages through Fluent/ICU-style formatting. This keeps the migration small enough to verify.

### Candidate Engine Evaluation

The implementation should compare `fluent-rs` and ICU4X message-formatting options behind the adapter. The selection criteria are:

- supports plural categories and named arguments;
- works offline and embedded in the desktop binary;
- keeps type-safe or lintable message identifiers;
- has acceptable binary-size and startup-cost impact;
- can coexist with current JSON locales while migrating.

### Locale Quality Checks Use Current Data

The previous `task_todo` claim is stale, so the acceptance path should use scripted checks over current locale files. The checks should reject pseudo-translations and missing keys, and they should identify plural-sensitive messages that still use plain string replacement.

## Risks / Trade-offs

- **Risk: losing type safety** - Moving from nested structs to string keys can make missing keys easier to introduce. Mitigate with generated key constants or linter rules.
- **Risk: migration churn** - Full FTL conversion can create a noisy diff. Mitigate by using an adapter and migrating high-value messages first.
- **Risk: over-fitting to one engine** - Fluent and ICU differ in model and file format. Mitigate by keeping UI code on KatanA-owned formatter types.

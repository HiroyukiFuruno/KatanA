## Why

`v0.18.0` は現在進行中の `windows-linux-support` change で cross-platform 基盤を整える計画だが、その次に控えている diagnostics、menu、shortcut、editor、local LLM の草案は依存関係と難易度が大きく異なる。現状の実装には土台が点在している一方で、どの concern をどの minor version で切るかが未固定のため、scope creep と設計の衝突を先に抑える roadmap artifact が必要である。

## What Changes

- `v0.19.0` 以降の大枠を OpenSpec の正式 artifact として定義し、post-`v0.18.0` の release roadmap を固定する
- 次の primary concern を minor version ごとに割り当てる
  - `v0.19.0`: markdownlint-compatible diagnostics surface
  - `v0.20.0`: menu expansion
  - `v0.21.0`: customizable shortcut system
  - `v0.22.0`: editor authoring enhancements and image asset workflow
  - `v0.23.0`: local LLM foundation and lint autofix
  - `v0.24.0`: local LLM document generation
  - `v0.25.0`: local LLM-based translation overlay for dynamic/external English text
- 各 roadmap entry に対して、goal、affected modules/specs、prerequisites、Definition of Ready、Definition of Done、open questions を明示する
- 初期スコープを守るための方針を固定する
  - `v0.19.0` では full markdownlint engine parity ではなく supported rule subset の official ID / message sync を優先する
  - `v0.22.0` では full WYSIWYG editor ではなく Markdown source-first authoring UX を優先する
  - `v0.23.0` では bundled inference runtime ではなく local endpoint / provider abstraction 上の統合を優先する
  - `v0.25.0` では app 既存 i18n を置き換えず、dynamic/external strings の補助翻訳に限定する

## Capabilities

### New Capabilities

- `release-roadmap`: post-`v0.18.0` の minor release plan を versioned artifact として保持し、各 release entry の scope、dependency、DoR、DoD、open questions を追跡する

### Modified Capabilities

## Impact

- 主な影響範囲は `openspec/changes/v0-19-0-post-v0-18-roadmap/*` の artifact 一式
- roadmap の根拠として、次の既存実装・spec を参照する
  - `crates/katana-linter/src/markdown.rs`
  - `crates/katana-ui/src/views/panels/problems.rs`
  - `crates/katana-ui/src/macos_menu.m`
  - `crates/katana-ui/src/shell_ui.rs`
  - `crates/katana-ui/src/views/panels/editor/ui.rs`
  - `crates/katana-core/src/ai/mod.rs`
  - `crates/katana-platform/src/settings/*`
  - `openspec/specs/menu-enhancement/spec.md`
  - `openspec/specs/markdown-authoring/spec.md`
  - `openspec/specs/local-asset-preview/spec.md`
  - `openspec/specs/ai-provider-abstraction/spec.md`
  - `openspec/specs/i18n/spec.md`
- follow-up implementation は、この roadmap をもとに各 minor version ごとの dedicated OpenSpec change へ分割する前提とする

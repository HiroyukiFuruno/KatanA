## Why

Preview は KatanA の主要体験だが、現状は `katana-ui` と parser / renderer / vendor fork の境界が弱く、後続の preview-driven editing や renderer migration の足場として脆い。まず user-visible behavior を変えずに、preview 入出力と metadata の contract を整理する。

## What Changes

- Native preview を KatanA-owned adapter API 経由で扱う。
- `katana-ui` が parser token、renderer internals、vendor fork 固有 API に直接依存しない境界を作る。
- TOC、scroll sync、block highlight、search、action hook に必要な metadata を renderer-neutral DTO として定義する。
- 現行 renderer は最初の adapter implementation として包み、preview の見た目や操作は変えない。
- preview-driven editing はこの change に含めない。

## Capabilities

### New Capabilities

- `preview-adapter-contract`: Native preview の入力、出力、metadata、action を adapter contract として定義する。

### Modified Capabilities

- `preview-rendering`: Preview rendering call site が adapter API を利用するように移行する。

## Impact

- `crates/katana-ui/src/preview_pane/*`
- `crates/katana-ui/src/views/panels/preview/*`
- `crates/katana-ui/src/views/panels/toc/*`
- scroll sync / block anchor / heading anchor logic
- preview integration tests and metadata contract tests

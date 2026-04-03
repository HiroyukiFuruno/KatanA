## Context

現行コードには menu と command の断片が既にある。

- `crates/katana-ui/src/macos_menu.m`
  - File / View / Settings / Help の native menu がある
  - ただし command coverage は限定的で、将来の拡張を前提にした registry はない
- `crates/katana-ui/src/state/command_palette_providers.rs`
  - command palette には settings、workspace、close all、refresh、updates、about がある
  - menu と palette が同じ source of truth を見ていない
- `crates/katana-ui/src/app_state.rs`
  - export、diagnostics、workspace file ops など多くの `AppAction` がある
  - しかし user-facing command metadata は action に付随していない

このまま shortcut customization へ進むと、menu、palette、shortcut editor が別々の command 名・availability・grouping を持つことになる。`v0.19.0` ではまず command inventory を正規化する。

## Goals / Non-Goals

**Goals:**

- menu、palette、future shortcut editor が共有する command inventory を定義する
- File / View / Help の command grouping と availability を整理する
- macOS native menu と non-macOS command surface の parity を上げる

**Non-Goals:**

- user-customizable shortcut の実装
- すべての hidden/internal action を menu へ露出すること
- Window / Tools menu の full design 確定

## Decisions

### 1. command inventory を `AppAction` とは別の metadata layer として持つ

`AppAction` は実行 payload として有用だが、label、group、visibility、availability まで持たせると肥大化する。そこで user-facing metadata layer を別に持つ。

- 採用理由:
  - menu、palette、shortcut editor で再利用しやすい
  - availability 判定を UI surface ごとに重複させずに済む
- 代替案:
  - 各 surface で個別に command 一覧を持つ: drift しやすく不採用

### 2. `v0.19.0` は File / View / Help を primary scope とする

ユーザー草案に沿って、まずは File / View / Help の充実を進める。Settings は既存導線があるため secondary concern とする。

- 採用理由:
  - user-facing value が高い
  - scope が分かりやすい
- 代替案:
  - 全 menu を同時に設計する: `v0.20.0` を押し下げるため不採用

### 3. disabled state を command inventory で扱う

workspace 未選択、active document 不在、dirty state 不在などで command availability が変わる。これを menu 側の ad-hoc 条件にしない。

- 採用理由:
  - parity と testing を揃えやすい
- 代替案:
  - native menu と palette が個別に availability を判定する: surface 間で差分が出るため不採用

## Risks / Trade-offs

- **[Risk] command inventory の導入が `v0.19.0` を抽象化だけで終わらせる**  
  -> Mitigation: File / View / Help の実際の command exposure まで Done に含める
- **[Risk] non-macOS surface と native menu の表現差が残る**  
  -> Mitigation: parity は command coverage を優先し、表現差は許容する
- **[Risk] Window / Tools 相当の menu が後から必要になる**  
  -> Mitigation: inventory は grouping 拡張可能な shape にする

## Migration Plan

1. 現行 menu / palette / top bar の command を棚卸しする
2. shared command inventory を導入する
3. File / View / Help の entries を inventory から生成する
4. docs と palette labels を inventory source of truth に寄せる

## Open Questions

- Export を File に入れるか独立 group にするか
- Help に docs / GitHub / release notes をどこまで含めるか
- Window / Tools 相当を `v0.19.0` に含めるかは後続判断でよいか

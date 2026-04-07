## Why

現在の KatanA は単一の白基調 SVG アイコンセットを前提としており、ユーザーが見た目の方向性を切り替える余地がない。アイコン自体の容量は小さく、git 管理下で複数の審査済みアイコンパックを同梱した方が、再現性、ライセンス管理、レビュー、リリース安全性の面で現実的である。

## What Changes

- アイコンを git 管理下の built-in icon pack として管理し、実行時ダウンロードやユーザー持ち込み適用を初期スコープから外す
- 既存のアイコンセットを `katana-icon` として定義し、default pack にする
- `katana-icon` に加えて、commercial use を阻害しない permissive license を持つ curated pack を 5 種類同梱する
- selected pack ごとに icon render policy を持てるようにし、既存の monochrome tint 前提に加えて colorful icon を扱えるようにする
- third-party pack と 1:1 で互換しない icon は、その pack の visual language に寄せた KatanA authored icon で補完する
- 設定画面に icon pack の選択 UI と preview を追加し、再起動なしで即時反映する
- pack ごとの license / provenance / override 方針を repository 内ドキュメントとして残す

## Capabilities

### New Capabilities

- `icon-theme-packs`: built-in icon pack の選択、fallback、color-aware rendering、pack completeness を扱う

### Modified Capabilities

- `settings-persistence`: 選択中の icon pack を設定として永続化する

## Impact

- 主な影響範囲は `assets/icons`、新設する `assets/icon-packs/*`、`crates/katana-ui/src/icon.rs`、`crates/katana-ui/src/svg_loader/*`、設定 UI、settings schema、license / asset provenance docs
- permissive license の curated pack とそのライセンス文書を repository に同梱する
- icon tint 前提の linter / validation と rendering policy に変更が入る可能性がある

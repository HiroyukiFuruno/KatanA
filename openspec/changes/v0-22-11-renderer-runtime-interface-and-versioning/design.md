## Context

v0.22.10 では、OS の Chrome / Chromium アプリに依存しない高速な Mermaid 描画として Rust 管理 JS を評価している。公式 Mermaid.js を使えるため表示互換性の見込みはあるが、KatanA 本体が Mermaid.js の DOM / SVG 互換、版（version）管理、公式比較画像、更新手順まで抱えると責務が広がりすぎる。

KatanA は Markdown のプレビュー、テーマ、キャッシュ、export 操作を持つ。Mermaid / Draw.io / export 描画の専門性は `katana-renderer` に分離する前提で、まず KatanA 内の接続境界（interface）を整理する。

## Goals

- Mermaid.js に渡す値は Mermaid.js config と互換のある形で保持する。
- KatanA 独自のサイズ制約、テーマ適用、cache policy、診断情報は Mermaid.js config の外側に置く。
- 利用する Mermaid.js の版を固定し、実行時に CDN や OS アプリへ依存しない。
- Mermaid.js の版、checksum、描画 runtime profile を cache key と検証証跡に含める。
- `katana-renderer` 分離時に移す責務と、KatanA 側に残す責務を文書化する。
- Draw.io 描画と HTML / PDF / PNG / JPEG export も、同じ runtime 所有境界の懸念として扱う。
- `mmdc` より軽く速い描画体験を、初回描画と連続描画の計測で維持できるようにする。

## Non-Goals

- この change では `katana-renderer` repository 自体を作成しない。
- この change では Mermaid.js の全表示差分を追加補正しない。表示補正は v0.22.10 の責務とする。
- この change では Draw.io と HTML / PDF / PNG / JPEG export の完全移行を完了条件にしない。
- OS にインストールされた Chrome / Chromium アプリや `headless_chrome` へ戻さない。
- 実行時の退避経路（fallback）は作らない。

## 接続境界

KatanA から Mermaid runtime へ渡す値は、次の4層に分ける。

1. `source`: Mermaid の生テキスト。
1. `config`: Mermaid.js にそのまま渡せる config。`theme`、`themeVariables`、`securityLevel`、`htmlLabels`、diagram-specific config を含める。
1. `policy`: KatanA 独自の描画制約。最大幅、最大高さ、余白、背景、viewport、cache profile を含める。
1. `context`: KatanA 側のテーマ snapshot、文書情報、診断表示用の metadata を含める。

戻り値は、少なくとも次を含む。

- 描画済み SVG
- SVG の幅、高さ、viewBox
- Mermaid.js 版
- renderer profile
- warnings / errors
- cache key へ入れる fingerprint

## Preview 分離との関係

将来 preview を別 module / repository へ分離する場合も、依存方向は次に固定する。

```text
KatanA shell / preview -> katana-renderer
katana-renderer -> preview へは依存しない
```

`katana-renderer` は egui、preview widget、KatanA UI state を参照しない。preview 側は Markdown AST、テーマ snapshot、スクロール同期などを持ち、図形や文書出力に必要な部分だけを中立 DTO（データだけの型）として `katana-renderer` へ渡す。

これにより、preview が後で分離されても、preview が `katana-renderer` を利用する構造になり、循環依存にはならない。

## Mermaid.js 版固定

無印 `~/.local/katana/mermaid.min.js` は、どの公式版で描画したかが追えないため採用しない。v0.22.11 では一時的に KatanA repository 内で次のように管理する。

```text
vendor/mermaid/<version>/mermaid.min.js
vendor/mermaid/<version>/mermaid.min.js.sha256
```

更新は `make mermaid-js-update VERSION=<version>` のような入口に集約する。更新時は埋め込み JS、checksum、公式比較画像、cache profile を同時に更新する。

## `katana-renderer` 分離境界

将来の `katana-renderer` は次を所有する。

- Mermaid.js の版固定と更新手順
- Rust 管理 JS runtime
- DOM / SVG / layout shim
- SVG 正規化
- 公式 Mermaid.js との比較画像生成
- runtime profile と診断情報

`katana-renderer` は library と CLI の両方を持てる構造にする。KatanA は library API を使い、CLI は単体 render、公式比較画像更新、性能計測、外部ツール利用の入口として扱う。CLI は core API の薄い利用者に留め、KatanA 専用の preview 状態や UI 型を持ち込まない。

想定する CLI 例:

```bash
katana-renderer mermaid render --input diagram.mmd --output diagram.svg
katana-renderer mermaid reference-update --fixtures assets/fixtures/mermaid_all
katana-renderer mermaid bench --fixtures assets/fixtures/mermaid_all
```

KatanA に残すものは次に限定する。

- Markdown から Mermaid block を抽出する処理
- 現在のテーマ snapshot の作成
- preview / export UI
- renderer へ渡す config / policy の組み立て
- cache 保存先と表示 fallback

## Draw.io と export

Draw.io 描画、HTML export、HTML から PDF / PNG / JPEG への変換は、Mermaid と同じく「表示できるか」だけではなく、OS 非依存、速度、正確性、配布安定性で判定する必要がある。

v0.22.11 では完全移行ではなく、次を決める。

- Draw.io が Mermaid と同じ runtime interface に乗るか、別の描画 backend として扱うか。
- HTML / PDF / PNG / JPEG export が、diagram runtime と同じ所有境界で扱えるか。
- 未接続の export は黙って失敗させず、明示的な未対応状態として扱うか。

## Risks

- interface 整理と表示補正を同じ change に混ぜると、比較結果の評価軸が曖昧になる。
- Mermaid.js 版固定を KatanA 本体に長く残すと、分離予定の責務が再び固定化される。
- Draw.io と export を同時に完全移行しようとすると、v0.22.11 の主目的である Mermaid 分離境界がぼやける。
- Rust 管理 JS は描画待ちを体感しないほど速い可能性が高いが、正確性は公式 Mermaid.js 比較で確認し続ける必要がある。

## Verification

- OpenSpec validation を通す。
- Mermaid runtime interface の単体テストを追加する。
- cache key に Mermaid.js 版と renderer profile が含まれることを確認する。
- `make mermaid-diagram-update` が固定版 Mermaid.js を使うことを確認する。
- Draw.io / export は、未接続または後続移管の扱いが文書と実装で矛盾しないことを確認する。
- `mmdc` との初回描画 / 連続描画の比較を、少なくとも代表 fixture で記録できるようにする。
- CLI は core API の利用者として設計し、KatanA runtime が CLI 実行に依存しないことを確認する。

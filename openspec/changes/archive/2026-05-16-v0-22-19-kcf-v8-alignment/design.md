## Context

KatanA v0.22.18 時点の作業領域（workspace）は `katana-canvas-forge = "0.1.6"` と `v8 = "=139.0.0"` を参照している。一方で、同じ作業領域に入った `katana-diagram-renderer = "0.1.0"` は `v8 = "=147.4.0"` を要求する。

V8 実行環境（runtime）はプロセス全体（process-global）の初期化状態を持つため、KatanA の 1 プロセス内で異なる `v8` crate バージョンを安全に併存させる設計にはしない。現状の不整合では Mermaid プレビューのワーカー（worker）が描画前に停止し、画面上は `[Mermaid] Diagram render worker disconnected before producing a result.` と表示される。

kcf 側は課題（issue）[HiroyukiFuruno/katana-canvas-forge#15](https://github.com/HiroyukiFuruno/katana-canvas-forge/issues/15) の対応として v0.1.7 を公開済みであり、リリースノート（release note）でも `v0.1.7: bump v8 to =147.4.0 and follow API` が確認できる。KatanA 側の課題は [#293](https://github.com/HiroyukiFuruno/KatanA/issues/293)。

実装時点では、`mathjax_svg = "3.2.0"` も `v8 = "139.0.0"` を要求していた。これは図形描画の依存ではないが、同じ KatanA プロセス内で読み込まれる V8 依存関係（V8-backed dependency）なので、単一 V8 の完了条件に含める。

当初は `mathjax_svg` を `vendor/mathjax_svg` へ置いて V8 147.4.0 へ追従する方針だったが、pre-push 検証で `mathjax_svg` と図形描画器が同じプロセス内で別々に V8 を初期化し、V8 のグローバル状態が poisoned になることを確認した。そのため、`mathjax_svg` の公開 API は維持しつつ、ローカルパッチ内の実行環境を QuickJS へ差し替える。

また、ユーザーレビュー用の `scripts/screenshot` はリポジトリ直下とは別のローカル `Cargo.lock`（gitignore 対象）を持つ独立 manifest である。この manifest が crates.io 版 `mathjax_svg` を解決すると、スクリーンショット実行時だけ古い MathJax V8 経路が再流入する。

## Goals / Non-Goals

**Goals:**

- KatanA v0.22.19 で `katana-canvas-forge` を v0.1.7 へ更新する。
- 作業領域の `v8` 固定指定（pin）を `=147.4.0` へ揃える。
- `mathjax_svg` による旧 V8 の再流入と重複 V8 初期化を防ぎ、数式描画の挙動は維持する。
- `scripts/screenshot` の依存解決でも同じ非 V8 MathJax 経路を使う。
- `Cargo.lock` を依存関係グラフ（dependency graph）と一致する状態へ更新する。
- Mermaid / Draw.io プレビューがワーカー起動前に停止しないことを確認する。
- HTML / PDF / PNG / JPEG 出力（export）が kcf 0.1.7 でも回帰しないことを確認する。

**Non-Goals:**

- KatanA 側へ V8 初期化の退避経路やバージョン切り替え層を追加すること。
- `katana-diagram-renderer` を古い V8 へ下げること。
- kcf / kdr の内部描画器（renderer）実装を KatanA 側へ戻すこと。
- 新しい図形言語、出力形式、画面操作（UI）を追加すること。

## Decisions

1. kcf v0.1.7 を取り込み、KatanA 側で独自の回避策（workaround）を入れない。

   不整合の原因は kcf 0.1.6 と kdr 0.1.0 の `v8` 固定指定差分であり、kcf 側は v0.1.7 で同じ `=147.4.0` へ追従済みである。KatanA 側でワーカー再起動や退避経路（fallback）を増やすと、根本原因を残したまま描画経路が複雑になる。

2. 作業領域の `v8` 固定指定は `=147.4.0` に統一する。

   V8 は複数バージョンを同一プロセスで扱う前提にしない。`Cargo.toml` の作業領域依存関係（workspace dependency）を単一バージョンへ寄せ、`cargo tree -i v8` で参照元を確認する。

3. ロックファイル（lockfile）更新は `cargo update -p v8 -p katana-canvas-forge` を基本にする。

   依存関係更新の範囲を課題 #293 の対象へ限定し、関係のないクレート（crate）更新を混ぜない。必要な間接依存更新が出た場合は、`cargo update` の出力と `Cargo.lock` 差分で理由を確認する。

4. `mathjax_svg` はローカルパッチ内で非 V8 実装へ置き換える。

   V8 は初期化状態をプロセス全体に持つため、同じ V8 バージョンへ寄せても、複数 crate が別々に `initialize_platform` する構成は安全ではない。`mathjax_svg` の JavaScript bundle は維持し、Rust 側の実行環境だけ QuickJS へ差し替えることで、呼び出し側の API と数式描画の出力経路を大きく変えずに V8 初期化を除外する。数式描画の互換性は `katana-ui` の MathJax SVG 生成テストで確認する。

5. 検証はプレビューと出力の両方で行う。

   今回のユーザー影響はプレビュー上のワーカー切断だが、kcf は HTML / PDF / PNG / JPEG 出力にも関与する。Mermaid / Draw.io プレビューの回帰に加えて、出力経路で kcf 0.1.7 が動作することを確認する。

6. `scripts/screenshot` にも非 V8 `mathjax_svg` patch を追加する。

   `scripts/screenshot` は独立 manifest なので、リポジトリ直下の `[patch.crates-io]` だけでは反映されない。ユーザーレビュー前の自動スクリーンショット確認で旧 V8 が再流入しないよう、同じ path patch を `scripts/screenshot/Cargo.toml` にも置く。

## Risks / Trade-offs

- Risk: `v8 = "=147.4.0"` への更新で API 差分が KatanA 側の直接利用箇所に出る
  -> `cargo check` と `katana-core` の図形 / 出力関連テストを先に通し、必要な修正は V8 API 追従に限定する。

- Risk: kcf v0.1.7 の間接依存更新が想定より広がる
  -> `Cargo.lock` 差分を確認し、課題 #293 に必要な依存関係以外が更新されていれば作業を止めて判断する。

- Risk: `mathjax_svg` の QuickJS パッチで数式描画が壊れる
  -> `katana-ui` 側に MathJax SVG 生成の回帰テストを追加し、inline / block の最小出力を確認する。

- Risk: アプリ本体は直っていてもスクリーンショット runner だけ旧 MathJax 経路を引く
  -> `scripts/screenshot/Cargo.toml` が同じ `vendor/mathjax_svg` patch を使うことと、スクリーンショット実行結果を確認する。

- Risk: プレビューは直るが出力経路だけが見落とされる
  -> HTML / PDF / PNG / JPEG 出力の回帰確認を tasks.md の完了条件（Definition of Done）に含める。

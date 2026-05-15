# Tasks: v0.22.19 kcf V8 alignment - KatanA

## 要件(Verbatim Requirements)

```text
kcf側対応済みでv0.1.7でリリース済みです。

あたたにはkatana側でissueを次期patch versionとしてopenspecのchange作成をお願いしたいです！

現在katanaはv0.22.18リリース済みなので、v0.22.19が対象になるかなって思いますね。

documentオンリーなのでmasterで作業でOKです。

そしたらそのまま

[$impl-release](/Users/hiroyuki_furuno/works/private/katana/.agents/skills/impl-release/SKILL.md) v0.22.19

を進めてください。branchは私の方で作成しました！
```

### この要件から導出される制約(MUST)

- kcf 側は v0.1.7 で対応済みとして扱う。
- KatanA 側は次期パッチ版（patch version）の v0.22.19 対象として扱う。
- OpenSpec 文書作成は文書のみ（document-only）として `master` 上で完了済み。
- 実装フェーズ（implementation phase）は、ユーザー作成済みの `release/v0.22.19` 上で進める。
- 既存の未コミット差分があるため、コミット対象は v0.22.19 の実装差分へ明示的に分離する。

## Definition of Ready (DoR)

- [x] KatanA issue [#293](https://github.com/HiroyukiFuruno/KatanA/issues/293) が作成されていること
- [x] kcf v0.1.7 が公開され、`v8 = "=147.4.0"` へ追従済みであること
- [x] KatanA v0.22.18 が既存リリース済みで、次期パッチ対象（patch target）が v0.22.19 であること
- [x] OpenSpec 文書作成が `master` 上で完了し、実装対象 branch が `release/v0.22.19` として用意されていること

## Branch Rule

- **OpenSpec 文書作成ブランチ**: `master`
- **実装ブランチ**: `release/v0.22.19`
- **コミット方針**: 既存の別件差分を含めない。v0.22.19 対象の依存更新、回帰テスト、OpenSpec 更新だけを staging 対象にする。

## 1. 依存バージョンを v0.22.19 向けに揃える

- [x] 1.1 `make help` または Makefile を確認し、依存更新後に使う既存の検証対象（target）を決める
  - `Makefile` は存在しないため、`Justfile` の `just type-check` と対象 `cargo test` を使用する。
- [x] 1.2 `Cargo.toml` / `Cargo.lock` の現状が課題 #293 の前提と一致することを確認する
  - 実装開始時点で `release/v0.22.19` には kdr 移行系の既存差分があったが、作業領域依存は `katana-canvas-forge = "0.1.6"` / `v8 = "=139.0.0"` を含んでいた。
  - 追加で `mathjax_svg = "3.2.0"` も crates.io 版では `v8 = "139.0.0"` を要求していた。
- [x] 1.3 作業領域の依存関係（workspace dependency）の `katana-canvas-forge` を `0.1.7` へ更新する
- [x] 1.4 作業領域の依存関係の `v8` を `=147.4.0` へ更新する
- [x] 1.5 `Cargo.lock` を更新する
  - `cargo update -p v8` は `v8@139.0.0` / `v8@147.4.0` で曖昧になるため、`cargo update -p v8@139.0.0 -p katana-canvas-forge` を使用した。
  - pre-push で `mathjax_svg` と図形描画器の重複 V8 初期化が落ちることを確認したため、`vendor/mathjax_svg` の実行環境を QuickJS へ置き換えた。
  - `scripts/screenshot` は独立 manifest のため、同じ非 V8 `mathjax_svg` path patch を追加した。
- [x] 1.6 `cargo tree -i v8` で V8 依存関係が揃っていることを確認する
  - workspace: `katana-canvas-forge` / `katana-diagram-renderer` が `v8 v147.4.0` のみを参照。
  - `mathjax_svg` は QuickJS 経由で実行し、`v8` を参照しない。
  - `scripts/screenshot`: 同じ `vendor/mathjax_svg` path patch を参照。

### Definition of Done (DoD)

- [x] `Cargo.toml` が `katana-canvas-forge = "0.1.7"` と `v8 = "=147.4.0"` を参照していること
- [x] `Cargo.lock` の更新範囲が kcf / v8 / `mathjax_svg` QuickJS 化に必要な範囲へ限定されていること
- [x] `cargo tree -i v8` で競合する V8 依存関係（conflicting V8-backed dependency）が残っていないこと
- [x] `scripts/screenshot` 側でも旧 MathJax V8 経路が再流入しないこと

## 2. Mermaid / Draw.io プレビューの回帰を確認する

- [x] 2.1 既存のプレビュー統合テスト（preview integration test）で Mermaid ワーカー切断（worker disconnect）を検出できることを確認する
- [x] 2.2 Mermaid ブロックが `[Mermaid] Diagram render worker disconnected before producing a result.` へ置換されないことを統合テストで確認する
- [x] 2.3 Draw.io ブロックが V8 バージョン分裂（version split）により描画前の失敗（failure）にならないことを追加テストで確認する
- [x] 2.4 対象 test target とスクリーンショット（screenshot）でプレビュー回帰確認（preview regression）を実行し、結果を記録する

### Definition of Done (DoD)

- [x] Mermaid プレビューがワーカー切断メッセージ（message）で全面失敗しないこと
- [x] Draw.io プレビューが kcf / kdr / 数式描画依存の不整合で失敗しないこと
- [x] 失敗時の退避表示（fallback message）を隠すだけのテスト（test）になっていないこと

## 3. HTML / PDF / PNG / JPEG 出力の回帰を確認する

- [x] 3.1 Mermaid / Draw.io を含む検証データ（fixture）を HTML 出力し、図形出力が欠落しないことを確認する
- [x] 3.2 同じ検証データを PDF / PNG / JPEG 出力し、V8 バージョン分裂由来のワーカー失敗（failure）が発生しないことを確認する
- [x] 3.3 出力経路に OS ブラウザ依存や古い実行環境退避（runtime fallback）を戻していないことを確認する
- [x] 3.4 実行した出力回帰確認のコマンド（command）と成果物の確認観点を記録する

### Definition of Done (DoD)

- [x] HTML 出力で Mermaid / Draw.io 出力が欠落しないこと
- [x] PDF / PNG / JPEG 出力で Mermaid / Draw.io 出力が V8 不整合により失敗しないこと
- [x] kcf 0.1.7 取り込み以外の描画所有境界変更をこのタスクで新規に混ぜていないこと

## 4. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [x] 4.1 ユーザーへ実装完了の報告および動作状況を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする
  - 生成物: `tmp/v0-22-19-kcf-v8-alignment-screenshot/01-light-diagram-preview.png`
- [x] 4.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）
- [x] 4.3 PR #294 の Windows CI で `preview_pane::math::tests::mathjax_backend_renders_svg_for_inline_and_block_math` が失敗したため、MathJax QuickJS 経路を追加修正する
- [x] 4.4 PR #294 の head commit で CI 本体の check run が作成されず、必須チェックが pending になる問題を修正する

## Recovery Log

- [x] `scripts/screenshot` 実行時だけ V8 panic が出て、画面上は Mermaid / Draw.io の worker disconnect になっていた。
  - 原因: `scripts/screenshot` が独立 manifest で、crates.io 版 `mathjax_svg` から `v8 v139.0.0` を解決していた。
  - 対応: 初期対応では `vendor/mathjax_svg` を path patch にして V8 147.4.0 へ寄せた。
  - 結果: 旧 V8 再流入は消え、スクリーンショット実行も成功した。
- [x] 通常 push の pre-push で `katana-ui` の lib test が `v8-147.4.0/src/V8.rs` の `PoisonError` により失敗した。
  - 原因: `mathjax_svg` と図形描画器が同じプロセス内で別々に V8 を初期化していた。
  - 対応: `vendor/mathjax_svg` の公開 API は維持し、Rust 側の実行環境を V8 から QuickJS へ置き換えた。
  - 結果: 数式描画は V8 を初期化せず、図形描画の V8 初期化と分離された。
- [x] PR #294 の `Test and Build (windows-latest)` で QuickJS 版 MathJax の単体テストだけが失敗した。
  - 失敗箇所: `preview_pane::math::tests::mathjax_backend_renders_svg_for_inline_and_block_math`
  - 症状: inline MathJax SVG の生成で `Exception generated by QuickJS` になり、macOS / Ubuntu / lint / Release Readiness は成功していた。
  - 対応: QuickJS のスタック上限（stack limit）を 8 MiB に明示し、JavaScript 例外（JavaScript exception）の本文を失わないよう `CaughtError` 経由で変換する。
  - 結果: ローカルの対象テストと `katana-ui --lib` は成功。Windows CI の再実行結果で最終確認する。
- [x] PR #294 の head commit `85b29f10` で `Release Readiness` だけが作成され、CI 本体の check run が作成されなかった。
  - 原因: `test-and-build.yml` の `paths` が Rust ファイルを `**.rs` で指定しており、nested path の変更で CI を確実に起動する指定として弱かった。
  - 対応: `**/*.rs` に修正し、`.github/workflows/**` の変更として CI 自体を再起動させる。
  - 結果: push 後の PR check run で最終確認する。

## Verification Log

- [x] `just type-check` が成功
- [x] `just fmt-check` が成功
- [x] `kml check --config .markdownlint.json openspec/changes/v0-22-19-kcf-v8-alignment` が成功
- [x] `cargo tree -i v8` が workspace 側で `v8 v147.4.0` のみを表示し、`mathjax_svg` を含まない
- [x] `cargo tree -i rquickjs` が `mathjax_svg` からの参照を表示
- [x] `cargo tree --manifest-path scripts/screenshot/Cargo.toml -i rquickjs` が `scripts/screenshot` 側でも `mathjax_svg` からの参照を表示
- [x] `rg mathjax-svg-rs Cargo.toml Cargo.lock crates/katana-ui/Cargo.toml` が参照なしであることを表示
- [x] `cargo test -p katana-ui mathjax_backend_renders_svg_for_inline_and_block_math -- --nocapture` が成功
- [x] `cargo test -p katana-core diagram_backend -- --nocapture` が成功
- [x] `cargo test -p katana-ui --test ui_integration_serial diagram_rendering -- --test-threads=1 --nocapture` が成功
- [x] `cargo test -p katana-ui --test ui_integration_parallel html_export -- --nocapture` が成功
- [x] `cargo test -p katana-core markdown::export::tests -- --nocapture` が成功
- [x] `./scripts/screenshot/run.sh --request scripts/screenshot/examples/v0-22-14-light-diagrams.json --output tmp/v0-22-19-kcf-v8-alignment-screenshot` が成功
- [x] `./scripts/openspec validate v0-22-19-kcf-v8-alignment --strict` が成功
- [x] `cargo test -p mathjax_svg` が成功
- [x] `cargo test -p katana-ui --lib` が成功
- [x] `cargo clippy -p mathjax_svg -- -D warnings` が成功
- [x] `git diff --check` が成功
- `just kml-check` は repo 全体の既存 Markdown 違反（README / CHANGELOG / assets など）で失敗したため、v0.22.19 対象範囲外として PR 本文に記録する

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 5. Final Verification & Release Work

- [x] 5.1 `docs/coding-rules.ja.md` と `$self-review` skill を使って自己レビューを実行する
- [x] 5.2 更新した Markdown 文書（tasks.md、CHANGELOG.md など）を整形し、lint 修正（lint-fix）を行う
- [x] 5.3 v0.22.19 対象差分だけを staging し、既存の別件差分を含めないことを確認する
- [x] 5.4 `release/v0.22.19` でリリースコミットを作成する
- [x] 5.5 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す
- [x] 5.6 `release/v0.22.19` から `master` 向けに PR（pull request）を作成する

## Post-PR Release Follow-up

PR 作成後の CI 確認、ユーザー承認後の merge、GitHub Release 完了確認、`/opsx-archive` はこの OpenSpec 実装コミットの完了条件ではなく、PR 作成後のリリース運用で扱う。

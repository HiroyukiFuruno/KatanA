## Definition of Ready (DoR)

- [x] proposal.md、design.md、specs が揃っていること
- [x] kcf 側 issue [#4](https://github.com/HiroyukiFuruno/katana-canvas-forge/issues/4) が KatanA 側の前提として記録されていること
- [x] KatanA 側の対応範囲が「kcf 修正版の取り込み」「adapter と回帰テスト」「不要な renderer asset 取得経路の削除」「証跡生成」に限定されていること

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-14-kcf-theme-propagation` またはリリース用統合ブランチ（例: `release/vX.Y.Z`）
- **作業ブランチ**: 標準は `v0-22-14-kcf-theme-propagation-task-x`、リリース用は `feature/v0.22.14-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## 1. kcf 修正版の取り込み準備

- [x] 1.1 kcf 側 issue #4 の修正内容を確認し、`RenderInput` 由来のテーマが Mermaid / Draw.io の実描画へ使われることを kcf 側テストで確認する
- [x] 1.2 kcf の修正版 release version を確認し、KatanA の `katana-canvas-forge` dependency 更新対象を決める
- [x] 1.3 `Cargo.toml` / `Cargo.lock` を kcf 修正版へ更新する
- [x] 1.4 `cargo tree -p katana-canvas-forge --no-dedupe` で `egui` が含まれないことを確認する
- [x] 1.5 kcf issue #4 の修正版が未公開の場合は、KatanA 側で一時的なグローバル同期回避策を入れず、作業を止めて release 待ちまたは kcf 側対応へ切り替える（v0.1.3 公開済みのため停止条件に該当しないことを確認）
- [x] 1.6 kcf が Draw.io / Mermaid の min.js を組み込み済みであることを前提に、KatanA 側の `renderer_assets`、起動時取得、再取得用コマンドパレット（Command Palette）項目、多言語文言、関連 action / state を削除する

### Definition of Done (DoD)

- [x] KatanA が kcf のテーマ伝播対応版を参照していること
- [x] kcf 側で `RenderInput` の light / dark 差分が実描画へ反映されることを確認していること
- [x] KatanA 側に Draw.io / Mermaid の min.js ダウンロード URL、起動時取得、手動修復・再取得導線が残っていないこと
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. KatanA adapter のテーマ伝播

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 2.1 `DiagramThemeSnapshot` から kcf のテーマ入力へ変換する専用 adapter を追加する
- [x] 2.2 Mermaid / Draw.io preview が adapter 経由で light / dark のテーマ名、背景、文字色、塗り、線、矢印、Mermaid theme / Draw.io label color を渡すようにする
- [x] 2.3 `DiagramBlock::render()` と preview dispatch のテーマ入力経路を揃え、片方だけが古い `DiagramColorPreset::current()` に依存しないようにする
- [x] 2.4 export 開始時点の theme snapshot を background thread へ渡し、thread 内でグローバル状態だけを読み直さないようにする
- [x] 2.5 kcf 内部 `DARK_MODE` が true でも、KatanA が light を渡した場合に light 入力が維持される回帰テストを追加する

### Definition of Done (DoD)

- [x] preview / export の両方が同じテーマ変換 adapter を通ること
- [x] light テーマ入力が kcf へ渡されることを unit test で確認できること
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 3. Cache key と kcf metadata の整理

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 3.1 `KCF_MERMAID_BACKEND_VERSION` / `KCF_DRAWIO_BACKEND_VERSION` の古い `0.1.0` 手書き文字列をやめ、実際の kcf version / runtime / renderer profile へ追従する形にする
- [x] 3.2 KatanA の diagram cache key が、実描画で使われる theme fingerprint、kcf runtime、renderer profile の差分で変化することを保証する
- [x] 3.3 kcf の `cache_fingerprint` と KatanA の persistent cache key が、light / dark の差分を同じ意味で扱うことを確認する
- [x] 3.4 既存 cache が dark 配色を再利用する可能性が残る場合は、diagram cache version または key material を更新する
- [x] 3.5 cache key 回帰テストを追加し、同一 source でも light / dark / kcf runtime/profile の差分で key が変わることを確認する

### Definition of Done (DoD)

- [x] kcf dependency version と backend metadata の不一致が残っていないこと
- [x] 古い dark diagram cache が light テーマで再利用されないこと
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 4. Preview / Export の回帰テストと証跡

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 4.1 Mermaid light theme preview の unit / integration test を追加する
- [x] 4.2 Draw.io light theme preview の unit / integration test を追加する
- [x] 4.3 HTML export の Mermaid / Draw.io が current theme と一致する回帰テストを追加する
- [x] 4.4 PDF / PNG / JPEG export の入力 HTML が light テーマの SVG を含むことを確認する対象テストを追加する
- [x] 4.5 `scripts/screenshot` に light テーマで Mermaid / Draw.io を表示する review 用 request を追加する
- [x] 4.6 `./scripts/openspec validate v0-22-14-kcf-theme-propagation --strict` を通す
- [x] 4.7 対象テストと `just check-local` を通す

### Definition of Done (DoD)

- [x] light テーマで Mermaid / Draw.io が dark 的な配色へ戻らないことをテストと screenshot で確認できること
- [x] OpenSpec validate と対象品質ゲートが成功していること
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 5. Linux in-app 自動更新の透過的自己修復

### 要件(絶対遵守)

- Linux 環境で **in-app の「更新 & 再起動」クリック 1 回**だけで自動更新が成立する。
- ユーザーに**いかなる手動操作も要求しない**(`brew reinstall`、ターミナル操作、ファイルリネーム、再ダウンロード、いずれも NG)。
- 既存の被害ユーザー(`katana-desktop` という実行ファイル名で動いている Linuxbrew 経由インストールユーザー)も、次回の更新クリック 1 回で**透過的に自己修復**する(symlink、brew Cellar、ショートカット等を壊さない)。

### Background

`crates/katana-core/src/update/installer.rs:86-93`(Linux 分岐)は、展開先候補ファイル名を `target_app_path.file_name()`(= `std::env::current_exe()` のファイル名)から組み立てている。一方、リリースアセット `KatanA-linux-x86_64.tar.gz`(`scripts/build/package-linux.sh:19` 生成)は**ルート直下に `KatanA` 単体**しか含まない(SHA-256 / `tar -tzvf` で v0.22.13 アセットを実測確認済み)。

`scripts/release/update-linuxbrew.sh:61` の formula が `bin.install "KatanA" => "katana-desktop"` で実行ファイル名をリネームしているため、Linuxbrew 経由でインストールしたユーザーの `current_exe().file_name()` は `katana-desktop` となる。インストーラは `extract_dir/katana-desktop` を探しに行くが、tar 内の実体は `KatanA` なので `exists()` チェックが必ず失敗し `Extracted update does not contain a valid executable` で bail する。同型コードが Windows 分岐(`installer.rs:60-67`)にも存在し、AppImage / リネーム / シンボリックリンクで `current_exe()` の basename がアセット内ファイル名と乖離するあらゆるケースで再発する潜在バグ。

参考: macOS Cask(`scripts/release/update-homebrew.sh:57,111`)は cask token を `katana-desktop` としつつ `app "KatanA Desktop.app"` でリネームせず、アセット構造と一致させている。Linuxbrew formula だけがこの規約を破っている。

### 修正方針

**主修正は `installer.rs`**:

- 展開元はアセットの**既知名 `KatanA` / `KatanA.exe`** を一次参照する(`extract_dir.join("KatanA")` 等)。
- 想定ファイルが無い場合の防御フォールバックとして、`extract_dir` 直下を走査し唯一の通常ファイルを採用(将来のアセット構造変化、複数同梱化への耐性)。
- 展開先(`target_app_path`)は従来通り `current_exe()` のまま。relauncher の `mv {extracted} {target}` がリネーム動作を兼ねた in-place 上書きを行うため、ユーザー側のバイナリ名(`katana-desktop` 等)・symlink・brew 管理状態は一切変わらず、被害ユーザーは更新クリック 1 回で自動修復が完了する。

**副次修正(本質ではないが同 PR で整合性を取る)**:

- `update-linuxbrew.sh:61` の rename を撤去し、新規 install ユーザーの実行ファイル名を他プラットフォームと揃える。既存ユーザーには影響なし(installer.rs 側で吸収されるため)。
- `README.md` / `README.ja.md:126` の Windows Portable ZIP 起動コマンド誤記(`katana-desktop.exe` → `KatanA.exe`)修正。アセット実態に整合。

### DoR

- [x] 上記 Background が最新コードと一致することを確認する(`installer.rs:86-93`、`installer.rs:60-67`、`package-linux.sh:19`、`update-linuxbrew.sh:61`、`README.md:126`、`README.ja.md:126`)。
- [x] Base ブランチが同期済みで、`v0-22-14-kcf-theme-propagation-task-5`(またはリリース統合ブランチ向けの `feature/v0.22.14-task-5`)が作成済みであること。

### Tasks

#### 5.A 主修正: installer.rs によるユーザー操作 0 の自己修復

- [x] 5.1 `crates/katana-core/src/update/installer.rs` の Linux 分岐(86-93 行付近)を以下に変更する:
  - 展開元の一次候補を `extract_dir.join("KatanA")` に固定(`target_app_path.file_name()` 依存を撤去)。
  - 想定ファイルが無い場合は `extract_dir` 直下の通常ファイル(実行可能ビット付き)を走査し、唯一なら採用、複数または 0 件ならディレクトリ内容を含む bail メッセージを返す(actionable error 契約の維持)。
  - 展開先(`target_app_path`)とその後の `generate_relauncher_script` 呼び出しは変更しない。
- [x] 5.2 同ファイルの Windows 分岐(60-67 行付近)も同型の潜在バグであるため、一次候補 `extract_dir.join("KatanA.exe")` + 同じ走査フォールバックに揃える。
- [x] 5.3 macOS 分岐(29-56 行付近)は `.app` バンドル構造のため挙動を変更しないが、フォールバック値 `"KatanA.app"`(31-33 行)が現状の正規バンドル名 `"KatanA Desktop.app"` と乖離している点を確認し、現状アセット名(`"KatanA Desktop.app"`)へ揃える(防御フォールバックの正確性確保。実害は無いが整合)。
- [x] 5.4 Linux/Windows それぞれで、`target_app_path` が `KatanA` 以外(`katana-desktop`、`katana-bin`、AppImage 風名、シンボリックリンク経由の別名など)を指していても `prepare_update` が成功し、`UpdatePreparation.app_bundle_path` が `extract_dir/KatanA`(または `KatanA.exe`)を指すユニットテストを `installer.rs` のテストモジュールへ追加する。
- [x] 5.5 失敗系テストを追加: `extract_dir` 内に想定ファイル無し / 複数の通常ファイル / 0 件 各ケースで bail メッセージにディレクトリ内容が含まれること。
- [x] 5.6 relauncher スクリプト(`scripts.rs` Linux 分岐)の `mv {extracted} {target}` がリネーム動作を兼ねた in-place 上書きを行う既存挙動を、シナリオテスト(target の basename が extracted と異なる入力で、生成スクリプト本文が期待通りの `mv` 命令を含む)で明示的に固定する。`scripts.rs` のロジック自体は変更しない。

#### 5.B 副次修正: 命名規約とドキュメント整合

- [x] 5.7 `scripts/release/update-linuxbrew.sh:61` の `bin.install "KatanA" => "katana-desktop"` を `bin.install "KatanA"` に変更(formula/cask token は `katana-desktop` のまま、パッケージ ID と実行ファイル名を分離)。本変更は新規 install 向けの命名整合化であり、5.A の主修正によって既存ユーザーは影響を受けない(installer.rs 側で吸収される)。
- [x] 5.8 `README.md:126` / `README.ja.md:126` の Windows Portable ZIP 起動コマンド記述を `katana-desktop.exe` → `KatanA.exe` に修正(アセット実態に合わせる)。
- [x] 5.9 `README.md` / `README.ja.md` の Linux Manual Download セクション(`README.md:137-141` 付近)の起動コマンドが `./KatanA` であることを明記(現状曖昧)。

#### 5.C 検証と証跡

- [x] 5.10 `docs/release/update-verification.md` の Linux 検証手順を更新: 「Linuxbrew で `katana-desktop` をインストール → in-app 更新クリック 1 回で自動更新成功 → 再起動後に同じ `katana-desktop` 実行コマンドで新バージョンが起動」を必須シナリオに含める。
- [x] 5.11 ユーザー操作 0 の検証として、被害状態(`current_exe()` が `katana-desktop` を返す状況)を再現する手元手順を `docs/release/update-verification.md` に追記し、`scripts/screenshot` 等で証跡を残す。手動操作を一切求めないことを DoD で確認する。
- [x] 5.12 CHANGELOG(EN/JA)に**変更事実のみ**を簡潔に記録(`changelog-writing` skill 準拠):「Linux 環境で in-app 自動更新が `Extracted update does not contain a valid executable` で失敗する問題を修正。被害ユーザーは次回の更新で自動修復される(手動操作不要)。同型の Windows 潜在バグも併せて修正」。CHANGELOG にはユーザー手順を一切書かない(本修正は手順を要求しないため、書くべき手順自体が存在しない)。
- [x] 5.13 `cargo test -p katana-core update::`、`just check-local`(または該当パッケージのリンタ)、`shellcheck`(プロジェクトで定義されていれば)、`./scripts/openspec validate v0-22-14-kcf-theme-propagation --strict` を通す。
- [x] 5.14 `auto-update` capability の spec delta が必要か確認し、必要なら `openspec/changes/v0-22-14-kcf-theme-propagation/specs/auto-update/spec.md` を追加して「インストーラは展開元をアセット既知名で解決し、展開先のユーザー側ファイル名差異を吸収する」「自動更新はユーザーの手動操作を一切要求しない」を MUST として記述する。

### DoD

- [x] Linuxbrew 経由インストールで `katana-desktop` 実行ファイル名で動いているユーザーが、in-app 更新ボタンクリック 1 回だけで自動更新と再起動を完了できること(手動操作 0 を再現テストで確認)。
- [x] Linux Manual download(`KatanA` のまま起動)ユーザーも同様に自動更新が成功すること(回帰なし)。
- [x] Windows MSI / Portable ZIP の自動更新が引き続き成功すること(同型修正で回帰なし)。
- [x] macOS 自動更新が引き続き成功すること(回帰なし)。
- [x] 追加したユニットテスト(成功 / 失敗系)が CI で安定して通ること。
- [x] README(EN/JA)の Linux / Windows 起動コマンド記述がアセット実態と一致していること。
- [x] CHANGELOG(EN/JA)には変更事実のみが記録され、ユーザー向け復旧手順が記載されていない(本修正が手順を要求しないため)こと。
- [x] `docs/release/update-verification.md` に被害状態の再現と自己修復確認手順が記載されていること。
- [x] OpenSpec validate を通過していること。
- [ ] Execute `/openspec-delivery` workflow (`.codex/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 6. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [/] 6.1 ユーザーへ実装完了の報告および動作状況を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする
- [/] 6.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）
- [/] 6.3 `mermaid` フェンス内の ZenUML が raw Markdown として残る問題を修正する。KatanA 側では `zenuml` ソースを描画対象から除外しない最小修正に限定する。
- [/] 6.4 ZenUML が白表示になる根本原因を特定する。kcf v0.1.3 では出力 SVG の見た目本体が `foreignObject` 依存で、KatanA の既存ネイティブ画像化では描けなかった。kcf v0.1.4 取り込み後は PNG wrapper として描画されるが、背景が不透明な白で返るため、KatanA 側でテーマ背景へ合成する必要がある。
- [/] 6.5 kcf 側 issue [#8](https://github.com/HiroyukiFuruno/katana-canvas-forge/issues/8) を起票する。KatanA 側へ ZenUML 専用の WebView（アプリ内ブラウザ部品）/ Chromium（Chrome 系ブラウザエンジン）/ Playwright（ブラウザ自動操作ランタイム）経路は追加しない。
- [/] 6.6 kcf 側で ZenUML 出力契約が修正された後、`scripts/screenshot` で ZenUML 表示を含むスクリーンショットを生成し、白表示ではなく視認できる図になっていることを確認する。
- [/] 6.7 ZenUML のスクリーンショット確認後に `just check-local` を実行し、all pass まで修正する。
- [/] 6.8 `just check-local` all pass を担保した後、`git commit --no-verify` / `git push --no-verify` の使用を許可済みとして commit & push を実行する。
- [/] 6.9 Markdown に埋め込んだ PNG 等の画像を preview 表示するとき、背景が黒固定になり light 系テーマで不正表示になる問題を修正する。
- [/] 6.10 `mermaid` フェンス内の ZenUML を描画するとき、背景が白固定にならず、選択中テーマに追従するようにする。
- [/] 6.11 `just check-local` の `sample_mermaid_exports_html_pdf_png_and_jpeg_without_chromium` 失敗を修正する。ZenUML を renderer に渡す仕様変更後の raw Mermaid 数を正しく検証し、Linux コンテナのように ZenUML の Node / Playwright 依存が無い環境ではエラーHTMLではなく元のコードブロックを維持する。

---

## KatanA CLI Entry Point

このリポジトリでは OpenSpec の実行入口として `./scripts/openspec` を使用すること。グローバルの `openspec` コマンドが見つからない場合でも未導入と判断してはならない。このラッパーは `bunx @fission-ai/openspec`、次に `npx @fission-ai/openspec` へフォールバックする。

このスキル内で `openspec ...` と書かれているコマンドは、リポジトリルートから `./scripts/openspec ...` として実行する。

## 7. Final Verification & Release Work

- [x] 7.1 Execute self-review using `docs/coding-rules.ja.md` and `$self-review` skill
- [x] 7.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
- [/] 7.3 今回の PR 作成では、ユーザー許可により `git commit --no-verify` / `git push --no-verify` を使用する。品質ゲートは `just check-local` all pass を採用し、例外理由を PR 本文に記録する
- [ ] 7.4 Create PR from Base Feature Branch targeting `master`
- [ ] 7.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
- [ ] 7.6 Merge into master (`gh pr merge --merge --delete-branch`)
- [ ] 7.7 Create `release/v0.22.14` branch from master
- [ ] 7.8 Run `just VERSION=0.22.14 release` and update CHANGELOG (`changelog-writing` skill)
- [ ] 7.9 Create PR from `release/v0.22.14` targeting `master` — Ensure `Release Readiness` CI passes
- [ ] 7.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
- [ ] 7.11 Verify GitHub Release completion and archive this change using `/opsx-archive`

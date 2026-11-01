# v0.22.46 起動クラッシュとエクスプローラフィルター

## 要件(Verbatim Requirements)

> エクスプローラのフィルターの正規表現検索が機能していない。
>
> またkatanaが開くとクラッシュして開けない、致命的な不具合に遭遇した。
> 現在のkatanaのapp版が開くとクラッシュする。
>
> katana repositoryないにある特定のpngを開いたらそうなった。
>
> build版でも再現しそうなので、解析して優先的になおしてください。
>
> 正規表現は類似有りと思うので、致命的な不具合を修正したら一緒のpatch version内で修正してくリリースして下さい。
>
> *.pngで何も表示されなかった。私の知っている正規表現では、全てのpngだけが表示されるはず。
>
> 最新版を取り込まないと起動できないのでリカバリーも必要なわけですが。

### この要件から導出される制約(MUST)

- 起動クラッシュを優先し、正規表現フィルター修正も同じパッチ版で公開する。
- 配布版とソースビルドの経路を確認する。
- `*.png` でPNGだけが表示されるようにする。
- 起動不能な既存インストールの復旧も行う。

### 設計判断時の参照義務

- 設計変更時は上記の要件と整合させ、完了条件を変更しない。

## Definition of Ready

- [x] 配布版 v0.22.45 のアセット構造・checksum・architecture を検査した。
- [x] 保存状態の active_path と配布版の実クラッシュログを確認した。
- [x] master clean、stash 0、既存worktree 1、重複実装なしを確認した。

## Branch Rule

- Base / 作業ブランチ: `release/v0.22.46`。新規worktreeは禁止。
- この変更を同じDraft PRでレビューし、Ready、required checks、merge、公開へ進める。

## DoD・対象範囲・禁止事項

- 起動時に保存された高さ19,067pxのPNGを表示しても描画上限を超えない。
- 単独画像とMarkdown内画像の両経路、横長・縦長・上限境界を回帰検証する。
- エクスプローラ正規表現の原因を具体化し、モード切替・不正入力を回帰検証する。
- 依存候補を棚卸しし、既存のbuild/test/lint/coverageゲートを維持する。
- Draft → review → 指摘取得・修正・reply/resolve → P0/P1ゼロ → Ready → required checks → merge。
- GitHub Release公開成果物を検証し、OpenSpecと作業台帳に証跡を残してローカル整理する。
- gate無効化、master直接編集、別repo編集、未公開依存、remote branch削除は禁止。

## 1. 起動クラッシュ（親担当・最優先）

- [x] 1.1 配布版で再現: `tmp/v0-22-46/installed-startup.log`、exit 101、Y=19067 > 8192。
- [x] 1.2 回帰テストを先に追加し失敗を確認する。
- [x] 1.3 両画像表示経路を修正し、元ファイル・保存タブを保持して再起動を検証する。
- [x] 1.4 保存状態のバックアップ後、active tabだけを切り替え既存配布appの一時起動を検証する（36tabs/履歴保持）。

## 2. 正規表現フィルター（調査委譲・実装は親統合）

- [x] 2.1 相対パス照合、モード、キャッシュ、入力検証を調査する。
- [x] 2.2 ユーザーの具体例も照合し、原因を固定して回帰テスト・修正する。
- [x] 2.3 通常検索の `*.png` をbasename wildcardとして照合し、root/nested/否定/case/単語/regex切替の回帰を通す。

## 3. 依存と検証

- [x] 3.1 直接・推移依存とlockを棚卸しし、最新候補を評価・適用する。
- [x] 3.2 対象回帰・ソースビルドを回収する（通常pushの品質結果は5.2、required CIは5.4で追跡）。

## 4. User Review (Pre-Final Phase)

- [x] 4.1 自動検査と動作状況を提示する（本人実機確認を公開の停止条件にしない）。
- [x] 4.2 受領したフィードバックを作業項目へ反映し、実装と一時復旧を検証する（公開版への直接更新は5.5で追跡）。

- [/] `*.png` の期待を確認し、通常検索のwildcard照合へ反映した。
- [/] 履歴削除ではなくactive tab切替だけで一時復旧した。

## KatanA CLI Entry Point

OpenSpec は `./scripts/openspec`、検証は既存の `just` ターゲットを使用する。

## 5. Final Verification & Release Work

- [x] 5.1 self-review、Markdown整合、CHANGELOG日英同期、version同期。
- [x] 5.2 通常pushでpre-pushを通しDraft PRを作成する。
- [x] 5.3 reviewを実行・取得し、指摘修正・thread reply/resolve・P0/P1ゼロを確認する。
- [x] 5.4 Ready昇格、required checks成功後にmergeする。
- [ ] 5.5 GitHub Release成果物を検証し、既存appを公開版へ直接更新して元PNGの自動復元を検証した後、OpenSpecをarchiveする。
- [ ] 5.6 branch-hygieneでstash/master/ahead-behind/worktreeを再確認する。

## 調査証跡

- インストール済みapp: `/Applications/KatanA Desktop.app`、v0.22.45。
- active PNG: `assets/candidates/katana/legacy-krr-0.4.19/export_png/sample.png`、1280 × 19067。
- 配布アセットSHA-256: `e3de80920177e627674906356e5e86090764a250b93b528d80fc12258a9d2954`。
- 原因: `images.rs` と `local_image_loader/cache.rs` がdecoded画像を無制限に `load_texture` へ渡す。
- 具体例 `*.png` により、通常モードが `*` をliteral照合し、regexモードでも無効patternになることを特定。通常検索にbasename wildcard照合を追加し、正規表現モードは従来どおり標準構文を維持する。

## 検証結果（2026-11-01）

- 画像RED: crash-red.log、上限32に対する19067で失敗。GREEN: local-green.log 146件成功。
- 実PNGとalpha: actual-png-green.log 2件成功、upload549x8192・表示1280x19067。
- native復元: native-restored-startup.json success=true、mesh追加後もheartbeat frame240、15.52秒。元設定をtmpに複製し実デバイスで実行、原設定は変更しない。
- filter RED: filter-red.log 1件失敗。GREEN: filter-green.log Explorer20件成功、filter-clippy.log lint成功。一般regex不一致は未再現。
- 独立画像レビュー: P0/P1/P2指摘なし、native再起動の不足は上記実測で補完。
- 依存: root/screenshotのdry-runでunicode-properties0.1.5・yazi0.2.2を最新互換候補として採用。libc/KUC/generic-arrayは上流exact制約、pestはpsm競合、egui0.37/parking_lot0.13は破壊的移行となるため維持。tmp/v0-22-46/dependency-audit.md と compatible-updates.log に実測。
- source build: build.log exit0。リリース版version同期後の通常push/CIは未回収。
- wildcard: wildcard-red.logでroot.png不表示を再現、wildcard-green.logで7件成功。一般regex故障という初期仮説を、実入力のwildcard期待との不一致へ更新した。
- 一時リカバリー: recovery-state.jsonで36tabs維持・workspace.json履歴不変、installed-recovery-startup.json success=true、frame143、既存v0.22.45の起動を確認。公開v0.22.46への差替と元PNG復元は5.5で実施する。

- wildcard整理後: wildcard-split-regression-final.logでExplorer24件成功、filter_tests108行/wildcard_tests124行。final-build.logでv0.22.46 source build成功。
- release独立reviewのP2を採用: 3.2の事前回帰/buildをfuture許可から外し、未完ならphase gateが拒否するRED→GREENを確認（phase-review-red.log/phase-review-green.log 3件成功）。pushとCIの結果は5.2/5.4に明示分離しゲートを維持。

- 画像記録: image-harness-green.logでrunner58件成功、image-harness-run.logで実画像ready（0.238秒/0.215秒）を待って両経路撮影。tmp/v0-22-46/image-harness-output/ の2PNGを確認。検査contextの上限2048によりtexture137x2048、画質評価ではなく安全な描画完了の記録として扱う。実nativeの8192上限は別回帰で検証。

- releaseゲート最終: postpublication-final.log 7件成功。独立review再確認でP2解消、残P0/P1/P2なし。3.2未完を拒否するphase regressionを維持。

- preflight初回はsafe ZIP9.2.0→local katana-core0.22.46の未登録で停止。core/srcは公開v0.22.45から完全同一（git diff exit0）。既存safe edgeへexact1組だけ追加し、未知v0.22.47/別manifest/git source/orphan拒否を維持。zip-edge-red.log 1error→zip-edge-green.log 27件成功。独立High reviewでP0/P1/P2なし。just supply-chainは実3manifestのadvisories/bans/licenses/sourcesを実行してexit0。旧ZIP経路・advisory ID・2026-11-07期限は変更しない。

## Draft PRレビュー

- [/] Codex P2: Windowsの相対pathはbackslashのため、icon*.png / assets/*.pngの照合を両区切りで成立させる。PR407 thread PRRT_kwDORm09y86yYr30、comment4434972135。
- 通常push exit0、8コミットVerified、Draft PR407を作成し@codex review実行・取得済み。Draft由来SKIPPEDはCI成功扱いしない。

- Codex Windows P2と独立reviewのUnix filename互換P2を修正。MAIN_SEPARATORをprivate boundaryとして、Unixは/だけ・Windowsは両sepでglobを生成。通常literal/regexは無変更。portable両OSの実Regex + Unix cache + cfgWindows nativePathJoin/cacheを回帰。windows-glob-unix-red.logでRED、windows-glob-unix-green.log Explorer28成功。独立High再reviewで残P0/P1/P2なし。修正pushとthread reply/resolveは次段で回収する。

- 修正a1edf428の通常pushもexit0。署名Verifiedとremote一致を確認。PR407 comment4434972135へreply4435076494を直接投稿しthread PRRT_kwDORm09y86yYr30をresolve、取得した全threadは未解決0・P0/P1なし。Draftを維持して最新コードのCodex再レビューも依頼済み（comment6556181760）。Ready前に結果を再取得する。

- [/] 最新Codex P2: 単語一致+basename wildcardの境界をprefix後に適用し、assets (copy)/icon.png / assets-/icon.pngの表示漏れを修正する（PR407 thread PRRT_kwDORm09y86yZJrc / comment4435168015）。追加レビュー取得後Draftへ戻した。旧e624のCI結果は別途回収し、修正後の検査成功と混同しない。

- 単語境界P2: word-boundary-red.logで実cacheのassets (copy)親directoryが消えるREDを確認。normalglob本体にだけ境界を置く修正後、word-boundary-green.logでExplorer29件/fmt/clippy成功。両OS separatorの回帰とliteral/regex既存境界を維持。独立High最終reviewは残P0/P1/P2なし。thread reply/resolveと修正後CIは後段で回収する。旧e624の全3OSlint・供給網・Readinessはsuccess、Draft復帰により実行中test/CodeQLが自動cancelledとなった（first-ready-ci-result.json）。

- 単語境界修正7a75c8fdも通常push exit0・remote署名Verified。PR407 comment4435168015へreply4435238987を直接投稿しthread PRRT_kwDORm09y86yZJrcをresolve、全2threadsの未解決0を再取得。Draftを維持しCLIで最新Codex再reviewを依頼（comment6556697185）。追加結果はReady前に回収する。

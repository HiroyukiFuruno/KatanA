# v0.22.47 KatanA完結修正の一括公開

## 要件（Verbatim Requirements）

> [PR405](https://github.com/HiroyukiFuruno/KatanA/pull/405) これを解決して下さい！
> あとissueが滞納してるよｗ
> katanaだけで完結して直せるものを先に直してリリースしてください。katanaだけで完結するものをグルーピングして、一回のリリースでまとめて公開すること。
> グループ名入力できない。以前は入力しなくてもエンターでグループ作れました。
> 直したあとにちゃんと確認しろや
> 作成したあとの挙動はサブウィンドを閉じるです。
> キャンセルもできないな。
> グループは以前は動作していたので明らかにデグレード。
> グループにタブを追加の候補が空欄になる。空欄は百歩譲って仕方ないが、選択した色が出ていたと思う。
> 次のリリースのタイミングでrootのREADME.mdにアプリの画像を含めて下さい。
> 名前あり、同色は認めるが、名前と色が全く同じはNGとする。名前違い同色は許容。
> 名前なしも一種の名前がないという名前という捉え方に出来るから、名前なし同色を許容しない。
> 名前は必須じゃねーよ。
> グループの名前はあとからも変えることが可能なのでヌケモレなきようにね。

### この要件から導くMUST

- PR405の更新とKatanA内で完結する残作業をv0.22.47の一度の公開へまとめる。
- グループの名前入力・空欄Enterを回復し、作成後はsubwindowを閉じる。Cancel/Escも回帰検証する。
- 名前は任意とし、同一workspaceの別groupとの「名前＋色」の完全一致だけを禁止する。作成・後からの名前変更・色変更に共通適用し、自分自身を除く。
- 同名異色・異名同色・空名異色を許可し、空名同色を拒否する。重複時は保存・tab移動を行わず編集を続けられる。
- root README.mdへ実アプリ画像を含める。日本語版も同じ実画像に同期する。
- 本番全体の入力経路と配布版で操作し、単体部品の成功だけで完了扱いしない。
- 既存DoD・品質条件を下げず、他repoの修正/公開依存は根拠付きで分離する。

## Definition of Ready

- [x] 公開v0.22.46、master b60c2d30、PR405の9refsと旧SHA契約失敗を確認した。
- [x] 既存Open12件をauditし、KatanA完結部分と真のupstream依存を分類した。今回のCI411/group412を別途記録した。
- [x] 公式v0.22.46の独立profile/唯一processでgroup入力・Enter・Cancel・EscのREDを取得した。

## Branch Rule

- Base/作業統合枝: `release/v0.22.47`。既存checkoutだけを使い新worktreeは禁止。
- PR405の差分を含むcheckpoint b824d286から継続。既存PR headへ通常fast-forward pushで集約し、最終title/bodyを全修正へ更新する。
- 親がbranch/commit/push/全体formatを管理する。担当者は分離した範囲だけを編集し、途中で独断commitしない。
- master直接編集、force/admin/no-verify、remote削除、他repo source編集は禁止。

## DoD・検証境界

- 原本/設定/group/tab/historyを保持し、独立fixture/profileで自動入力とnative操作を検証する。
- 既存build/test/lint/coverage/CodeQL/供給網/文書QAを維持し、未実行を成功と記載しない。
- Draft→review取得→指摘修正/reply/resolve→P0/P1ゼロ→Ready→必須CI→通常merge。
- v0.22.47は一度だけ公開。PR headはDependabotのため自動release/v起点と区別し、merge済masterからreuse_existing_version=trueの明示dispatchを使う。
- actual tag/5assets/checksums/manifest/source/run/attestationを照合し、公開後の操作/受入/capture、Issue・OpenSpec・localcleanupを完了する。remote refsは保持。
- 外部修正/公開が必要な#336、#386のKDV #70、#354のKDV #69は独立残件。本人操作/本人承認はClose/公開条件にしない。

## 1. グループ操作退行（#412、最優先）

- [x] 1.1 旧配布版native REDと既存単体テストの境界差を記録する（tmp/pr405-resolution/group-input-diagnosis.md）。
- [x] 1.2 full KatanaAppの実menu/pointer・別frame text/keyでREDを取得し直接原因を確定する。
- [x] 1.3 限定修正と、名前あり/空欄Enter→groupちょうど1増加→subwindow close継続をGREENにする。
- [x] 1.4 Cancelbutton/Esc、既存rename不変、他popup/modal・背景入力の保護を回帰検証する。初期修正の検証であり、追加要件の未保存draftに対するCancelは1.7で再検証する。
- [x] 1.5 candidate native操作で各経路を独立実行し、設定保存・再起動後も結果を確認する。
- [x] 1.6 人間SSの追加不備: タブ追加先の空名/名前ありgroup候補を選択色で識別できるよう、過去履歴と類似menu経路を確認し修正・実入力/描画回帰を追加する。
- [x] 1.7 名前任意の「名前＋色」重複検査を作成・名前変更・色変更・直接actionへ共通適用する。自己除外、空名matrix、同色異名、同名異色、legacyデータ保持を検証する。新規は未保存draftで名前/色を確定時に一括検査し、拒否時の状態不変と編集継続、Cancel/Esc時の破棄をfull-app経路で検証する。既存の有効な名前付き直接作成は維持する。
- [/] 追加先group候補の空名行に選択した色が見えず、識別できない指摘。
- [/] 名前は必須ではなく空名も一つの名前として扱い、作成後の編集も含め名前と色の組だけを一意にする指摘。

## 2. 更新helperと文書host（#409、#386、#385）

### Definition of Ready

基準source/入力と担当範囲が確定。group UI修正とは独立して実装可、共有dispatchやshellの変更は親が重複を調整する。

- [x] 2.1 両update導線の実spawn失敗でエラー表示・アプリ継続・temp lifecycleを検証して修正する。
- [x] 2.2 spawn成功時の既存更新/親終了を実process契約で維持する。
- [x] 2.3 loading/ready文書close・切替・reopen・別tab保持を実Office/HTML hostで検証し、KatanA側残差を修正する。
- [x] 2.4 PNG unchanged refresh/changed-file reload回帰を維持し、#385の既知修正・未再現残件・従来の調査停止範囲を正確に整理する。

## 3. CI・公開処理（#411/405、#408、#403、#343）

### Definition of Ready

sourceと既存実行入口を照合済み。group/updaterと書込範囲を分離し、CI例外で製品変更を隠さず通常v0.22.47へ統合する。

- [x] 3.1 9refsの公式upstream/tag/action.ymlを照合し、old SHA拒否/偽装通過のREDと完全固定のGREENを確認する。b824d286/実fixture29variants/Windows5+release27/既存resource契約成功。
- [x] 3.2 cleanup default/missing=false、explicit opt-inのみexact target/SHA/provenance/owner/hook条件を維持する回帰を通す。
- [x] 3.3 manual generated bumpのGit/API/process/metadata結合を実装し、別repo/run/attempt/source・改変/非bumpを拒否、same-source/PRclosed/v42契約を維持する。
- [x] 3.4 CI再利用の全入力同等性・信頼済成功run・拒否ケースを実装し、元の全品質条件を維持する。
- [x] 3.5 Windows cold/warm計測入口とmacOS normal/coverage inventory差・完全性・理由/digestの比較契約を実装する（実CI計測結果の回収は6.3）。
- [x] 3.6 v0.22.47の段階別task/artifact gateと公開後records scopeを登録し、CI前/公開前の循環を解消する。1〜5の事前検証は免除せず、push/review/CI/公開後にしか完了しない6.xだけを各段階のexact allowlistで扱い、誤branch/版/段階/追加未完項目を拒否する回帰を通す。

- [x] 3.7 通常pushで検出した旧CI契約の固定event検査を、同一SHAのmaster push/信頼済dispatch選択とverified consumerへ揃える。待機360/30・本体のsource/repo/master/actor/full8jobs/step検証を維持し、対象契約と17件の受入/拒否回帰を通す。

## 4. 実行環境・依存・受入準備（#372、#354、#339、#338）

### Definition of Ready

所有者・入力fixture・公開依存版を調査してから更新する。外部所有を推測しない。group/source/CIの確定を要する実測は依存順で実行する。

- [x] 4.1 listed image/venvのprovenanceを調査し、KatanA所有を証明できる現行recipe/venvのpip/CAを整備して実TLS正/負・既存検査を通す。所有不明・退役済みの旧対象は未検証のまま元Issueに残し、修復済みや外部所有と推測しない。
- [x] 4.2 direct/transitive/全lockの最新互換候補・MSRV・実APIを評価する。pending glam0.34.2/zip9.2.1も採否を正式に解決する。
- [x] 4.3 ZIP例外の根拠・期限2026-11-07・実利用と3manifestのstrict gateを再評価し、例外を勝手に延長しない。
- [x] 4.4 KDV/KRRの現行公開registry graphを確認し、必要な原本入力・Office/HTML受入と10 mixed cyclesの事前検査を完了する。
- [x] 4.5 canonical sample_diagramsの1280x2400 capture request・候補検査・exactsource/hash再現記録を整える。
- [x] 4.6 公開fixtureを使った実KatanaAppのtree/editor/preview画像を取得し、画像bytes・寸法・source/入力hashを確認してroot README.mdとREADME.ja.mdへ掲載する。
- [x] 4.7 README取得で判明した分割表示の背景による上部UI/tree/statusの上書き（#413）を残余content範囲に限定して修正する。実先行panel・縦横split・dark/lightのRED→GREENと、実App全画面の描画・canonical幾何の維持を確認する。

## 5. User Review Phase

### Definition of Ready

1〜4の実行可能な事前作業が完了し、未達・真の外部依存を明記した状態で、本人確認をblockerにせず証跡を提示する。

- [x] 5.1 screenshot request/操作記録を用意し、candidate group操作と対象回帰・文書受入の結果を提示する。
- [/] グループ名を入力できない退行、空欄Enter、作成後closeの指摘。
- [/] Cancel/Esc不能と、以前動いていた操作を本番経路で確認する指摘。
- [/] KatanAで完結するIssueを一回のreleaseへグルーピングする指摘。

## KatanA CLI Entry Point

OpenSpecは `./scripts/openspec`、UI記録は既存 `scripts/screenshot/run.sh --request <request> --output <ignored-output>`、build/test/lint/coverageは既存just入口を使用する。

## 6. Final Verification & Release Work

- [x] 6.1 self-review/独立High review、日英CHANGELOG・version/全lock同期、strict OpenSpec/KML検証。
- [ ] 6.2 通常pushを正式gateとして通し、PR405をDraftのまま最新内容へ更新してreviewを取得、指摘修正/reply/resolve、P0/P1ゼロを確認する。
- [ ] 6.3 Ready化して必須全checksと追加検査を回収し、Windows cold/warm同条件実測・macOS inventory同等性を実runで確認後、通常mergeする（admin/delete-branchなし）。
- [ ] 6.4 merge sourceのCI結果を回収後、v0.22.47を一度だけ明示公開する。tag・実bytes・署名来歴・identityの不一致は拒否する。
- [ ] 6.5 実公開版のgroup操作、起動・保存・再起動、packaged文書10-cycle受入、actual metadata receiptの正常/拒否検証を回収する。実経路とfixtureの主張を区別する。
- [ ] 6.6 final sourceのcanonical referenceを検証・採用し公開handoff記録を残す。specを同期・archive、担当範囲のIssueを証跡付きで整理し#388を同期する。
- [ ] 6.7 公開後記録を通常Draft review/CI/mergeで正式履歴へ統合し、branch-hygieneでstash0/masterclean/0ahead0behind/既存worktree確認・不要local枝整理・remote保持を完了する。

## 検証証跡

- Group native RED: tmp/pr405-resolution/group-input-diagnosis.md。公式v46独立profile、検索入力正常の後にgroup Text/Return/Cancel/Esc不動。
- Group full-App RED: group-rust-red-final.log / RTK tee 1793557241_cargo_test.log:164。最初のsizing pass由来disabledだけを原因としたred2は撤回し、実表示とmemory開閉の乖離を記録した。
- Group GREEN: group-rust-green-final.log 4件、group-rust-header.log 4件、group-rust-header-visible.log 1件、group-rust-clippy-final.log exit0。実表示treeの確認/Cancel/input消失を3frame確認。修正はgroup_header.rsのopen_memory(None)のみ、shellの広い例外は採用しない。独立High追加P0/P1/P2なし。candidate/公開版nativeは未完。
- 独立計画P1: v47のphase未登録で通常pushが循環するため3.6へ追加。3.5の実CI結果を6.3へ依存分離し、事前の比較契約は免除しない。

- Candidate build: group-candidate-build.log、既存just build exit0（4.54s）。native準備はgroup-native-green/identity.jsonにbinary/source/差分hashを固定。Macの画面ロックで自動解除失敗のため操作は未実施、1.5は未完のまま。画面unlockをuser_inputとして依頼し、他の実装/検証は続行する。本人の受入確認をClose条件に追加したものではない。

- Update helper macOS: update-409-review-core.log 2実process、update-409-review-ui.log 2実process、update-409-review-clippy.log full --lib --tests exit0。Windows成功fixtureの無関係なWait-Process失敗markerを修正済、Windows実行は6.3で未回収。

- 依存更新候補はroot/screenshotのglam0.34.2・ZIP9.2.1、paint11packagesを採用しversion/全lockを47へ同期。互換typecheck成功、paint2件成功、registry graph単一V8成功。parking_lot0.13は実libc解決競合、pest2.9.5は実psm解決競合で不採用、exact公開上流制約を保持。詳細tmp/pr405-resolution/dependency-assessment.md。
- #408/phase、#403manual source binding、#343限定gate/coldwarm観測の実装は未commit。独立HighでmanualCIの品質不足P1と比較失敗伝播/欠落tasks P2を発見し修正・対象回帰成功、再review中。generic test reuse/normalcoverage等価は未証明でskip禁止。
- native groupはnamed Enter/empty Enter/確認button/既存renameCancel/新規Cancelの実入力と実表示close/保存状態を確認中。Esc・正常再起動を含め全結果回収前は1.5未完。
- native QA事故: quit後のAX取得が専用QA appをCONFIGなしで再起動、通常settings/workspaceのmtime更新を観測。正確な直前baselineがないため事故由来差分は確定不可、推測復元は行わない。正確に専用QA pathのPIDのみ停止し、現在値snapshot/hashを保持。QAのみLSEnvironmentで再起動時も隔離CONFIG固定、以後通常profileはhash不変を確認。詳細tmp/pr405-resolution/group-native-green/audit-after-incident。

- Candidate native完了: tmp/pr405-resolution/group-native-green/native-summary.json/md。名前あり/空Enter・確認button・新規Cancel/Esc・既存renameCancel、実表示closeと保存state、正常exit0・再起動復元を確認。v46debug snapshotであり公開v47再検証は6.5に残す。人間が別不備のSS取得のため再起動を依頼し、PID34661で検証UIを前面表示して入力を停止した。SS未受領の間も独立検査は継続する。
- 原本7Office+HTMLのisolatedcopy/hashを照合し公式just test-document-fixtures exit0（host-original-fixtures.log、Office typed first-frame/PPTX slideshow/HTMLの3対象passed）。これはsource v47の結果でpackaged ten-cycleとは別。HTML inflight実worker1passed、Office inflightは初回15s idle=false失敗・同条件再試行13.10sで1passedのため不安定性を未解決として調査継続。

- 正式preflightと同じdependency-contract事前build後、scripts/release/test_*.py全789件/472.596秒 exit0（release-unittest-v47-final.log）。format exit0/53JSON変更なし、OpenSpec/日英CHANGELOG scopedKML exit0。ASTは新testのmagic numbers/process facade違反を検出し修正中、成功扱いしない。
- 個人設定保持について前記『以後hash不変』はその時点だけの観測で、native-summary収集時にはsettingsのactive_tab_idx/open_tabsが変化していた。current-audit/timelineを正とし、事故前厳密baseline無し、旧同一workspace36→現34tabs/0groupsの時期/因果は未確定。04:29→04:34は両hash不変。通常profileの完全無変更は主張しない。
- 公式fixture検査entrypointは毎回新規KATANA_CONFIG_DIRを開始前に固定するようscripts/ci/document-fixture-acceptance.shを修正し、host-original-fixtures-isolated.logで全3対象成功。既定workspace repoを読むtestの隔離を改善したが、過去の実profile writerを特定した修正とは主張しない。

- 独立再実行で更新成功processテストが失敗: update-409-final-source-ui.log / RTK1793562733 line175、markerファイルは存在するが内容空（0.42s）。core/UI共通の作成→書込完了の競合を特定し、期限15秒は据置で完全marker内容の到達を待つようfixtureを修正、最終結果を回収中。2.2は再び未完へ戻し、先行成功だけで完了維持しない。groupだけの関心別通常commitを進め、更新sourceは未stage。

- marker競合修正後: update-409-race-ui.log 2件/core.log成功失敗各1件、成功fixture各10回の実process反復全成功（update-409-race-repeat.log）。親の既存just入口でもupdate-409-final-race-just.log 2passed。期限15秒は不変。ローカル版更新Issue入力5件は403-japanese-commit-hooks.logで実hook通過/拒否を確認。
- Group修正は通常署名commit eee0f416a8c10ae38517b125b4df4a5627bd7ce5 (#412) に記録、未push。

- 所有調査の境界: 10/13〜29の限定session実tool証跡・shell履歴・KatanA全ref履歴を追跡。旧KCU image1件はexact buildID/source/recipeから外部KCU所有と判明、関連KCU #13はclosedだがexacttarget coverageは未確認。残3imageは現物不在・owner不明を維持し、KatanA外と推測しない。#372の元Close条件は満たしたと扱わずOpen残件を維持する。現在KatanA所有recipe/venvの整備とその実TLS結果とは分けて公開記録へ残す。
- コミット文面の言語規約と旧English固定ASTの矛盾を修正し、署名/固定版/Issue/pathspecを実行行へ結合、no-verify/comment/echo/複数commitの拒否を追加。target4/AST30/linter全targetclippyが成功（ast-ci-integrity-*.log）。検査無効化は行わない。

## 調査後に確定した今回の所有範囲

4.1は元Issue #372全体のCloseを意味しない。人間の『katanaだけで完結して直せるものを先に』を境界とし、現在KatanA-owned observer recipe/current evidence venvはpip26.2.2・実TLS5/5・unit4/4を検証。現物のない3旧imageは履歴調査でもowner未特定、KCU imageは実buildで別repo所有と判明したため、このpatchで修復済みとは算入しない。#372の未達Close条件を削除せずOpen追跡を継続する。所有不明を外部待ちと仮定したゲート免除でもない。

3.4/3.5は判定・拒否契約と実測入口を実装した範囲の完了。未知のdoctest/環境等価性は再実行を選択し、generic skip/reuseは認可しない。元Issue #343の汎用再利用・実測短縮・完全同等性まで達成したとは記載せず、実cold/warm/全OS CIは6.3、成立しない最適化はIssueに残す。全通常test/coverage/品質閾値を維持する。

新しい不備のSSは受領済み。色表示退行と、その後の名前＋色の重複要件を修復・検証するまで5.1と公開は未完。本人の動作承認を新たなDoDへ追加しない。

- 更新helperは通常commit 6c9d1312 (#409)、新規source/fixtureの実検査後に正式記録。文書host2.3はsource側2回帰と既存ready-close、原本3対象の実worker検査を確認した範囲で完了し、初回15s timeoutと後続成功12〜13sの不安定性をhost-office-diagnosis.mdへ保持。上流first-frame性能/worker固有cancelの完了には拡大しない。公開後packaged ten-cycleは6.5未完。

- 新SS受領: /var/folders/ql/4640yx8s22zg367pjjld7yc00000gn/T/codex-clipboard-92d4132c-01c5-40f3-9d6f-b3a8e83e396f.png。タブcontextmenuのグループ追加先submenuで無名groupが空行になり、色も見えない。過去表示の履歴を確認してから退行/既存欠落を判定し、同じv47へ修復する。ユーザーの既定名の追加は要求されていない。

- 色表示の履歴b0d16db8783eb9cf2c785521f57d7655289fce3fで色丸削除を確認。共通の色付きbuttonを両menuへ復旧し、空名/同名異色/短長名の描画、色・名前・右端からcorrect ID選択、tab所属済み除外、Explorer demo除外と従来の所属済み表示を実menuで検証。group-color-row-edge-tests.log 7passed、group-color-edge-ast.log 30passed、group-color-edge-clippy.log exit0。独立Highの右端/除外テスト不足P2は追補済み。actual v47 candidate/publicは6.5で未完。
- 追加の名前任意＋色pair要件をIssue412とspecに明文化。作成だけでなくrename/recolor/直接actionに共通適用し、未保存draftで重複拒否後の編集・Cancel/Escを検証中。root README実画像の公開fixture/requestも準備済み、最終source captureは4.6未完。

### 追加pair検査の独立レビュー指摘

- [/] P1: 有効な名前付き単一CreateTabGroup直接actionも、Manyと同様に即確定を維持し、重複時だけdraftに残す。実action正負を追加する。
- [/] P1: FinishInlineRenameはmutation拒否時にpopup/draft/inline状態を閉じない。受理または無変更時だけ閉じ、拒否後の修正を実actionで検証する。
- [/] P2: 空白だけの名前を比較上空名として扱う境界をspecにも追記。design decision7と一致させ、生値を勝手に変更しない。

- pair最終source固定後 group-pair-frozen-tests.log 32passed（隔離profile/full-app input/direct action/実JSON legacy保存再読込）、group-pair-frozen-ast.log 30passed、group-pair-frozen-clippy.log exit0、mutation入口audit保存。独立Highの2P1は修正source・回帰内容を再確認し追加P0/P1/P2なし、親も実物ログを回収。candidateと公開v47は5.1/6.5未完。

- 最新candidate実操作: group-native-v47/identity.json, launch-verified.json, launches-verified.jsonl, native-case-results.jsonl, native-summary.json。local debug v47/FE488 UI source、binary/worker/Cargo/lock hash固定。named Enter/empty Enter/Confirm close、空名同色拒否後編集、異名同色作成、後rename同名同色拒否→異色で成功、後recolor重複拒否→Cancel元色保持、自己no-op、新規Cancel/Esc破棄、既存rename Esc保持、通常Cmd+Qでmain不在→再起動4groups保持を実操作と保存JSON照合。終了コード0は未取得であり、owned-child exit receiptとは区別する。公開v47検証は6.5未完。
- native接続の『画面ロックか』は人間が解除済みと回答。CUA mouseButtonの短いr指定で右clickが成立し、長いright指定の不成立と区別した。追加システム設定変更や別UI手段は使わない。
- 最終pipeline sourceのactionlint v1.7.12 exit0、release test_*.py 791件/325.624s exit0（release-unittest-v47-with-host.log）。新host hyphen testsは別回収し、v47 6/observation7/version1/legacy acceptance34/publication9が全成功（host-parent-verification-*.log）。実140open/closeの完了には読み替えない。

- 4.4の事前検査: 原本Office7件+HTMLのprivate isolatedcopy/hash、source実workerのOffice typed first-frame/PPTX操作/HTMLの3対象（host-original-fixtures-isolated.log）とinflight/ready-close回帰を確認。registryは21:18 UTC時点でKDV0.5.12/KRR0.4.23のまま。10 mixed-cycleの実データは6.5で公開物へ結合するため、前段では7pair/140open-close・必要操作・実PID/epoch/seq/開始前closed/終了idle・性能閾値の記録導線と拒否契約を検査済み。host-acceptance-plan.mdに実行手順を固定。実cycle成功・実配布acceptanceとは主張しない。独立Highの版誤記/旧PPTX免除流用/時間過小計測3P1とcanonical事前作成P2は修正・回帰済み。

- #413は通常署名commit43fc8d04に記録。旧max_rectの実panel REDから残余範囲のGREEN、bottom/top/left+縦横+明暗4組（対象2tests）、AST30/Clippy成功。独立High追加P0/P1/P2なし。
- 正式capture source43fc8d04の前後clean HEADをsource_identityで確認。canonical-candidate-v47-fixedは1280x2400/sha256 6b5ece9d9d3548e4f0250d9126535e10daf90768029f77e9b2ed46c8c3d26527、source609の修正前PreviewOnly候補と同hash、geometry/clean interaction条件も成功。採用は6.6で公開後に行う。
- README実画像はassets/readme/workspace.pngへ無加工で複製し日英rootREADMEへ掲載。3200x2000/436273bytes/sha256 ac48c9cc563e5a6db50ec04045b0d849a7427759be482448b7244d5c20a17825。source/request/fixture/imageのhashをworkspace.capture.jsonへ記録、上部/tree/statusの実画素が非単色で全体を描画。actual KatanaApp in-processでありpackaged受入の代用にしない。旧黒覆い画像は採用しない。
- 最終self-reviewはcommit-self-review.mdと各独立High review。6.1はlocal/対象検査とsource整合を確認した範囲、通常push正式gate・Draft review回収以降は6.2〜6.7のまま未完。

- 1回目の通常push（normal-pr405-push.log）は送信前検査で停止、remote未更新。791tests/331.730s・版契約は成功後、旧shell契約が廃止済み--event push literalを要求してstep3で失敗。gate本体を弱めずselector/consumer契約へ追従し、version-bump-ci-contract-final.logとadmission17件が成功。通常pushを再実行して正式gateを完遂するまで6.2は未完。

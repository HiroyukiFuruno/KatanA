# TLS更新後の現graph検証と残件（2026-10-23）

## 判定範囲

検証対象は通常署名commit `ba0728010775242c6fa40d5ba9ad70566b134119`、tree `982fad4c4aa14d686a1546ce3036238d959af985`。既存 `release/v0.22.42` worktreeだけを使用し、検査中のtracked sourceを固定した。古い公開PRの成功は新graphの証明に流用していない。

両lockのTLS修正は rustls-webpki 0.103.16 / webpki-roots 1.0.10。root lock SHAは `927fac6b5c168bf578c0e787ea7ea17f0bd0c079a9cd13557a4c7134836c51dc`、撮影tool lockは `a0f0abdedb6751225121368cb633c0a0238e8b262d42497b3e185c50f9c5b5d5`。対象差分と公開依存の確認は [TLS修正の記録](tls-independent-patch-2026-10-23.ja.md) を参照。

## 実行結果

| 検証 | 実結果 | 証明しないもの |
| --- | --- | --- |
| macOS通常workspace | actual0、子processの二重計上を除き2170成功/0失敗/既存6ignored、doctest成功 | Windows/Linuxの実行成功、packaged受入 |
| vendor追加3gate・正式strict lint・fmt | すべてactual0 | 供給網全体・公開成功 |
| 正式coverage再実行 | 元4工程すべてactual0、meaningful gate成功、strict document surface100%/未実行行0 | 初回の断続的HTML timeoutの恒久解消 |
| release Python全件 | 現graphで564件/248.513秒/actual0、共有Gitconfig/source/両lock不変 | 旧HEADの成功流用、実Windows/packaged受入 |
| Linux実workspace | Docker Rust1.95.0/aarch64、コンテナexit0、子processの二重計上を除き2142成功/0失敗/既存6ignored、Python24+7成功 | macOS/Windowsとのテスト集合同一性 |
| Windows向けcross | Linux/aarch64上のcargo-xwin0.21.4、x86_64-pc-windows-msvc、workspaceとtestsのcheck actual0 | Windowsネイティブ実行、cold/warm速度、UI/GPU/packaged起動 |
| 撮影tool独立workspace | locked check actual0/6分03秒、配布用locked release build actual0、元release契約test54成功/0失敗/0ignored | 公開masterの修復完了、website素材再取得 |

normal commandは512.120秒、coverageの元4工程command計698.828秒。診断時間は別集計。全体job時間・完全単独CPU測定・短縮効果ではない。

撮影toolは元manifestの `cargo build --locked --release`（31分56秒）と、元CIの `cargo test --locked --release`（compile30.80秒、54test実行3.10秒）を回収。session46366/72b7db actual0、終了UTC `2026-10-23T07:52:20Z`。実runnerは `scripts/screenshot/target/release/katana-screenshot`、SHA256 `2208feac65df2731b8666e2cb284dcda67f152682265f12cd6c3fbebadf78953`。本人app・設定・Termsやwebsite素材取得は操作していない。

件数訂正: macOSの旧2169件はcore integrationを146と足し誤った値で、正しくは147/計2170。Linux raw全summary2144件には既に親に含まれるUI子2件があり、除くと2142。ignoredは6件で、旧awkのfield位置を誤った0件を訂正した。rawや実exitは変更していない。

Linuxはhostの観測親が中断後もコンテナが生存していたため、重複起動せず `docker wait` のコンテナ終了コード0と全ログを回収した。旧中断実行のEND/exit欠測は未完として保存している。Windows crossは元 `just check-windows` を変更せず実行し、既存compiled targetを使っている。cold実測とはしない。

## coverage初回失敗を消さない

初回の正式coverageはactual101。HTML dirty-target navigationで `startup_accepted=true` の後、元の2秒deadlineを2005msで超えた。テスト・deadline・skip・品質予算は変更していない。

同じinstrumented binaryのfocused実行は1成功/0.24秒、全UIは1315成功/既存2ignored/28.15秒。別の成功実行をsampleしたところ、KRRの初回text measureからsystem font DBの全探索を観測した。ただし失敗時そのもののstackではなく、根本原因を確定した証拠ではない。旧診断sampleの `/bin/sleep` は診断用テストの意図的な子であり、app deadlockの根拠にしない。

既存 [KRR #109への一次証跡](https://github.com/HiroyukiFuruno/katana-render-runtime/issues/109#issuecomment-6345015908) を送付済み。KatanA側に代替rendererや期限延長を追加せず、兄弟repoも編集していない。元条件・diagnostic OFFの正式再実行は全4工程成功したが、断続的失敗を修復済みとは扱わない。

## #343の評価と未完を分ける

現normal対coverage比較はcandidate共通2160、normalのみ10、coverageのみ0。coverage4工程の6組のpairwise intersectionはすべて0。candidate集合は実行回数ではなく、cfg/計測/fixture/doctestの実行同等性を立証しない。したがって `execution_equivalent=false` / `reuse_authorized=false` を保持し、通常検査もcoverageも省略しない。

Windows exact compiled-target cache・安全key・Ready後配線・hitでも実子検査の維持、protected CI run/job identity検査は実装済み。一方、CI artifactの内容・store/profile/input digestまで照合して重いテスト結果を再利用するadmission経路は未実装。診断manifestが存在するだけで実装済みとはしない。

元Issueは「再利用も評価する」「証明できない検査は維持する」という段階導入である。現在の証拠では検査省略を正当化できないため維持する。作者が書けるPASS JSONを信用する別基盤や、新たな本人承認待ちを作らない。Windows native cold/warm、導入前後の全体比較、現HEADのprotected CIは未完。

## 公開に依存する残件

### Open Issue別の現在位置

2026-10-23のライブ取得は15件。`implemented` はコードがあるという状態で、公開や実機受入の完了ではない。

| Issue | 実装状態 | 残る確認・依存 |
| --- | --- | --- |
| #353 撮影lock | 現release graphへ同期済み、locked配布build/契約54件成功 | 公開master反映、website側撮影 |
| #352 関数長lint | 実装済み | current required CI・正式統合・公開 |
| #351 URL非同期状態 | 実装済み | current required CI・実packaged受入・公開 |
| #350 TLS安全依存 | 実装済み、両lock更新 | 上流ZIP安全性を含む全供給網、current required CI・公開 |
| #349 PPTX開始可否 | 実装済み | current required CI・実packaged受入・公開 |
| #348 PNG取消 | 実装済み | current required CI・実packaged受入・公開 |
| #345 Office実worker受入 | 実装済み | 現最終graphの実packaged/全OS受入・公開 |
| #343 検証軽量化 | 部分実装・評価済み範囲あり | Windows native cold/warm、protected CI/全体比較。同等性未証明の検査は維持 |
| #340 HTML data-URI | 実装済み | current required CI・実packaged受入・公開 |
| #339 KDV採用 | registry採用済み、部分対応 | KDV #69安全公開版、採用後の全gate・packaged受入 |
| #338 canonical crop | KatanA producer/handoff済み、部分対応 | KDV ownerの独立採点とcanonical採用 |
| #336 KLE採用 | 未実装 | KLE公開後のregistry採用・ABI・実editor host E2E |
| #335 hooks/cleanup | sourceと実fixture済み、部分対応 | 安全公開OID成立後のproduction activation/cleanup receipt |
| #312 PDF theme | 実装済み | current required CI・実packaged受入・公開 |
| #258 Canvas | 実装済み | current required CI・実packaged受入・公開 |

- #335: Issue/range admission・dispatcher/rollback・publication identityの実装と実fixture回帰は存在する。production activation・live公開OIDに基づくcleanup receiptは公開後に回収する。未成立の公開OIDを捏造しない。
- #338: KatanA producer/manifest/hash/geometry/retained bundleの生成・検査は完了範囲。独立4軸scoreは `not_evaluated`、KDV ownerのcanonical採用は未完。現在のKatanA画像を正解へ再採用して欠落を隠さない。
- #353: 古い公開masterの撮影lock不整合。現release graphはregistry KDV0.5.12/KRR0.4.23へ同期済みで、release PRには撮影toolのlocked契約testが存在する。現branchの検証を公開master修復・website素材取得と混同しない。
- #339: KDV #69の安全な公開依存採用・再検証が残る。現在のroot/screenshot供給網は旧推移ZIP0.6.6/2.4.2/8.6.0のadvisoryでactual1、paint-metricsはactual0。TLS2件は消失したが全供給網PASSではない。
- #336: KLE公開版が未成立のため、registry採用・ABI・実editor host受入は別の上流待ち。
- 通常push→current review→個別reply/resolve→Ready→現必須CI、全OS packaged/全5asset/checksum/clean-machine受入、本人最終受入/Terms、merge/公開/Issue更新/アーカイブ/cleanupは未完。Draft重CIや通常hookを迂回しない。

## 生ログと保持

worktree直下の `tmp/` に保持する。これらは成果物の出所・実結果であり、作者が更新可能なmanifestだけをtrusted required checkへ昇格しない。

- `oct22-current-native-gates-normal.log`、`oct22-current-native-gates-coverage-before-html-retry.log`、`oct22-current-native-gates-coverage.log`
- `oct22-current-native-gates/inventory/normal.json`、同 `coverage/` の4工程、`comparison.json`
- `oct23-current-native-coverage-report.json`、`oct23-current-native-retained.sha256`（11件照合済み）
- `oct23-linux-container-recovered.log`、`oct23-linux-container-exit.txt`、`oct23-linux-worker-sha256.txt`
- `oct23-current-windows-cross.log`、`oct23-current-screenshot-check.log`、`oct23-current-screenshot-release.log`
- `oct23-current-release-python.log`
- `oct23-platform-results-retained.sha256`

専用Linux targetの内容回収は最後のmount root rmdirがEBUSYでactual101。後続read-only検査actual0で4KiB/空を確認した。cleanコマンド成功とは書き換えない。Macの専用native targetと撮影check targetは公式保護clean actual0で回収し、ソース・他target・他volumeは削除していない。

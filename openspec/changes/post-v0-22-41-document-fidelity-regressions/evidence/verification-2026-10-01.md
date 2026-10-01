# 2026-10-01 継続検証

## 対象と完了条件

`release/v0.22.42` の既存document-fidelity差分を正式履歴へ統合し、Draft PR、現HEADレビュー、必要な公開上流版、全品質・実配布受入、CI公開と後処理まで進める。上流HTML未解決は最終公開の依存条件であり、独立したKatanA作業を停止する理由にはしない。新worktree、stash保全、master編集、path/gitによる未公開sibling採用は行わない。

## この実行で確認済み

- `just check-full`: format成功、workspace strict Clippy成功、fixture統合8件成功。coverageはレビュー担当が誤って追加したWorkspace equalityテストのE0369で終了。既存WorkspaceはPartialEqを持たず、指摘自体がTreeEntry deriveの誤読だったため、その比較追加を撤去して再実行する。coverage・platform・supply-chainの成功は未確認。
- startup契約: exit 0、内部7件成功。identity契約: exit 0。binary architecture: 7件成功。render依存契約: 6件成功。HTML release契約: exit 0（旧v0.22.38 fixture契約であり現配布受入ではない）。
- stash 0件、master clean、作業HEADはorigin/masterとahead/behind 0/0。リリース差分は未commit。
- 現Cargo graphの互換lock更新、JS更新は0件。egui0.36.2 vendor同期は未実施。詳細はdependency-audit.md。

## 自己レビューの修正対象

- 不採用: WorkspaceのPartialEq回帰というレビュー指摘。mainが差分・元ソースを確認し、deriveはTreeEntryだけでありWorkspaceに既存PartialEqはないと確定。不要なAPI追加・比較テストは採用しない。revision独立性の既存回帰は保持する。
- fresh configではTerms modalから早期returnし、completed-frame heartbeatへ届かない。規約を自動承認するのではなく、modalの実描画完了後にもheartbeatを記録する。
- document workerは`open_session(source.clone())`で入力bytesを複製し、元Vecをsessionの生存期間保持する。`&mut source`からbytes所有権をsessionへ移し、metadataだけ残す修正と実PDFの回帰を追加。
  - focused Rust testは実PDFのopen/frame/close、bytes消費、URI/revision維持を検証して1件成功（0.56秒）。

## 再検証

Cargo workspace・Info.plistをv0.22.42へ同期し、workspace lockを通常の`cargo update --workspace`で再生成した。外部の互換依存更新はなく、ローカル4crateのversionだけ更新。`just fmt`成功後、変更後の`just check-full`を再実行中。fresh-config heartbeat回帰は、並列UIテストが結果を汚さないようchild processの環境へ隔離する。

- fresh-config heartbeatのchild-process回帰1件、test targetのstrict Clippy、fmt checkが成功。親プロセスの環境変数は変更しない。
- 変更後の全gateはformat・workspace strict Clippy・fixture統合8件（8.38秒）が成功し、coverage workerをビルド中。
- 独立した`just supply-chain`はexit 0。advisories/bans/licenses/sourcesの全4分類成功。推移的な重複crateの警告は残るが、検査規則は変更していない。
- Explorer/workspaceとHTML/画像/Markdown/math差分の限定レビューを完了。誤読によるWorkspace指摘以外のP0/P1は追加なし。最終全差分レビューを代替するものではない。
- 既存release-preflightはv0.22.42でexit 0だったが、version別globがactive post-v0-22-41 changeを拾わず、未完了の必須受入を検査していないことを確認。これは公開承認の根拠には使わない。明示的なversion/change bindingと、strict/artifact-pendingモードの未完了拒否を回帰テスト付きで修正する。
- v0.22.42の変更履歴を日英同期で準備した。記載時刻は準備時点であり、公開時に実リリース時刻へ同期する。未解決HTMLは修正済みと記載していない。
- 公開gateのbinding修正はmain再実行でも12回帰成功。必須IDが欠けた空/短縮台帳も拒否する。実台帳を`release-artifact-pending`で検査してexit 1となり、未完了のHTML・全品質・clean-machine受入を拒否することを確認。
- coverage単体はcore216、linter56、platform111、UI872、worker17件が成功（UI既存ignore2件、今回追加なし）。非UI統合、UI並列143件、UI直列18件も成功。coverage集計はDEBUG出力とheartbeat書込失敗ログの2行が未実行として拒否した。環境をchild processへ隔離した実経路テストを追加し、除外・閾値は変更しない。platformと最終coverage gateはまだ未完了。
- 長時間のsample Mermaid export統合13件は343.76秒で成功。実行中の3秒CPU sampleは`ImageExporter → KDV SurfacePainter.paint_diagram → paste_rgba_resized → image.resize`を示し、図画像のリサイズ/合成段階にいた。CPU約100%、footprint815.4MiB。coverage instrumented debug実行であり、releaseアプリの遅延・メモリ値へそのまま転用しない。生traceは`tmp/export-regression-live.sample.txt`。
- 診断2経路のchild-process回帰追加とフォント切替修正後、UI単体875件・main17件が成功（既存ignore2件）。元gateと同一の除外/構造行判定で意味のある未実行行0、document-surfaceの独立strict検査100%。生JSONを`tmp/coverage-2026-10-01.json`へ保存。全`check-full`一括成功とは扱わず、変更後通常Clippy・platformの終端を継続確認する。
- フォントメモリ修正の追加レビューでcustom→standard切替を復元する必要が判明。実OSフォントと実egui ContextのFontDefinitionsを比較し、初回標準の不変、custom primary、標準定義への完全復元を検証した。フラグだけを期待値とするテストにはしない。
- Linux全workspaceテスト成功、Windowsテストコードを含むcross-check成功。追加フォントテストの最終テストハーネス修正後にもWindows成功、Linux focused回帰は実行中。clean-machine配布起動の代替にはしない。
- release guard4ファイルを通常pre-commit経由で正式commit `783a1383`へ統合した。preflight wiringや残文書修正の統合はまだ未完了。crates.io完全一致の9回帰、version/change bindingの12回帰、実locked metadata検査が成功。

## Excelフィルタ接続と追加検証

公開KDV 0.5.7にはAutoFilter metadata、Candidates、ApplyValues、Clearとvisibility eventが存在するため、上流待ち扱いを解除してKatanAの薄い転送とヘッダー操作を実装中。全targetのcargo check、通常workspace strict Clippyは成功。状態回帰と実worker XLSX回帰、実クリック検証を続行する。候補数上限による打切りは明示してApplyを無効化し、空白は空文字のtyped値として送り、黙って候補を欠落させない。

Linux native workerを通常ビルドして実fixtureを開く回帰が成功（0.04秒）。CandidatesにNorthが含まれ、ApplyValuesでvisible_row_count=4、Clearで7を実session/worker channelから受信した。Mock・固定待機・見かけ上のUI状態だけの検証ではない。ヘッダーボタンからの実入力、全gateと配布実行は引き続き未完了。再確認時の最新KRR公開版は0.4.21（2026-09-28公開）であり、0.4.22は採用していない。

さらに実egui入力でheader→Candidates、menu→Apply/Clear、truncated時のApply無効・Clear有効、Select all後の適用を検証し、フィルタ関連11件成功。初回テストは、既定CloseOnClickで無効Applyや候補操作でもpopupが閉じる不具合を検出した。egui公開APIを確認し、フィルタ専用のCloseOnClickOutside wrapperへ修正後、同じ320x240 viewport・ボタンbounds検査・条件付きrepaint待機（Harness::run）で成功。固定wait・viewport拡大・無効化で回避していない。

最新変更でLinux UI全体871件成功、既存ignore2件（追加なし）、Windowsのtestコードを含むcross-check成功。AST23件・通常workspace strict Clippy・supply-chain全4分類も成功。旧native Mac coverageは診断2行まで解消済みだが、新フィルタ分を含む全coverageの再実行は未完了。配布受入・canonical scoreも未完了のまま維持する。

追加レビューで、既存NonBlank条件を開くと全候補が未選択となり、Apply時に空の値集合へ変わる問題を検出した。条件metadataと候補eventを通す回帰は修正前に `[] != [North, South]` で失敗し、非空候補を選択集合へ投影する最小修正後にフィルタ12件成功。KDVの条件評価自体をKatanAで複製した修正ではない。ヘッダーボタンと親グリッドのクリック競合は実RawInput回帰で再現せず、CandidatesのみでSelectAtなしを検証した。推測の商用コード変更は行っていない。

その後のLinux全workspaceテストはexit0。長時間export回帰14件350.23秒、UI並列統合141件（既存ignore2件）、直列統合18件が成功。Windows test-inclusive cross-check、native全target strict Clippy、AST23件、format/diff、MathJax typecheck/buildも成功。generated JS SHA-256は従来通り`8352ceae524e4b592c8dd69f30caac0b64c15e72cec6e4718154c144a0faff99`。採用旧stashのnavigation回帰を含むnative修正と公開依存を通常hooks経由のcommit `a296e49e`へ正式統合した。新フィルタを含むcoverage・配布受入・canonical scoreは未完了。

入力6原本Officeのhash/format、HTML原本hash、source-tree fingerprint、公開lock graph、配布main/sidecar identityとclean-machine heartbeatを照合する受入検査を追加。checker18件・release gate13件、計31件の回帰が成功。これは検査器の回帰であり、未実行の配布受入manifestは作成していない。

Linux実回帰はフォント探索が入れ子ディレクトリを辿っていない不具合を検出した。ディレクトリsymlink循環を避ける再帰探索へ修正し、実filesystem回帰2件とcustom→standardの実OSフォント回帰1件が成功。Windows cross-checkも成功。通常commitは一度、並行実装中のフィルタに対する既存AST制約違反で拒否された。lint規則を緩めず修正してAST23件を成功させ、フォント探索修正を`1dd570a5`へ正式統合した。画像タグ修正はparser33件・preview42件の再実行成功後、通常hooksを通して`ae5b0baa`へ統合した。Explorer回帰29件とフィルタ接続後のworkspace strict Clippyも成功。

## ハーネスのproc-macro障害（詳細）

現SDK27/linker環境でtiny proc-macroを生成し、dyld_infoがstrip=debuginfo/symbolsだけをLINKEDIT不整合として拒否した。strip=noneと最適化3の組合せは成功。壊れた既存dylibはcodesign検証が成功しており、再署名で解決する障害ではない。

[Rust upstream #157750](https://github.com/rust-lang/rust/issues/157750)のstrip時Mach-O alignment不具合と一致する。`.cargo/config.toml`のrelease build-overrideだけstrip=noneにし、本体opt-level/coverage/受入時間の条件は維持する。実際のscreenshot Cargo graphでserde/serde_deriveのrelease check、full release runner buildが成功した。

v0.22.42 runner・release Office workerで原本HTMLの1280x900/#s15 host要求を実行。公開KDV 0.5.7/KRR 0.4.21では`open_file`段階に留まり、60秒で初期frame/typed errorなし、最大CPU88.1%、終了は外部TERM。proc-macro障害は解消したがHTML受入は失敗。生ログは`/var/folders/ql/4640yx8s22zg367pjjld7yc00000gn/T/katana-html-acceptance.sndnIz/{runner.log,evidence.txt}`。このrunnerはin-processであり配布main executableの受入とは扱わない。

## 未公開上流probeとの比較

既存KRR担当のJSONをmainが原本hashと照合した。原本53818bytes、SHA-256 `c02d2d7a2420e4e15e3d98a044a310c67bc75fa858c95c4867b9c3f5d7aca012`。未公開v0.4.22候補のnative runtime probeは初期frame34929ms、close0.73ms、全体35.93秒、exit0。JavaScript69.5msに対してlayout_svg32.23秒、rasterize2.16秒で、主要遅延はlayout側。単体probe成功は公開版採用、KatanA host、sticky目次、配布受入の代替にしない。証跡はKRR `tmp/pr95-real-html-initial-close.json`。

`run.sh`の共有targetも分離し、各Cargo rootをlocked buildとする。これは独立した生成物管理修理であり、既に分離済みのhost scriptでも発生していたLINKEDIT障害の根因とは扱わない。

## macOS release mainの空workspace診断

最新のregistry lockで`cargo build --locked --release -j 2 -p katana-ui --bin KatanA`成功（51.96秒）。既存startup smokeをこのMacのrelease mainへ実行してexit0。fresh-config/Terms表示中もUI heartbeatが継続、main PID28243の実行path/hashを前後照合、peak RSS238752KiB（約233MiB）、UI font27093388bytes（約26MiB）、Office worker0。実行binary SHA-256は`8d28693e5990fa3d14c44df36792d641725d1cc27ae6ba35075a520dfb1bd184`。

これは開発Mac上のnative release main診断であり、配布bundle、他CPU/OS、clean-machineの受入ではない。smoke終了は既存supervisorのTERMであり、通常終了/全resource-releaseの証拠として流用しない。1.6/6.2および必須配布manifestは未完了のまま維持する。

## 実mainのOffice初回読込と非同期化

commit `a296e49e`のrelease mainを既存ユーザー設定で起動し、実操作のDEBUGログを確認した。初回source intakeはPPTX 5,845,262bytes/13,083ms、XLSX 27,857bytes/13,073ms、PPTX 40,852,621bytes/13,057ms。XLSXはsession open32ms、最初のframe40msであり、KRR描画だけを原因にしない。同じファイルの再読は1〜6ms。独立read-only計測でも別の約670KB XLSXの初回読み出し13.08秒、canonicalize相当0.00秒、SHA-256相当0.01秒、直後の3回の再読0.00秒だった。OSのcold read要因は未確定で、quarantine/provenance属性だけを根因と断定しない。

KatanAのUIスレッドからcanonicalize/読込/検証/hashを背景intakeへ移す修正を実装。UIは非blocking pollで結果を取得し、既存source revision比較を維持する。pending同一pathは読込を重複spawnせず更新要求をまとめ、変更通知・強制更新を保持する。HTML/Markdown移行時はpending receiverを破棄し古い結果を適用しない。既存のDEBUG=true限定helperへcanonical/read/validate段階とopen/read段階を追加した。

実FIFOを使い読込を完了させない間にUI frameを実行する回帰、取消済みchannelの実結果が別画面へ適用されない回帰、同一pendingの更新併合、実PDFの未変更/内容変更/強制更新のsession generation回帰4件がLinuxで成功（0.08秒）。native全target strict Clippyは成功。追加回帰と全UI再検証を続行し、実mainの修正版再受入と配布受入は未完了のまま維持する。稼働中の旧release mainは勝手に終了・置換していない。

## ハーネスと配布検査の追加レビュー

背景intake修正を`437f6c81`へ通常hooks経由で統合。配布受入checkerはmainとは別の実sidecar PID、観測されたcanonical pathも必須化し`11977d02`へ統合した。checker20件、release guard13件が成功。実Office sidecarを起動する配布受入manifestを捏造・作成してはいない。

現public registry graphでrelease screenshot runnerを再ビルドし、38単体テスト、release全target strict Clippy、paint-metrics2件が成功。rootのformat style editionを明示し、別Cargo rootでも同一規約の通常fmt checkを使えるようにした。

実CLIで文書未選択exportを再現し、修正前はPNG未生成なのにexit0でskipした。文書必須の明示errorへ修正後、同じ実CLIのnegative contractが成功。candidate生成経路は新規absolute出力rootを要求し、`assets/reference`・既存出力への書込を拒否する。不正入力の実script contractは成功し、参照画像・95点基準は変更していない。

現public graphで実canonical interaction contractが成功。controls-onを実描画から拒否し、Light typography/Dark diagrams両方で同一frameのcontrols-off、scroll0、logical1187x2225、physical crop(88,268,2374,4450)を確認した。diagrams frame1223、markdown section17、全overlay counter0。runner SHA-256 `b9fb3031a59eb356a04b1c476eb8bd6166c9ca8fd1cedd697f5f5337a58be2a7`、diagrams full PNG `f77baa764224dcc653ed0288263dab16c292d8a353933d4278e6343c8ed67bf1`、typography full PNG `394f828d1a89f89d0f15f6b4a62a6d704f4b275b9222f0e2b1225d2930ccfa3d`。生資料は`tmp/canonical-interaction-public-2026-10-01/`。これはin-process candidate evidenceでありcanonical score、baseline採用、配布main受入を完了扱いにしない。

同runnerと実release Office workerで下部sheetタブの実input/geometry回帰が成功（0→1→0、初期frame1.036秒）。mixed HTML/XLSX開閉はwarm baseline167616KiBから10回後168048KiB（+432KiB）、UI frame413→485、preview/surface/worker/frame/texture/cache全0。in-process検証であり実配布main・ユーザー原本HTML受入ではない。追加レビューでcold baselineのcloseが固定waitだけだったため、条件付きidle確認への修理を継続する。

追加レビューのclose条件修理後もmixed cycle実行が成功。warm167984KiBから169168KiB（+1184KiB）、UI113→183、全resource0。曖昧なviewport併記は旧CLIがexit0で受理したが、修正版実CLIはrequest load段階でexit1。新回帰を含む39件、release全target strict Clippyが成功。開閉の固定waitや入力寸法の無言優先を検証成功の根拠に残さない。

最新runnerと公開KDV0.5.7の実release workerでlegal data-descriptor DOCXを開き、Page1/2の初期frame2.026秒、close→idle、exit0を確認。生candidateは`tmp/docx-descriptor-async-2026-10-01/`。従来のlocal-header size failureはこの経路で再現せず、最終配布main受入は別途残す。

配布CIのcwdを展開先へ変更し、checkout内resourceへの偶然の依存を排除。別レビューでpartial targetのskipped jobが公開を許可するP1を発見し、全platform job成功を必須化。新contract assertionは修正前に失敗し修正後成功（binary architecture7件を含む）。asset collectionは5成果物を非emptyで必須化し、欠損/空/未知/衝突/symlink/hidden-entry拒否と実checksum照合が成功。実Actions/clean-machine検証は未完了。

## 正式統合後の全coverage再検証

HEAD `7e43924d` の全coverageでworkspaceテスト、UI parallel141件（既存ignore2件）、serial18件は成功したが、strict document surface gateは`painter_grid.rs` 179/182行（98.3516%）で失敗した。ログは`tmp/coverage-gate-20261001.log`、集計は`tmp/coverage-gate-20261001-current.json`。全coverage成功とは報告しない。

未実行のinlineクリックテスト失敗分岐を、正確なCandidatesコマンド1件とResizeのみのグリッドコマンドとの等価比較へ変更した。商用コード、100%閾値、除外設定は変更していない。format/diff check、focused実テスト1件、katana-ui lib strict Clippyが成功。独立レビューでP0/P1指摘なし、クリック不発・余分なコマンド・SelectAt混入の拒否を確認した。

変更後の全coverage gateは成功。UI parallel141件（既存ignore2件）、serial18件を含む全工程と集計を完了し、strict document surfaceは100%、`painter_grid.rs`は184/184行。ログは`tmp/coverage-gate-20261001-rerun.log`、JSONは`tmp/coverage-gate-20261001-rerun.json`。依存更新前のこのsource graphの結果であり、次の依存移行後の再検証・配布受入・全combined gateを完了扱いにはしない。

## egui移行後の実アプリ起動と品質（2026-10-01）

egui0.36.2移行を通常hookを通したcommit`886907a1`へ正式統合した。公開KDV0.5.7/KRR0.4.21/KUC0.3.17 graphのfull locked metadata、native release全target strict Clippy、paint-metrics2件、供給網4カテゴリ、全coverage gateが成功。document surface100%、painter_grid184/184。移行後の全coverageログ/JSONは`tmp/coverage-egui-0362-20261001.{log,json}`。実input/capture、platform/combined、公開は未完了。

新release main SHA-256`46e618fe768abc29d65c1fd5e1f8bd565f363e905c34d68a8aa742b8a6b3f3aa`を独立した設定で起動し、PID64901、継続UI heartbeat、peak RSS253392KiB（247.45MiB）、owned fonts27093388bytes、Office worker0を確認した。原始ログ/heartbeat/configは`tmp/trash/2026-10-01-113653-startup.YMB8Av/`へ保持。これはlocal native実mainの空workspace smokeであり、全OS clean-machineやOffice入力を含む配布受入ではない。旧PID50338は終了させていない。

起動テストの独立レビューでLinux wrapperのみ終了させるcleanupをP1として検出し、`f5eaadb9`で修正した。shared helperを実bash/sleep子孫で実行し、既知main/初期frame前の探索の両経路でowned main/wrapper消滅、PID clear、無関係プロセス生存を条件pollで確認。実early-exit負例と全startup contractも成功した。修正後の実main smokeはPID94562、peak RSS234976KiB、同じfont bytes、worker0で成功し、終了後PID94562は消滅、旧PID50338は保持。

移行後runner SHA-256`c7a527c638db70fceff2daea1081afeb456e73ce6189ad925e0f2e0848e917cb`でcanonical interaction contractとExcelタブの実0→1→0が成功。生資料は`tmp/canonical-egui-0362-20261001/`と`tmp/xlsx-sheet-tab-input-egui-0362-20261001/`。両full PNGは移行前のhashと完全一致（typography`394f828d1a89f89d0f15f6b4a62a6d704f4b275b9222f0e2b1225d2930ccfa3d`、diagrams`f77baa764224dcc653ed0288263dab16c292d8a353933d4278e6343c8ed67bf1`）。same-frame geometry/counterも保持する。独立source-renderer95点採用や配布main入力の完了ではない。

## ビルド容量の記録

通常commit hookのdevチェックと別runnerの再ビルドでhost空きが857MiBまで減少した。commit hookは成功したがrunnerは1GiB閾値に従い中断し、終了済みの誤った別target` scripts/screenshot/target`のみCargo cleanで6443ファイル/2.8GiB解放。既存`target/screenshot-harness`を明示して再開する。実source/証跡/使用中アプリ/他repoは削除していない。

空き容量が8.4GiBへ減ったため、使用中のroot/debug・coverage・新runner targetは残し、稼働していない旧`scripts/screenshot/target`だけをCargoのtarget-dir指定cleanで解放した。再生成可能な生成物38147ファイル、20.8GiBが対象。ソース、stash bundle、worktreeは削除していない。

その後空き5.2GiBとなったため、終了済みの失敗coverage runが残した`target/llvm-cov-target`だけをCargo cleanで解放（24345ファイル、15.6GiB）。新runnerとfocused testが使用中のroot/debugは保持。次のcoverage runは同じ品質条件のまま再生成する。

さらに空き4.3GiBとなった際、root/debugのfocused testとpreflightが終了済みであることを確認し、通常dev profileのKatanA workspace 4packageの生成物だけCargo cleanで解放（25483ファイル、21.4GiB）。別targetで進行中のcoverageとrunnerは削除していない。

後に空き135MiBとなり、稼働cargo/rustcがない通常root/debugをCargo dev-profile cleanで解放（50040ファイル、32.8GiB）。その後、coverage全テスト/修正再テストと両集計を完了してJSONを保存し、非稼働coverage dev生成物もCargo cleanで解放した。source/release runner/Office worker、profrawと保存JSONは対象にせず、通常Clippyの再検証へ切り替えた。

Linux UI全体検証後、実行中containerがないことを確認し、このrepoのLinux target volume内のcore/linter dev生成物のみCargo cleanで解放（285ファイル、26.0GiB）。Docker VM内部の再生成可能な領域であり、host APFS空き容量は約5.4GiBのまま増えていない。別repo、source、worktree、証跡、VM全体は削除していない。

native `just check`再実行はworkspace testの依存ビルド中にhost空き119MiBとなり、OS error28で失敗した。終了したcargo/rustcを確認して、失敗したroot dev生成物のみCargo clean（21760ファイル、8.5GiB）し、host空き8.4GiBへ復帰した。Linux側で同じ最新sourceの全workspaceテストを実行して成功し、native Clippy/ASTも再実行成功。nativeの一括gateが成功したとは報告しない。

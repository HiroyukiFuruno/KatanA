# Nextest比較候補の独立実装（2026-10-20）

## 対象と非対象

Issue #343の重複削減を進めるため、KRRの正式`root-test-and-coverage`経路（参照source0884d431）を読み、macOS限定の手動`just nextest-shadow`を実装した。従来の通常test・coverage4工程・CI・品質閾値は変更していない。

これは実行記録を採る非本番候補であり、`execution_equivalent=false`、`reuse_authorized=false`、`report_checked=false`を固定する。通常test、doctest、非instrumented fixture、全OS検査は引き続き必要。製品全体のshadow実行、coverage report、時間短縮、実行同等性、trusted reuse、公開の成功は未証明。

## 実装契約

- Nextest0.9.148／cargo-llvm-cov0.9.1を実バージョンで確認してから実行する。
- 既存coverageと同じ4選択・default features・cfg coverageを使い、単一の専用targetを共有する。全候補testを同一groupの最大1processへ制限するため、速度改善の主張には使わない。
- 出力は物理repoの新規tmp配下だけ。source開始終了・両lock・producer/config digest・コマンド・実終了コード・時間・phase別Nextest store.zipのdigestを保存する。store.zipの内容を品質合格とは扱わない。
- 記録収集はentry数4096、store数16、深さ8、総64MiBを上限とし、alias・非regular file・読み取り中の変化を拒否する。
- 非instrumented Office workerはcoverage target外へ排他的にコピーし、前後digest一致とreadonly executableを確認する。後続coverageによる同名worker再linkと分離する。
- 記録欠落でもphaseのraw child exitと理由を保持し、成功childに記録がない場合はproducerを2で失敗させる。非0 childは元exitを保持して後続工程へ進まない。

## 検証

- `just test-nextest-shadow-contract`: 28件PASS、1.059秒、actual exit0。worker固定の未実装はImportError actual1でREDを確認してから修復した。実子processの失敗7・signal・記録書込失敗、実fileのalias・FIFO・entry超過・変更を含む。
- 新Python9ファイルは全200行以内、全関数30行以内。Justの手動入口解析、diff check成功。独立最終レビューのP0/P1/必須P2は0件。
- 依存0の実Rust2member workspaceを`tmp/nextest-shadow-probe/2026-10-20-worker-snapshot`へ作成し、実Nextest／llvm-covで4工程・4store、全stage exit0、producer exit0、source stable=trueを確認した。これは製品renderer/Office受入の代替ではない。
- worker snapshot SHAは`168e8a1cc85b32da0594cb05073615b727024e639796f8f134d0e6048a188e04`。coverage後のtarget worker SHAは`52d7814bd6b728484fda9d7567e9f689da10b0d4d2991eaafbcaec2d8be873d3`へ変化したが、実行用snapshotは不変だった。
- 同fixtureへ意図的assert失敗を通常commitした別実行はactual exit100。stageはworker0→workspace-lib-bin100のみで、後続3工程のdirectoryなし・producer completed=falseを確認した。
- 共有Git config SHAは開始終了とも`bcf708315eb9ae84c706129108d49862ef47e24a0d13a615f0b3bd58e0fb3c29`で不変。master・上流repo・本人appは非操作。

初稿successはuser configのunknown profile warningを検出し、設定を補正したwarning-free実行を別に回収した。最新successの最初の補助検証はsnapshotとinstrumented targetのSHA一致を誤要求してexit1となり、役割が異なることに合わせた補正検証でexit0を確認した。いずれも隠蔽せず、runner成功と検証器の誤条件を分けている。

## 保持ログと次の作業

### 実行入力identityの追加修復

元#343のmanifest要件へ再照合し、source/argvだけでは実環境を復元できない欠落を修復した。ROOTと各phaseのchild開始前に、同じcwd/envで実CARGO wrapperの`-V`と指定RUSTCの`-vV`を取得する。phaseで変更するRUSTFLAGSに応じてwrapperのtoolchainが変わる可能性があるため、ROOTの結果はphaseへ流用しない。既存helperの親環境版取得は呼ばず、既存の環境allowlist定数・worker digest・OS/machineを使う。

tool取得失敗・空出力・明示RUSTC空文字/複数commandは失敗とし、stageを開始しない。空白を含む単一executable pathは有効。相対workerはchildのrepoへ束縛する。全環境dumpは行わず、phase固有のstate/profile/workerと検証済みNextest/llvm-covの実版文字列を保存する。

- 最終33件PASS1.656秒、実CARGO/RUSTC sentinel・ROOT/phase別版・失敗時stage未起動・encoded flags・相対workerを回帰化した。変更3ファイル125/134/195行、全関数30行以内。
- 相対workerの追加前の32件成功版で、別fresh依存0 fixture`tmp/nextest-shadow-probe/2026-10-20-input-identity`の実4phase/4store・全stage0・source stable=trueを回収。Cargo/Rustc1.97.1、Nextest0.9.148、llvm-cov0.9.1とphase実環境がreceiptへ保存された。相対path追加は別の最終33件で検証し、同一版の実Rust結果へ読み替えない。
- rawは`tmp/oct20-input-identity-success.log`と`tmp/oct20-input-identity-verify.log`。この段階も製品full shadow・同等性・削減・report合格ではない。

既存正式coverageのworker再linkも別tiny fixtureで切り分けた。通常`llvm-cov test`相当の4phaseは全0、worker SHAは各phase後不変だった。Nextest側の再link結果だけで既存正式gateを不備と断定せず、既存coverage.shは変更していない。rawは`tmp/oct20-regular-worker-coverage-phases.log`。

- `tmp/oct20-nextest-shadow-worker-success.log`
- `tmp/oct20-nextest-shadow-worker-success-verify-final.log`
- `tmp/oct20-nextest-shadow-worker-failure.log`
- `tmp/oct20-nextest-shadow-worker-failure-verify.log`
- 同fixtureの`tmp/success/shadow.json`と`tmp/failure/shadow.json`

次は共有build窓を調整してcleanな製品HEADで実shadowを採り、通常実行との対象・挙動差を評価する。KDVの稼働中の重検査へ製品buildを重ねていない。未証明の検査を削除せず、#343は部分対応を維持する。

# 通常pushの検査とキャッシュ整理の競合防止

## 失敗と確認できた範囲

`079b392c` の通常pushでは、native export検査13件が成功した後、次のMarkdown検査の実行ファイルが存在せず、起動前にOS error 2で失敗した。並行した既存の `just clean-all` は全体終了1だったが、再生成可能なキャッシュ20.3GiBの回収を報告し、空き23GiBを確認した。どの整理workerが当該ファイルを削除したかは特定できていない。

既存guardは、Cargoのprofile lockと実行中のファイルを確認していた。しかし、Cargoが個々のtest binaryを実行している間、今後使う別のtest binaryも含めて検査全体を保護する共有leaseはなかった。並行整理との競合はこの境界と整合するが、個別削除の因果関係を断定しない。

## 修復の契約

通常pushの `run_owned_check` から実行する `just check` 全体に、実効Cargo targetごとの共有leaseを保持する。cleanとsweepは同じtargetの排他leaseを非ブロッキングで取得し、検査中なら元の拒否・安全skipの動作へ戻る。整理中の通常pushも明示的に拒否する。

leaseは削除対象targetの外に置き、userごとの固定したprivate directoryとcanonical targetのhashを使う。環境変数による一時directoryの違いで同一targetのleaseが分裂しないようにし、Windowsではpathの大文字小文字差も正規化する。symlink、reparse point、非regular file、不正owner/mode、hardlink、不正な保存場所を拒否する。

これはCargo内部のprofile lockとは別の協調機構である。通常checkの起動前にCargoの内部lockを先取りせず、clean/sweepが従来のprofile lockを保持する条件と期間も維持する。検査のskip、失敗の成功化、閾値変更、hook迂回は追加しない。まだこの入口で包んでいない任意の直接Cargo実行まで、保護済みとは主張しない。

## 初期対象検証

実プロセスの回帰で、待機中のCargo testが後で使う別のintegration binaryを、実clean/sweepが削除しないことを確認した。idle後の整理再開、整理中のowner check拒否、失敗・割込み後の解放、異なるtargetの独立、環境差、不正lockも検査した。

`tmp/oct25-target-lease-validation.log` に実終了を保存した。新process回帰4件、owner検査8件、clean guard4件、sweep guard6件、clean process3件はすべて終了0。Pythonの構文検査も終了0。Windows用のbyte-range lock経路を実装したが、macOSの検証だけをWindows実測の成功へ置き換えない。

この初期成功だけでは中断時の子孫保護を証明できず、独立レビューで必須修復を検出した。原因メモは `tmp/oct25-cargo-test-clean-concurrency-diagnosis.md`、元の失敗は `tmp/oct25-079b392c-normal-push-recovered-cache.log`。

## 独立レビュー後の修復と最終検証

親だけが終了した場合にも検査側でleaseを保持するhelperを追加した。さらに、群leaderが終了しても生きている孫の検査を保護するため、自分が作ったPOSIX process groupの非zombie memberが終了するまでhelperを維持する。状態確認に失敗した場合は保護を継続し、終了を確認せずleaseを解放しない。POSIXの共有lockと異なり、Windowsではbyte-range lockにより同じtargetのmanaged checkも直列化する。

canonical化前の元path componentを検査し、symlinkや他userが書ける親を拒否する修復も行った。runtime根は環境変数に依存しないuser identityから解決する。Windowsでは公開Shell APIでprofile根を取得する。新回帰は既存の `test_cargo_*guard.py` の探索条件へ接続した。

独立reviewerは、実際の固定 `just check` から中間leaderと、TERM/INTを無視して待機する孫を起動した。ownerへのTERM/INT/KILLのすべてで孫の生存中は排他leaseを拒否し、barrier解放と終了後だけ再取得できることを確認した。leaderが既に終了した後のTERM転送も実確認した。修復担当の報告と独立判定を分け、最終の未対応P0/P1/P2は0件である。

主査は最終6ファイルのSHAを照合し、正式 `just test-cargo-sweep-contract` 18件とowner9件を終了0で回収した。正本は `tmp/oct25-target-lease-root-final-validation.log`、SHA照合は `tmp/oct25-target-lease-root-final-manifest.json`、独立判定と実出力は `tmp/oct25-target-lease-independent-final-review.md` と `tmp/oct25-target-lease-independent-final-validation.log`。最終helper SHAは `eaf5a8cc564b76025f3d623731ea6f3ec024090e6cd8be2a5698150d96bc68a9`。

通常署名commit `c6586270328b111a68513c0b578b30c8c21faa42` へ統合した。旧通常pushの失敗を成功へ読み替えず、新HEADのrequired checks・native Windows検証・公開完了は別に確認する。

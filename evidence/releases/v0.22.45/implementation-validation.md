# v0.22.45 実装・対象検証記録

公開・CI・原本のネイティブ性能計測とは分けた、ローカルの実装検証記録です。

## 更新準備の容量確認 — #399

- 更新処理の対象検査48件成功。UI更新処理の対象検査52件成功。最終モーダル表示の検査8件成功。各検査には重複範囲があり、合計件数として加算しません。
- 公開済みv0.22.44 macOS ZIPを使用。圧縮223368232bytes、展開585370525bytes、公開checksum一致。
- 旧処理は隔離320MiB APFSボリュームでダウンロード後のENOSPC28を再現。
- 新処理は展開を開始せず拒否。最終検査の再試行前必要容量1461218146bytes、利用可能容量333870120bytes。ダウンロード済みarchiveの自動削除を双方の値へ反映。十分な空き容量では同じ実archiveが検査成功。
- 通常利用のインストール先の置換・再起動は行っていません。隔離ボリュームは切断済みです。

## 画像配置と段階別性能ログ — #400 / #385

- 実Office workerを使うpreview_pane対象検査520件成功、失敗0、既存の無視指定1件。
- 実PNG描画元の配置3件、raster描画元の配置4件、表示された画像上のzoom・画像外まで継続するdragを確認。
- DEBUG=falseで性能ログなし。DEBUG=trueで実PNGの選択、watch登録、queue待ち、open/read/decode、結果回収、texture登録、mesh登録を確認。要求・revision・textureの同じidentityの重複を拒否し、同じtextureの4フレーム連続描画を確認。
- queue待ちはqueue_attemptからstartで算出。queuedは送信後の確認ログです。texture登録・mesh登録はGPU表示完了の証明ではありません。
- watcher通知で別revisionのtextureに置換される実ログを確認。この事実だけで正常・不具合・30秒遅延の原因を断定していません。
- 現行ソースのAST検査23件成功。通常の整形成功。

## 未回収の結果

- 空の専用キャッシュ・設定を使うarm64の実release本体とOffice workerのビルドは成功。独立したad-hoc署名の診断bundleがネイティブ起動し、v0.22.45利用規約画面まで確認。
- 本人が同意操作を行うという指示を維持し、同意設定の複製・書換え・自動承認は行っていません。本人用の設定・アプリも変更していません。
- 原本Office/HTML読込中にPNGを開くネイティブ性能計測は、同意画面通過後に継続します。30秒遅延は未解決として#385を維持します。
- 正式push検査・PRレビュー・必須CI・公開は、この記録で成功と扱いません。

## 検証用キャッシュ作成・掃除 — #401

- producerがCargoより先にtargetを作る場合だけ、空であることを確認して所有者タグを作成。未知データや不正タグ、symlinkを所有済みとして扱いません。
- 独立レビューで、タグ作成後からproducer終了までのclean競合と、親共有leaseからの排他昇格の問題を修正。POSIXでは実行と子孫終了まで共有leaseを保持し、初期化のみ別sidecarで直列化。
- 実rustcのコンパイル待ち・生成バイナリ実行待ち・共有leaseを持つ親の下での実行を含め、契約24件成功。稼働中のclean拒否と終了後のclean成功を確認しました。Windowsでのcleanup拒否を保つ契約も維持していますが、この記録でWindows実行済みとは扱いません。

## 依存関係の確認・更新

- `just outdated` の仮更新診断は、KDV固定版のlibc=0.2.190と診断候補0.2.191の競合で失敗しました。現行の固定graphのビルド失敗とは区別します。
- manifestの更新候補確認と、両lockに対する`cargo update --dry-run`を実施後、既存`just update`が成功。pulldown-cmark0.13.6および互換9package更新を採用し、KDV/KRRの固定版やoverride無しを維持。
- 新graphのMarkdown対象検査94件成功。起動中のネイティブ診断bundleは更新前baselineとしてlockhashを別途固定し、新graphの配布受入へ流用しません。

## 更新先の相対パス互換性 — #399

- 相対ファイル名の空の親パスを容量APIへ渡すとENOENTとなることを実測。共通helperで空の親を現在ディレクトリへ正規化しました。
- 実HTTPを使う独立した子プロセスで相対ダウンロードと相対archive/更新先の容量検査を回帰。修正前失敗、修正後48対象検査成功。AST23件成功、独立レビュー指摘なし。通常コミット9a694446。

## 正式pushの失敗と修復 — #354 / #343

- 最初の通常pushはexit1。preflightが安全なZIP9.1.3のkatana-core0.22.45というexactな依存組合せを未登録として拒否しました。新しい脆弱性の検出とは区別します。
- 当該組合せだけを追加し、未知のsource・版46・不正manifestの拒否を維持。27対象検査と実際の両lock供給網overlayが成功。既存の脆弱なZIP経路・承認・期限は変更していません。通常コミットd2680c04。
- pre-pushをpr-ready→strict→lightの順へ変更。実Lefthookで成功時の順序と、readiness失敗時にstrictが実行されないことを確認。必須検査を省略せず、不適合metadataを先に拒否します。通常コミットebeb31bc。新HEADの正式push結果は別途回収します。

## 将来版の配布成果物照合 — #362

- v44の実公開run選択は実APIで41002934022/1へ成功。一方、3OSのidentity artifact生成が旧42限定だったため、v44に存在しない証跡を完了扱いしていません。
- 将来版向けにversion/source/run/attemptを固定した3manifest・5asset照合と、公開後の読取専用検査を実装。従来42の承認・文書QA・必須job契約は維持。231対象検査成功、独立Highレビュー指摘なし。
- exactな45の2変更だけを対象に、未実行CI/公開タスクを未完了のまま各段階で検査する経路を追加。対象・既存契約82件成功、独立Highレビュー指摘なし。
- 公開後記録経路の新規10・既存release-flow27検査成功。独立HighレビューはP0/P1/P2なし。通常統合は継続中。実45のCI manifest・公開asset receiptの回収は未実行です。合成fixtureを実配布受入や95点評価へ流用しません。

## PNG選択より前のUI待ち時間 — #385

- HTML監視の同期生成・破棄、遅延HTML再読込、保留中の文書openが、画像action処理より前に実行される経路を確認。実buildのnotify backendはmacos_fseventであり、watcher破棄にはstop/joinが存在します。30秒の実測原因とはまだ断定していません。
- 通常の動作・順序・backendは変更せず、DEBUG限定の共通時計・開始/終了spanを追加。画面更新全体は遅いframeだけ記録し、実作業は対で記録。UIログは512イベントに制限し、対の2枠を先に予約。無効時にpath複製やtrace用のlockを増やしません。
- 実ファイルを使う子プロセスのON/OFF・対・上限と、実AppState/KatanaAppのHTML/MD/未選択の全遷移を含む計測検査3件、既存HTML監視8件成功。静的検査filter30件成功。最初の静的検査はテスト内の数値literalで失敗し、命名定数へ修正して再検査しています。ルールの無効化はありません。
- 独立Highレビューは追加の時計初期化・frame全体spanを含めP0/P1/P2なし。新しいnativeビルドでの原本計測は未回収です。既存baselineとはsource/lock/hashを分けます。

# Windows配布起動のフレーム間待機を切り分ける

## 実結果

PR367はsource `3bea23cd9a4d820873dfca57324f0259a65a00ee` のレビュー・必須CI成功後、通常merge `6862ca631e1326b3343cd7ae7bb7b4f53df96907` へ統合した。正式公開run [40272327344](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40272327344) / attempt1 のWindows job121410238971はビルド・配布物生成後、元の200ms観測 `last_frame=3; current_frame=3` で不合格。Linux配布起動とmacOSのIntel・Apple Silicon両起動は成功したが、run全体はfailure、Publishと公開後cleanupはskipped。終端後もtag/Releaseは双方404を実確認した。公開完了や[Issue364](https://github.com/HiroyukiFuruno/KatanA/issues/364)解決とは扱わない。

実診断artifact12629922719のstdoutでは、heartbeat1〜7とcumulative pass0〜6のbegin/endが全7対となり、解析エラー0件。UI処理は3.298〜10.921ms、heartbeat記録は0.243〜0.392ms、フレーム開始間隔は368.292/309.579/321.960/296.673/302.026/161.614ms。UI処理や記録自体に300ms停止を観測したわけではない。後続フレームが進んだことも、元の不合格を合格へ変更する根拠にはしない。

- 実jobログ: `tmp/oct26-6862-windows-package-failure.log`、SHA256 `8ba5050b4ddb8ac2f987c2a2f104f45206deaf54c808c13c2248eb0cac5d3098`。
- 実stdout: `tmp/oct26-6862-windows-startup-diagnostics/stdout.log`、SHA256 `6d09c5735889ae71fcf082fdf77a734de6501a53831a36eb4169cada98a9b2a7`。
- 対の解析: `tmp/oct26-6862-windows-ui-phase-result.json`。解析は時刻・対応関係の確認であり性能合格判定ではない。

## 証明できた境界と未確定部分

使用中のeframe0.36.2では、Windowsの`RepaintNow`経路がUIを呼ぶ前に、`Resized`イベント内でsurface再設定を実行し得る。stdoutには`Event::Resumed`1件に対し`RepaintNow`6件がある。egui-wgpu0.36.2のresize処理は同じ寸法でもsurfaceを再設定し、wgpu30.0.1の再設定はGPU idleを待ち得る。しかし、実空白区間の何msがresize・描画・OSメッセージ待機・ログ出力なのかは未計測。

KatanAの全viewport commandを確認し、空workspaceの各フレームでサイズ変更を発行するコードは見つからなかった。初期サイズ1280x800・最小800x500・最大化・中央配置の設定だけでは反復resizeを発生させる根拠にならない。初期配置とviewport patchの実装も確認した。TRACE同期出力の負荷はあり得るが、255msの無ログ区間やTRACE追加前の不合格を説明し切らず、単独原因とは断定しない。

## 次の実行

元smokeが失敗して終了処理を終えた場合だけ、同じ抽出済み配布本体を新しい空の検証設定で起動し、Windows公式のWPR/ETWが利用できれば補助記録を採る。記録は元smokeの計測後に行い、200ms×10・100ms再描画予約・RSS・font・worker・本体identity・元の失敗を変更しない。本人の設定やTerms承認を触らず、driver/backendや窓サイズも変更しない。

WPRとprofile・記録状態を実runnerで確認し、他の記録がある場合や状態が曖昧な場合は開始しない。専用のGUID付きinstance名を開始前に記録し、その同じinstanceだけを停止・保存する。開始結果やログ保存が失敗しても、そのinstanceに限定して後処理する。他の記録のcancel/forceや保護解除を行わない。各外部commandの待機にも上限を設ける。ETL・変換結果の存在は原因特定や起動合格ではなく、実データを回収して待機箇所を判別する材料とする。

公式仕様: [WPR CLI](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/wpr-command-line-options)、[専用instanceの指定](https://learn.microsoft.com/ja-jp/windows-hardware/test/wpt/wpr-command-line-options#インスタンス名)、[記録profile](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/built-in-recording-profiles)。この診断の実行・原因判定・必要なKatanA修復・通常統合・新CI・全OS公開は未完のまま追跡する。

## 対象検証

- 診断scriptのsource契約7件・既存全OS公開契約4件は主査実行で成功。初回候補に対する所有・期限の3項目の不足を検出し、専用instanceと未知の開始結果の後処理を修正した。公式の記録中statusを拒否する不足も修正し、記録なしを誤認しない正負の文字列判定を確認した。
- 既存の起動契約scriptは実exit0。既存の失敗fixtureの拒否を成功へ読み替えず、終了処理の各時間・identity条件も維持した。
- 既存の公式PowerShell7.4.2のamd64コンテナをネットワークなし・repo読取り専用で実行し、Parserエラー0、実.NET正規表現の公式status正負判定を確認。検証前後のscript SHA256は `a540a15859e149f2b9ef2d29063f65a135930d223b43b486eb5ee3e587bb380c` で一致した。
- 元のWindows smoke blockのbytesは変更前と同一、SHA256 `726979a6a0eb68c1eb8c64f4b277c4fef923d9f67043b21469caa376b7ac7cd7`。この確認は、追加診断のWindows/WPR実行や起動合格の証明ではない。
- 2026-10-26の最新本人回答は「元基準を維持して調査・修正を続ける」。200ms条件の例外は適用しない。

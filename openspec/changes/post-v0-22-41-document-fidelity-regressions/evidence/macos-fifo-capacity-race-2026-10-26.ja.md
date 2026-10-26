# macOS coverageのFIFO容量fixture競合

## 実際の失敗

- source `32807aa8da79448fb4edf1f7e4cbe0047395ed36`、CI [40256001313](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40256001313)、macOS job `121357855783` のcoverage実行がexit101。全CIはfailure、rootのexact watchもexit1だった。他7 jobsとRelease Readinessの成功を全CI成功へ読み替えない。
- 親 `cancelled_fifo_reads_terminate_pending_requests_and_recover_capacity` の子 `fifo_capacity_child` が、`local_intake_capacity_tests.rs:142` の `pane.document_intake.is_none()` で失敗した。子は0.01秒で終了しており、元5秒期限の不足ではない。
- 実raw `tmp/oct26-328-macos-ci-failure.log` はSHA256 `c5c7e059ec7d49d19f6fc962ebde2711f2419985727919da6244ddf64803c408`。共有assertのqueued/laterどちらの呼出しだったか、どの取消checkが先行したかまではstackから確定できない。
- 同sourceの通常テストは成功。修復前ローカルの既存 `just T=preview_pane::document_surface::local_intake::capacity_tests test-specific` もexit0・3件成功だった。これはCIの実REDを否定せず、再実行成功による免除にも使わない。

## 原因と修復範囲

FIFO writerのopen成功はreaderとのopen接続を保証するが、workerがその後の取消checkを通過してOS readで停止していることは保証しない。`SourceLocalOps::read` と `CancellableReader::read` にはopen後・metadata後・read前の取消確認がある。早期取消を正常に検知したworkerは `ActiveReadGuard::drop` で枠を返せる。その場合、queued/laterの通常PDFは正当に受理され、結果到達前の1回のpollではintakeが残る。

旧fixtureは、この正当な順序でも「2枠が必ず残り、必ず容量エラーになる」と仮定していた。製品の取消遅延や容量不具合を示す証拠ではない。取消を遅らせる商用変更、固定sleep、無条件retry、期限延長で合わせない。

修復はテストの責務を分ける。

1. private poolの単体検査で、実requestの取消・2 active・対象の非active・容量拒否・active exemption・枠返却後の再受理を決定的に検証する。OS readの実動作を証明するITとは呼ばない。
2. 実worker/FIFO/PDFを使う既存子検査では、元5秒内のintake終端を必須にする。容量エラーなら元のnone/非loading/operation/causeを全て維持する。正当な読込成功なら、非loading・エラー無し・canonical URI・PDF形式/MIME・原本revisionを厳密照合する。pendingや別エラーを成功扱いしない。
3. writer終了、active全解放、通常PDF再読込を必須のまま保ち、回復後は原本bytesも照合する。intake成功を文書描画完了と取り違えない。

製品のpool/取消処理・公開API・可視性・worker数/queue数・元5秒・親のkill/reap・coverage/ignoreは変えず、mock ITや商用testportを追加しない。Issue [#359](https://github.com/HiroyukiFuruno/KatanA/issues/359) のfixture修復として追跡する。

## 検証の状態

- sourceと実ログに基づく独立原因レビューは完了。候補は容量単体検査、実FIFO子の厳密終端、通常PDFで成功側を必ず通す検査、回復後の原本bytes照合を実装した。製品の差分は既存poolへのcfg(test) module配線だけである。
- 最終候補の既存 `just T=preview_pane::document_surface::local_intake test-specific` はexit0・19件成功（`tmp/oct26-fifo-intake-reviewed-final.log`）。その前の18件成功は、成功側を必ず通す追加ケースより前の結果として区別する。
- 公式 `just fmt-check` は最終候補でexit0。`just ast-lint` の30件成功はAST linter自身の単体検査であり、全sourceのAST適合や全coverageの成功とは呼ばない。全sourceの通常hook検査は通常commit/pushで、新HEADの全coverageはReady CIで回収する。
- 独立差分レビューは未対応P0/P1 0件。取消済みrequestが残っていても1枠返れば再受理できる単体検査へ補強し、失敗診断の不要なpanic closureも元のexpectへ戻した。最終再検査19件・整形・diff checkのexit0を主担当が回収した。通常commit/push以降は別工程として回収を続ける。
- 新HEADのcurrent review、全必須CI、全OS実配布起動、GitHub Release/全5配布物、公開後の証跡更新は別工程であり、この診断で完了とはしない。

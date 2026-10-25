# Linux FIFO取消テストの接続競合

## 観測と原因

HEAD `83cf2bdb089d740e097cff99589f199793b2df70` の [Ubuntu job 121124753214](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40178653330/job/121124753214) は、UIの他テスト成功後、FIFO取消capacityテストを終了できなかった。Chromeの実行中ログで特定し、取消後の実ログでも09:09:00 UTCの60秒超通知から09:48:37の取消まで終端がないことを確認した。runの結論は `cancelled` であり、自然終了したテストのfailureではない。

旧fixtureの `sender_is_taken` は、workerがsenderを取得したことだけを示す。`SourceLocalOps::load/read` にはその後、FIFOの `File::open` より前に取消判定がある。取消されたreaderがopenしないまま終了すると、後処理の新しい `File::create` は接続相手を無期限に待つ。親の `output()` も無期限だった。これはfixtureの競合経路であり、通常の文書で製品が同じ停止を起こす証明ではない。

## 修復と検証範囲

実FIFOのnonblocking writer openでreader接続を確認し、writer handleを保持してから取消する。後処理は保持handleのdropによるEOFとし、新しいblocking openを行わない。親のchildは既存の期限付き回収パターンを使い、失敗時もkill/waitする。

製品のworker/queue、取消処理、既存5秒予算、ignore、元のcapacity拒否・通常入力の回復assertは維持する。新規API・製品testportを追加しない。

## 回収した結果

- 旧fixtureのblocking writer行を実FIFO・reader不在の独立Rust childで実行し、1秒後も停止することを再現。回収harnessがkill/waitしexit=-9。旧fixture全体の自然終了failureとは区別する。
- 最終sourceの対象3件が成功。実取消処理がopen前に取消を返すFIFOで、新接続確認は期限切れを返し、reader不在以外のI/Oエラーを保持する。既存のcapacity拒否・通常入力の回復も成功。
- 同じ最終binaryを100回反復して100/100成功（3件/回、計3.536秒）。外側回収保険は7秒だが、testの5秒期限は不変。全scheduleでの安定を証明したとは扱わない。
- AST30件、fmt、影響lint、差分検査が実exit0。
- 追加のtest-inclusive Clippyは背景test moduleの配置でexit101。内容を変えずmoduleを末尾へ移し、同じClippy条件を再実行してexit0。背景回帰1件、fmt、差分検査もexit0。通常lint成功だけでこの追加失敗を隠していない。

FIFO修復source SHA-256: `4e0af5191f02d2f5a133457865dc9c8d7ff3125c52e2662a6619b9d6886a83a0`。

独立最終レビューで上記sourceと、配置だけを変えたsplitのSHA-256 `863b4a7728ab529dd031b8d2181ba26a8947232553bd2b7c019090fd35e5ec32` を照合し、P0/P1は0件。splitはmodule内とそれ以外の内容が同じであることを主査・独立で確認した。

実OS接続はreaderのopen到達を示すが、次のread syscall到達までは証明しない。接続後の取消によりactiveが早く解放されるscheduleでは元capacity assertが失敗する余地を残す。assertを外して合格させず、親の期限で無期限待機を防ぎ、新CIで実結果を確認する。

新HEADの通常commit/push・Linux全体と全必須CI・公開確認は継続中。現時点で新CI合格や公開済みとは扱わない。

関連: [KatanA #359](https://github.com/HiroyukiFuruno/KatanA/issues/359)。本人の実機確認はClose条件に含めない。

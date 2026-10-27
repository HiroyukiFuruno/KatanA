# Windows の描画同期待ちを減らす候補

## 同じ配布物を使った実測

正式 run [40388796186](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40388796186)、source `d2b78cefd967bb5995b3000facc2ace17366256b` の実配布物を比較した。

| 環境 | 元の起動検査 | 記録された frame 開始数 | 開始間隔の中央値 |
| --- | --- | ---: | ---: |
| Windows Server 2025 | 不合格 | 8 | 274.240 ms |
| Windows Server 2022 | 合格 | 21 | 298.053 ms |

main SHA-256 は両方とも `42c7e4f91fa6cc564caad2515ef75ce543e6462a9ff64036e2558fd413abab3b`、worker は `ec819e7a1fe76cce6a51669ae9486fcb520f4952b382768a4776587136243bc9`。比較側は再ビルドせず、元の smoke 本文をそのまま実行した。

両環境とも実際の描画間隔は約 300ms だった。合否の違いだけで描画遅延が解消したとは判断できない。検査ループには 200ms の待機以外の処理も含まれるが、その所要時間の内訳は未確定である。合格した runner への切り替えや基準緩和で済ませない。

実記録は `tmp/oct27-d2b78cef-same-payload-windows-host-comparison.json`、回収した比較 artifact `12687150353` と元 Windows 診断 artifact `12686068707` に対応する。macOS の build・両 arch 起動は同 run で成功したが、Linux/Windows 失敗により Publish/cleanup は skipped で、公開成功ではない。

## 実装候補の範囲

現在の eframe 0.36.2 の既定値は `SurfaceConfig::LOW_LATENCY`、`AutoVsync`、最大待機 frame 数 1。wgpu の画面取得や表示には同期待ちの経路があるが、採取済み ETL だけで今回の待機開始 API を確定してはいない。

Windows のみ、既存 API の `AutoNoVsync` を使う候補を検証する。対応する表示方式はライブラリ側が選び、利用できなければ対応済み方式へ戻る。非対応の `Immediate` を強制しない。最大待機 frame 数 1、Windows 以外の既定設定、協調的な再描画要求、実起動検査の条件は維持する。新依存、未公開 fork、試験専用の描画分岐は追加しない。

垂直同期を待たない方式では画面の一部が異なる frame になる可能性や、連続描画時の GPU/CPU 使用量増加の可能性がある。設定を変えただけで改善とせず、通常検査・レビュー・元の Windows 起動検査と実際の frame 記録で候補の効果を確認する。

## 未完了の検証

候補の bin 対象検査 1 件、strict Clippy、rustfmt と独立レビューは成功し、対応が必要な P0/P1/P2 はなかった。通常 commit/push、current review、全 CI、新配布版の実起動・記録回収は継続中。元 200ms×10、RSS/font/worker/本体照合、全 OS 公開の条件は変更しない。旧 run の成功や比較だけを、新候補の合格へ流用しない。

## 合格時にも実際の描画間隔を確認する

元 Windows smoke の終了処理へ、停止後の stdout/stderr/heartbeat を別名 `completed-*` で保存する 3 行だけを加え、合格時も診断 artifact を回収する。従来の失敗時スナップショットは上書きしない。コピーの追加を除けば、元 smoke 全体が byte 単位で一致することを検査した。

元ブロック SHA-256 `726979a6a0eb68c1eb8c64f4b277c4fef923d9f67043b21469caa376b7ac7cd7` に対し、新ブロックは `f4a7e1010e974d03171d5d5ba4b046da701328c98fc5670d08559afeaac4b852`。比較 helper の参照も新しい固定値へ更新し、200ms を 201ms へ変える改変を拒否する検査を含めて 6 件成功した。記録追加前の回帰検査では、終了処理に保存がないことを検出した。回収失敗を空 artifact の成功と扱わない。

検査自体の PASS だけを改善の証明とせず、新しい実ログの描画間隔も数値で確認する。これは既存基準の緩和ではない。

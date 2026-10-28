# Windowsの画面提示・GPUイベントを追加採取する

## 現在の一次結果

- PR373のHEAD `1daa14c6abf4664aeda7cc206313da657d0e1b37` は、通常push、全ページのレビュー取得、重大指摘なし、未解決thread 0、必須CIとReadinessの成功を確認してマージした。マージ `234bc3819e040e203b74d14a0032db6f80323e54` は署名検証済みで、検証したHEADとtreeが一致する。
- このマージを入力とする正式公開 [40578480784 / attempt 1](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40578480784) は終端failure。Linuxの配布生成・起動、macOSの配布生成・ARM/Intel両方の起動、同一Windows配布物のServer 2022比較は成功した。
- 元のWindows 2025配布起動は `last_frame=4; current_frame=4` で不合格。例外ログ時刻は `2026-10-27T21:13:04.3725209Z`。Publishと公開後cleanupはskippedであり、v0.22.42の公開成功ではない。Server 2022の成功を元環境の成功へ置き換えない。
- 後続のheartbeat snapshotはframe 5/7まで進んでいるが、失敗時の4/4とは時点が異なる。恒久停止とも、元検査の合格とも扱わない。
- 今回のCPUはEPYC 7763で、以前の既定構成2ケースの成功時にも同じCPU名を観測していた。CPU名の違いだけを原因とする説明は採用しない。

今回の停止条件は、実配布物の既存必須検査の不合格である。補助診断の採取成功を、新たな公開必須条件として追加する変更ではない。

## 次に区別する区間

既存のCSwitch/ReadyThread記録には待機スタックがある。一方、今回と既存4本のETL由来CSVでは、検査したDxgKrnl、DWM-Core、DXGI、Direct3D12、DirectCompositionのprovider GUIDに一致する行は0件だった。profileで有効になっていなかったのか、その区間に発火しなかったのかは、この集計だけでは確定しない。汎用Win32kやImageLoadの行を、画面提示イベントの代用にしない。

そのため、既存の任意診断 `wait-stacks` の記録に限り、待機スタック用profileとWindows組込の `GPU.Verbose`、`DesktopComposition.Verbose` を併用する。[MicrosoftのWPR CLI仕様](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/wpr-command-line-options)で定義された複数profileのstartと、組込profileのexportを使う。

## 実装の境界

- 追加処理は `WaitStacks` がtrueの場合だけ。通常のGeneralProfile経路、アプリ、元smoke本文、Sleep 200ms×10、latency 1、RSS/font/worker/binary identityの条件は変更しない。
- 組込の2定義を各30秒以内の既存bounded native helperでexportし、実際の定義、SHA-256、WPR versionを記録する。欠落・export失敗は明示的な診断失敗とし、代替profileへ黙って変更しない。
- 記録のstart/stopは既存のunique instanceだけを所有する。default instanceの停止、他の採取への干渉、OS保護の無効化は追加しない。
- start後は、3つのprofileがMemory modeで有効と表示されたことを確認し、requested profiles、実status、start/statusのexit codeを保存する。0件でも空配列として判定する。
- 既存selection JSONのprimary profile/path/hashキーを保持し、追加profileの記録を拡張する。profileの有効確認だけでは、実ETLに必要なイベントがあることや原因を特定したことにはならない。
- 同一配布物・元smokeの結果と、別の所有processで行う追加採取の結果を分けて記録する。新しい製品表示設定やdriverを採用する変更ではない。

## 検証と残作業

対象trace契約11件と診断契約18件、差分検査が成功した。独立した差分の機械レビューはP0/P1なし。Linux上のPowerShell 7.4.2の実parserで構文エラー0を確認し、本体から抽出したstatus一致式の0/1/2/3件を実行して期待するCountと一致した。これはWindows PowerShellやWPRの実動作の代用ではない。通常commit/push、Draftレビューを通して同一配布物で追加採取する。

新しいGPU/画面提示イベントの実採取、待ち区間の判別、必要なKatanA修復、修復後のレビュー・必須CI・正式全OS公開は未完了。今回の診断入口の追加を、製品の修復や公開完了とは表現しない。

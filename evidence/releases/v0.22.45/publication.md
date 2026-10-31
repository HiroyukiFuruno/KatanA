# v0.22.45 公開結果

[GitHub Release](https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.45) / [実装PR402](https://github.com/HiroyukiFuruno/KatanA/pull/402) / [公開run41207347968](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41207347968)

- 公開元は署名付きmerge 6bb5a5b0ccc455d10a2b162050cd4a7088382fe1。イベントhead c6864bc8c7fda05fc4a9c2d6eab7ca2fddd98fb0と区別し、実tag・全6署名証跡のsource/run/attemptを照合した。
- 最終製品HEADのReady CI41200391869・Readiness41200391876と、実merge sourceの3OS CI41207347589が成功。全26製品コミットはVerified。Windowsの内側producerがSystem32のWSLを選ぶ問題を指定PATHの実行ファイル解決で修復し、実Windowsの25契約も成功した。
- 全5配布物とchecksumsを実ダウンロードし、サイズ・SHA256・GitHub API digest・正本CI receiptの一致を確認。全6署名付き来歴はGitHub-hostedの正しい公開workflow・source・run/attempt・subject digestを暗号学的に検証した。
- macOS arm64/x86_64・Linux x64・Windows x64の配布版起動検査が成功。Linuxはsoftware Vulkan環境であり、物理機の性能実測ではない。Windows Server2022比較診断のSKIPPEDを成功に数えない。
- 3OSの実raw identity manifestを生成・保存し、公開後Readiness41213893520で実公開assetを再取得して照合した。publication-identity.jsonと3manifestは同CI artifactのbyteそのままで、文書QAや95点の証明ではない。
- Homebrew/Linuxbrewのlatest・versioned各4ファイルは0.22.45と実公開SHAに一致し、Ruby構文も成功。WinGetは[PR467087](https://github.com/microsoft/winget-pkgs/pull/467087)へ提出済み。Microsoftの検査・merge・catalog反映は別追跡であり、この記録はWinGet配信済みとは主張しない。

## 公開した改善と残件

更新前の空き容量確認と容量・権限の原因を失わないエラー表示、画像より下の制御ボタンと後続レイアウト維持を公開した。互換依存を更新し、両lockfileを同期。採用上流は公開registryのKRR0.4.23/KDV0.5.12であり、次版の採用は次回へ分離した。

[#385](https://github.com/HiroyukiFuruno/KatanA/issues/385)の原PNG30秒は未再現・原因未特定。本人指示で追加調査を終了して現在構成を公開した。解消済みとはせず、inactive PNGのprivate cache消去漏れも未実装の後続として残す。上流レンダリングの未解決や本人実機確認を完了扱いしない。

レビューP2の[手動version bump公開後のidentity結合 #403](https://github.com/HiroyukiFuruno/KatanA/issues/403)は未実装。この公開は検証済みのPR closed経路を使い、手動bumpを含む全経路対応済みとは主張しない。source/run/attempt照合を弱めず後続対応する。

元のメモリ予算・Windows描画条件・終了条件・必須自動検査は変更していない。本人確認はIssue Close条件に含めない。公開後記録では製品・lock・workflowを変更せず、公開sourceとの一致を維持する。

ローカルの完了したrelease/v0.22.45は正式mergeを確認して通常削除した。remoteの同branchは既存公開Actionsのcleanupが削除しており、ローカルから追加のremote削除はしていない。記録branch自体の整理は記録PRの正式merge後に行う。

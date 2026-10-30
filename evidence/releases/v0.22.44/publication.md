# v0.22.44 公開結果

[GitHub Release](https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.44) / [実装PR397](https://github.com/HiroyukiFuruno/KatanA/pull/397) / [公開run41002934022](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41002934022)

- 公開元は署名付きmerge `20bf151f40165084c8748b4cca8c7ab54ff6bdf2`。PRイベントのhead SHAと混同せず、実tag/API/local tagと全6署名付き来歴のsource/runを照合した。
- 製品HEAD b70d05bfの[Ready CI40992549904](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40992549904)・[Readiness40992549926](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40992549926)と、merge sourceの[3OS CI41002933192](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41002933192)が成功した。
- 全5配布物を実downloadし、checksum/API digest・同run成果物とのbyte一致を確認。DMG/MSI内の本体・Office workerも対応ZIPと一致した。
- 配布版の起動検査はMac arm64/x86_64・Linux x64・Windows x64の4環境で成功。LinuxはCIのsoftware Vulkan環境であり、物理機性能の実測ではない。Windows Server 2022比較診断はSKIPPEDで成功に数えない。
- Homebrew/Linuxbrewのlatest・versioned各metadataは0.22.44と実配布SHAに一致し、4 Ruby構文検査も成功。新しい登録tokenによる実書込を公開runで確認した。
- WinGetは[PR465985](https://github.com/microsoft/winget-pkgs/pull/465985)への提出成功。Microsoft側の検査・merge・catalog反映は別追跡であり、この記録は配信済みとは主張しない。

## 公開した改善と残件

共有Loadingの両軸中央配置、PDF/PPTX活性化・次/前タブ後の左右キー、画像選択の同期stat除去、allocator/HTML終了待ちのUIからの分離、不要な文書初期化・描画の取消を公開した。入力所有の10回帰、Loadingの5対象と実caller、実Explorer混在描画、全テーマの分割背景の回帰を維持した。

[#385](https://github.com/HiroyukiFuruno/KatanA/issues/385)の原PNG30秒・Missing全条件は未確定のためOpen。局所改善を全面解消とは呼ばない。[#386](https://github.com/HiroyukiFuruno/KatanA/issues/386)は、既に実行中の大型PPTX native openを中断できない上流境界を[KDV72](https://github.com/HiroyukiFuruno/katana-document-viewer/issues/72)へ分離し、残件を維持した。上流レンダリング全体の95点比較や本人実機確認を完了扱いしない。本人確認をIssue Close条件に含めず、公開後の新規不具合として追跡する。

元のメモリ予算・終了条件・必須自動検査は変更していない。公開後controllerは版別の固定契約を共有し、製品/lock/配布元sourceの変更を許可しない。

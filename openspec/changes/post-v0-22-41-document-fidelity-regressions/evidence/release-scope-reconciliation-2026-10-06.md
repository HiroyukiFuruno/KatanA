# v0.22.42 公開範囲の整合

## 追加検証と現状

- reload契約は新sessionのframeを検証するce3ffaa2へ補正した。新sessionではframeがリセットされ得るため旧sessionとの大小比較を使わない。RED3ERROR→scope10PASS/gate26PASS、旧strict48PASSを維持。実配布receiptはまだ欠落。
- 公開042の3OS結果を再取得しLinux/Windows成功・macOS失敗を確認。通常push91452もnative1099PASS/1FAIL/既存ignore2でactualexit1、未upload。失敗確定後の元snapshotを維持した同一test binary採取候補は公式4PASS/AST23/Clippy/fmt/CI契約8PASS。単独対象1PASSを原因修復としない。
- default本人profileの既存Terms承認0.22.9をreadonly確認した。現UIはSomeを受理するためdefault再承認待ちを停止条件にしない。隔離clean-machine profile/本人最終受入は未確認であり、自動承認や設定複製はしない。

- ビルド前source gateと公開前packaged-host gateを分離した。後者は実配布binary、元入力・参照・測定hash、全対象、正常close/RSS、scroll/reload/dirty-sourceを必須とし、KRR106の測定済みChrome乖離だけを未解決として区別する。未測定やhost操作欠落は拒否する。
- 機械回帰は正式evidence venvでgate26件、divergence9件、既存strict48件、flow8件が成功。初稿fixtureのhost_actions欠落1FAILと標準PythonのPillow不足1ERRORは保持し、補正後結果と区別する。実mainの配布受入receiptはまだ生成していない。
- 両manifestの最新互換/major/pinned dry-runとJS依存確認後、両lockのctutils0.4.3、zerocopy/derive0.8.60だけを更新。KDV0.5.12/KRR0.4.23の公開sparse index非yank/checksumを再確認した。上流次期版を独自採用しない。
- 更新graphの画像loader16件、AST23件、strict impacted Clippyは成功。旧候補watcher1FAILは原因未確定であり今回の成功を原因解消と読み替えない。
- GitHub公開HEAD04269741のmacOS通常suiteは1096PASS/1FAIL/既存ignore2。dirty-target HTMLの初frame2秒待ちが失敗した。後続別プロセスの診断成功は元失敗の修復証拠ではない。Linux成功、Windows未完了、元期限不変で切り分けを続ける。

## 正本となる本人決定

KDV/KRRの不備取り込みは次期とし、今回はKatanA所有の改善を完了して公開する。Issue作成は上流編集許可ではない。未解決の描画差分・変換遅延・再起動artifact不足は既知制限として公開する。独立95点、Chrome完全一致、全不備解消を主張しない。

## 今回の必須事項と次期事項

| 対象 | 今回のKatanA必須事項 | 次期・未解決の上流事項 |
| --- | --- | --- |
| HTML（3.2） | 正常なsource/dirty保護、入力転送、scroll/reload、表示・正常close、操作回帰 | ChromeのCSS/JavaScript一致性: KRR #106 |
| Office ZIP（3.4） | 公開版で開ける合法DOCXの表示と安全性回帰、明示的な失敗表示 | 報告されたPPTX ZIP local-headerエラー: KDV #67 |
| Office描画（3.5） | hostの実書体・Unicode identity、セル罫線/geometry、inputを検証 | host未検証を上流延期へ移さない |
| 初期読込（3.6） | 背景intake、UI非停止、段階別実測、同一入力の回帰 | DOCX/PDFの変換・初期描画: KDV #65 |
| 再起動cache | 公開API境界と未実装範囲を明記、静止画を操作session復元と偽らない | KDV #66、KRR #106（#107は統合先への移管であり解決ではない） |
| 上流採用（5.1/5.2） | 公開KDV=0.5.12/KRR=0.4.23のregistry/lock/checksumを検証 | 上記の将来修正版の公開・採用を今回の待機条件へ戻さない |

Issue URL:

- https://github.com/HiroyukiFuruno/katana-document-viewer/issues/65
- https://github.com/HiroyukiFuruno/katana-document-viewer/issues/66
- https://github.com/HiroyukiFuruno/katana-document-viewer/issues/67
- https://github.com/HiroyukiFuruno/katana-render-runtime/issues/106

## 変更しない公開条件

全build/test/lint/coverage/supply-chain、現在HEADのreview/required checks、全5配布asset/checksum、宣言済み4CPU/OS targetの実packaged/clean-machine受入を残す。正常closeは5000ms、cold RSS増分196608KiB、warm増分65536KiB、UI heartbeatと旧世代終了・idle資源0を維持する。実main/sidecar/source/input/PNGのidentityを束縛し、in-process、合成receipt、以前のHEADのCIを代用しない。本人確認・Terms承認を代行しない。

## 現物監査と残実装

2026-10-06時点のrelease checkerは3.2を必須未完了として拒否し、acceptance checkerはHTMLのChrome geometry/active state完全一致を無条件に要求する。3.2だけをallowに追加してもHTMLの次期事項が再び公開停止条件となるため、それだけの案は不十分。

- [ ] tasks/spec/designを今回host要件と次期Issueへ明示的に分離する。混合タスクを全てdone/allowにしない。
- [ ] Issueに厳密に紐づく延期契約を機械検証する。未知task・任意allow・host要件の延期を拒否する。
- [ ] HTML参照と実packaged測定の保存・identity検証を維持し、既知の上流差分を未解決として記録する経路を設ける。差分を消す、参照をKatanAから生成する、95点成功へ書き換えることは禁止する。
- [ ] host入力/scroll/reload/正常close/資源/packagingの欠落を延期契約で回避できない回帰を追加する。
- [ ] 元のstrict経路の失敗検知と新しいhost公開範囲の正例/負例を実行し、通常履歴/current reviewへ統合する。

この文書は範囲監査の証跡であり、上記ゲート実装・受入・公開の完了証明ではない。

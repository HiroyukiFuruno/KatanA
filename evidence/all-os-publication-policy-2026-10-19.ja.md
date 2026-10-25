# 全OS一括公開への統一

## 本人決定

2026-10-19、version管理上のOS対応は一律とし、緊急時のOS単独公開機能を残さないと本人が決定した。公開方針の回答待ちは解消し、通常・タグ・手動実行を全OS一括へ揃える。OS単独公開を次期の必須機能として追加しない。

## 実装

- `.github/workflows/build-and-release.yml` のmanual target入力とmacOS/Linux/Windows build・macOS smokeのOS選択分岐を撤去した。
- `just/release.just` の正規callerから旧 `-f target=all` を除去した。
- publishのpreflight/全3OS build/macOS両arch smoke成功条件、全5配布asset、identity/checksum、供給網、attestation、配布先更新は変更していない。失敗したOSをskip扱いにして公開する修正ではない。
- 新契約は既存preflightのrelease Python discoveryから実行される。

## 検証

旧契約でtarget入力/旧callerを検出する2FAIL/actual1を確認した（`tmp/oct19-release-all-os-red.log`）。主査で当初のjob切り出しが空になるテスト不足を検出し、行頭のjob境界へ修復した。

最終契約は各4jobの `if: false` 再導入を個別拒否し、step-level `if: success()` は受理する。主査が変異そのものの実在もassertした。公開条件の部分化・identity/MSI欠落を拒否する。

- 新契約4件/0.004秒/actual0: `tmp/oct19-release-all-os-main-final.log`。
- release flow10件/actual0: `tmp/oct19-release-flow-contract-final2.log`。
- CI admission12、owned check2、packaged契約8、公開identity契約8は各actual0: `tmp/oct19-release-{ci-admission,owned-check,packaged-contract}-final3.log` と `tmp/oct19-release-published-identity-final.log`。
- default actionlint1.7.12 actual0: `tmp/oct19-release-actionlint-final3.log`。空ログは成功時の出力で、実Cloud実行の証跡ではない。
- 主査ASTで新test83行/全関数30行以内、diff/変更文書KML成功。独立最終レビューのP0/P1/必須P2は0件。

## 未完了との境界

これはローカル設定・caller・回帰の修復であり、実GitHub Release/全OS配布物の成功ではない。KDV #69の安全性修正版採用と通常push後に、manual-targetの3review threadへ各reply/resolve・全件再取得を行う。未pushの修復を公開済みとしてresolveせず、元のDraft→review→Ready→必須CI→公開・後処理を維持する。

# 最新本人指示に基づく公開判定

## 今回の担当範囲

本人は自動テストに含まれない手動確認を今回の公開停止条件にせず、公開後に自身で確認し、残る不具合は改めて対応するよう指示した。背景色の修復は今回に含める。また本人による実機確認・本人だけの承認はIssue Close条件に含めない。

既存の自動build/test/lint/coverage/供給網/全OS CI、起動検査、全5配布物のarchitecture・main/worker・checksum・source/run一致を維持する。旧strict/katana-hostの手動証跡入口は明示指定の検査として残す。旧測定の不合格や未採点を合格・95点へ書き換えない。

## 判定経路

v0.22.42の通常公開には`automated-release`を使う。最新承認文書の固定SHAを確認し、元のsource実装markerと未知のtask拒否を維持したまま、手動host JSONを要求しない。公開前は現workflow run/attemptとbuild commitを、全3OSのmanifestと実5assetへ照合する。

公開後は既存の固定evidence branchだけで`post-release-automated`を使い、安定版Releaseの存在と実tagを確認する。現在のdocs-only HEAD/runを過去のbuildへ誤って強制せず、実buildのrun/attempt・tag・全3manifestと新規downloadした全5asset/checksumを照合し、production sourceに変更があれば拒否する。

## 検証

最初の9file統合はflow15/Draft5/gate35/artifact9/source-publication6、計70件で実exit0。独立レビューで、公開後checkerがworkflowに未接続であるP1を検出した。既存download/manifest入口をworkflowへ接続する2file修復を行い、flow17/Draft5/artifact9/source-publication6の計37件が実exit0、抽出Bash構文も成功。REDは16件中1ERRORで実exit1を保存。独立再レビューで接続済みを確認し、新P0/P1なし。最終2file SHAは主査も照合一致。現時点で実GitHub Release公開や全5asset検査完了を意味しない。

元ログは`tmp/oct25-automatic-release-integrated-validation.json`と個別log、独立レビューは`tmp/oct25-automatic-policy-independent-review.md`。署名G `8cf6b7c8`へ通常commit実exit0。push、新HEADのcurrent review、Ready後の必須CI、merge、全OS公開と公開後検査は別々に実行して記録する。

# 通常セットアップのdispatcher接続（2026-10-19）

対象: [Issue #335](https://github.com/HiroyukiFuruno/KatanA/issues/335)。実装前HEADは80e0bded。

## 修復

通常setupがLefthookだけを導入し、既存の依存公開証跡dispatcherを導入しない配線漏れを修復した。
再実行時にはLefthookがdispatcherを置換するため、installerの失敗後に検査が外れたままになる不備も実fixtureで再現した。

setupを専用transactionへ接続し、既知dispatcherを保存してからLefthookとinstallerを実行する。
Lefthook非0・installer非0/例外時は、生成後hookのbytes/modeが変化していない場合だけ既知dispatcherを復元する。
未知wrapper・backup・symlinkを正規扱いせず、既存ownerの順序・終了codeを維持する。

## 検証

- 対象28件: 全件成功、7.753秒。`tmp/oct19-335-final-hook-tests.log`。
- 実Lefthook 2.2.0と一時Git repository: 同一owner再導入成功。
- 異なるowner生成後のinstaller拒否: 終了2、既存dispatcher復元・backup保持。
- hook生成後のLefthook終了23: 終了23、既存dispatcher復元・backup保持。
- 実試験ログ: `tmp/oct19-335-real-setup-rollback.log`。
- bytes/mode変更後の復元拒否・未知wrapper/symlink拒否を回帰検証。
- 構文・diff検査成功、独立最終レビューのP0/P1/必須P2は0件。

## 未完了

実作業repositoryのhooks・Git設定は変更していない。公開後のactivation、実cleanup receipt、current公開契約の証明は未完了。
この修復をIssue全体、必須CI、実配布物受入、GitHub Releaseの完了へ代用しない。
通常pushは別途、既存KDV推移ZIPの供給網ゲート失敗が解消してから行う。hook迂回・検査緩和はしない。

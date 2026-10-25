# #343 機械inventory解析の限定工程

対象: https://github.com/HiroyukiFuruno/KatanA/issues/343

## 実装範囲

libtest一覧のname/kindを保ち、重複・相反kind・未知の非空行・件数不一致・footer後の内容を拒否する。空行、CRLF、正当な0件/0ignoredを扱う。defaultではfooterを要求する。historical case-only fragmentを明示的に診断する場合だけfooter不要を指定し、戻り値のfooter_counts=Noneで未検証を保持する。

ignored一覧は通常一覧のsubsetとして検査する。skip substringは別の選択条件として保持し、元一覧を変えない。選択数は成功数ではない。source/OS/features/環境・実行元の信頼性やdoctestは、このparserだけで推定しない。

既存preflightの純UT discoveryを一回だけ一般化し、重い検査の追加実行・削除や閾値変更は行っていない。

## 実検証

- 純format8件と既存downstream26件、単一discovery34 PASS/exit0: `tmp/issue343-inventory-parser-main-2026-10-14.log`。旧案のpytest依存・空行拒否・0ignored拒否・関数長をmainで補正した。worker案を実検証成功として扱っていない。
- 実保存一覧をparserで読んだ。export14件、fixture skip後13件、UI lib1242件、ignored2件のsubset検査はexit0。保存一覧のfooterは無いためfooter整合は未検証である。これらは78162c26時の一覧であり、新HEADのnative test成功・coverage同等性ではない。
- Python構文、zsh構文、diff検査、file200行以下/function30行以下を確認した。
- 通常testとcoverageにはsource cfg、fixture/ignored、doctest、Windows cold/DEBUG、serial条件の差が残る。`update/cleanup.rs`のcoverage版no-op、`update/installer.rs`のcoverage除外は実code差であり、同名テストを無条件に統合しない。

## 未完了

全native/coverageの同条件collector・機械集合比較、保護されたCI成功元とsource/OS等のbinding、stale/dirty/tamper拒否、Windows cold/warm、導入前後時間比較は未完である。auto reuse/skipやCI成功代替を導入していない。DoDを下げず、#343とrelease全体を閉じない。

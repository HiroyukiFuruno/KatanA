# 下流Issue証跡の構文検証（限定工程）

対象: https://github.com/HiroyukiFuruno/KatanA/issues/335

## 実装範囲

Issue本文の明示markerとJSON blockを検査する。markerが無い本文は未分類であり、下流チェックの免除ではない。上流公開版・API移行説明・manifest/lockfile・検査ログ・source commitの必須項目、型、SHA、重複、安全な相対pathを検査する。作者が記録したcommandは実行しない。

GitHub Release URLは同repoのHTTPS tag参照に限定し、生の空白・制御文字、decode後のdot/dotdot経路、不正URLを拒否する。正当な階層tagとLF/CRLF/CR本文を受理する。同じCargoファイルが異なるcommitに現れる記録は正当であり、重複pathは一つのcommit内だけで拒否する。

これは純粋な構文検証である。作者のJSONは公開・実行・CI成功の証明ではなく、必須チェックを省略する根拠にしない。markerをIssue全体の強制受付契約として有効化していない。

## 実際の検証

- 初稿の不備を実assertで確認: 失敗exit/空白移行説明/危険pathを誤受理し、複数commitの同一pathを誤拒否した。`tmp/issue335-schema-main-behavioral-red-fields-2026-10-14.log` はexit1。
- URL正規化・traversal・不正URLの型付き拒否、repository dot component、CRLF/CR本文の不備も旧実装で実REDを回収した。`tmp/issue335-schema-main-url-expanded-red-2026-10-14.log`、`tmp/issue335-schema-main-repository-red-2026-10-14.log`、`tmp/issue335-schema-main-crlf-red-2026-10-14.log` はexit1。
- 修復後の単一discoveryは21 PASS/exit0: `tmp/issue335-schema-main-repaired-discovery-2026-10-14.log`。Python構文、preflightのzsh構文、diff checkもexit0。
- 深いJSONは旧parserでも今回環境で拒否できていた。RecursionErrorの型付き拒否は堅牢化であり、再現していない不具合のREDとは記載しない。
- readonly独立再レビューでP0/P1残0。全5filesは200行以下、関数30行以下。SemVerの計算量懸念は未実測P2として残し、性能改善済みとしない。
- 既存preflightへ純UT discoveryを一回だけ追加した。既存の重い検査・品質条件は変更していない。

## current review修復

public458fabcfのcurrent review5508460213のP2を採否判断して修復した。raw URLと一度decodeしたtagのbackslashを拒否する。同change内でrepository大小文字を同一視し、packageとversionはexactとしてtripletを一意にする。同tripletの完全重複・相反checksum・異URLは拒否するが、異versionと別changeでの再記録は許容する。Cargoには正当な複数版の共存があり、same repo/packageの全拒否は過剰制限として採用していない。

未変更の公開実装で6例の誤受理を実assert RED/exit1として確認した（`tmp/issue335-current-review-runtime-red-2026-10-14.md`）。修復後の既存単一discoveryは26 PASS/exit0（`tmp/issue335-current-review-green-2026-10-14.log`）。独立ASCII識別子文法との1365ケース比較、00a/01-/-の受理、a.01の拒否、4096文字不正末尾の拒否を含む。構文/diffチェックと独立readonly再reviewも成功し、P0/P1残0。

SemVerは最初の非数字より前を数字だけへ限定する同言語の2regex修復を行った。旧1024/2048/4096入力は約0.025/0.100/0.403秒、実修復sourceでは0.000054334/0.000096167/0.000198042秒で拒否した。実測環境の観測であり固定時間閾値をUTへ追加していない。意味論検査は旧regexでも成功するため、性能修復に虚偽のsemantic REDを付けていない。通常commit/pushと個別reply/resolve・fresh取得は別工程で回収する。

## 未完了

実Git commit/blob/差分との結合、上流Release・registry・checksumのライブ照合、検査ログの存在/hashと信頼された実行元の照合、既存hook先行dispatcher、pre-push range/有効化、公開後の安全cleanupは未完了である。Issue335を閉じない。通常commit/push後の実hook結果・current reviewは別途回収する。全coverage/配布版受入/公開完了も主張しない。

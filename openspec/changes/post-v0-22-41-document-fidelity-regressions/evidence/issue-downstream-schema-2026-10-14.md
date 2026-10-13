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

## 未完了

実Git commit/blob/差分との結合、上流Release・registry・checksumのライブ照合、検査ログの存在/hashと信頼された実行元の照合、既存hook先行dispatcher、pre-push range/有効化、公開後の安全cleanupは未完了である。Issue335を閉じない。通常commit/push後の実hook結果・current reviewは別途回収する。全coverage/配布版受入/公開完了も主張しない。

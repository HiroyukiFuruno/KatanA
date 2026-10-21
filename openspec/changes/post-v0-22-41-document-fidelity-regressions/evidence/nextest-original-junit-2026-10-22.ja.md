# 元Nextest実行のJUnit取得

## 実装範囲

Issue #343 のshadow4工程は、従来storeのhashだけを記録していた。
各元実行から公式JUnitを生成し、binary・test名・結果・run UUIDと件数を保存する。
通常test、coverageの選択、既存skip、retries=0、直列化、品質閾値は変更しない。
`execution_equivalent` / `reuse_authorized` / `report_checked` はfalseを維持する。

- 元repo configを工程別に複製し、絶対report pathとreport-skipped=allを追加する。
  実験的record設定のuser-configは従来のまま使用する。
- 新output内で工程別のXMLとconfigを保持し、既存ファイル・aliasへの上書きを拒否する。
- 一つのstore pathから得たrun UUIDとXMLを照合する。ZIP内部や再生表示は解析しない。
- UTF-8、DTD拒否、16MiB上限、regular file / no-follow、前後file identityを検査する。
- 重複suite/test、件数不一致、複数結果、retryを拒否する。失敗・error・skipは区別する。
- post-childの結果取得失敗は診断へ残し、元childの非0 exitを保存する。

## 対象検証

- main初回: 関連49件、2.170秒、exit 0（実行結果 ab88ba / d33ec7）。
- 新IO helperのproducer hash欠落: 追加回帰で実exit 1、KeyErrorを確認
  （実行結果19579f / 0533d6。ツール出力として回収、専用rawファイルは未保存）。
- helperの追跡を修正し、IO拒否回帰を追加: 52件、2.409秒、exit 0。
- 関数の分離とtest helper整理後: 52件、2.336秒、exit 0。
- exit 0と失敗XMLの矛盾: 追加回帰で実exit 1を確認（実行結果6bc3f6 / fea11e）。
  矛盾を取得失敗として記録する修復後、関連53件、2.532秒、exit 0。
  正本log: `tmp/oct22-junit-contract-final.log`。テストhelperの分離後も該当3件exit 0。
- 両lockと共有Git設定のSHAは従来値のまま。依存版・除外・hookは変更していない。

固定Nextest 0.9.148のuser-config実schemaにprofile設定はなく、JUnitは
[repo config](https://nexte.st/docs/machine-readable/junit/)へ置く。
同tagのCargo.lockにあるquick-junit 0.8.0の
[serializer](https://github.com/nextest-rs/quick-junit/blob/quick-junit-0.8.0/crates/quick-junit/src/serialize.rs)
に合わせ、root/suiteのskippedを検査する。旧2024年XML例のdisabledは使わない。

## 未完了の受入

既存の依存なし実Rust probeで元Nextest4工程からXMLが生成されることは未確認。
`tmp/oct22-junit-real-probe-execution.py` を準備済みだが、共有MacのKDV検査終了前に
起動しない。上限は全体120秒、専用target100MiB、jobs1とする。

この変更は実行結果の取得であり、通常libtestとの同等性、trusted reuse、
実際の検査削減、Windows cold/warm、製品full gate、packaged受入、公開を
達成したとは扱わない。これらは元DoDの未完了条件として保持する。

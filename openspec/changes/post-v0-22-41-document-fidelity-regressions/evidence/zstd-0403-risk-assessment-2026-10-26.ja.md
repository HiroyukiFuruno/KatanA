# RUSTSEC-2026-0403の利用経路と限定例外案

## 状態

**影響調査済み・例外追加は未承認。** 通常pushの実検査はこのadvisoryで失敗した。
既存のZIP例外は0367/0368/0369だけで、新規advisoryの除外を認めていない。
`deny.toml`と例外対象ID・期限は変更しておらず、検査の不合格を維持する。
既存3件の経路判定を厳密にする別修正は
[到達性の検証記録](zip-policy-reachability-2026-10-27.ja.md)で追跡する。

Windows比較診断の署名commit `fea653d02cd8dde03cb7059348e12042a5986025` はローカル
作成済みだが、通常push session53160は供給網検査でexit1となり、push成功ではない。
Windows元200ms不合格の修復・比較・全OS公開も別の未完工程である。

## 公式情報と実利用

[RustSec公式advisory](https://github.com/RustSec/advisory-db/blob/main/crates/zstd-safe/RUSTSEC-2026-0403.md)
は、古いZstandard frameの伸長失敗時に、書込式デコーダから未初期化メモリが
出力され得る問題を報告している。影響APIは`DCtx::decompress_stream`の特定の
使い方で、`zstd::stream::write::Decoder`が該当する。公式情報はread式Decoder、
decode_all、bulk APIを対象外としている。修正版の条件はzstd-safe 8.2.0以上。

通常lockと全featureの逆依存を確認した経路は次のとおり。

`KDV 0.5.12 → ironcalc 0.8.3 / office2pdf 0.8.1 → zip 0.6.6 →
zstd 0.11.2+zstd.1.5.2 → zstd-safe 5.0.2+zstd.1.5.2`

- zip0.6.6のread.rsは`stream::read::Decoder`をimportし、Zstd entryを読み出す。
- write.rsは`stream::write::Encoder`を使用する。Encoderと影響するDecoderは別API。
- 調査したzip/ironcalc/office2pdf/KDVとworkspaceのcallerで、古い
  `stream::write::Decoder`を使用する経路は見つからなかった。
- 別系統のzstd0.14.3はzstd-safe8.2.0で、修正版条件を満たす。

したがって、このadvisoryを直接発火させるcallerは調査範囲では確認できない。
影響版がlockに存在しないという意味でも、任意の入力・外部converterを含む
全プログラムが安全という証明でもない。KatanA側で推移依存のmajor版だけを
置き換えることや、上流を無断で修正することは行っていない。

現GitHub latestはv0.22.41で、そのtagのcommit
`af55949c4ee7ddad414ed93fa360a67e9464acf4`のCargo.lockにも同じ旧zstd-safe5.0.2が
含まれている。旧公開元にある別系統の7.2.4は、今回候補では8.2.0へ更新済み。
候補だけに旧5系を新規導入したわけではない。ただしこれは公開元lockの照合であり、
全配布binaryの逆解析や旧公開版の全利用API監査ではない。

## 固定した依存の同一性

| package | registry checksum |
| --- | --- |
| KDV0.5.12 | `4d39820039dee936b5ae1b7d9656751223421b5140c2b25f91d15ebe7519d635` |
| zip0.6.6 | `760394e246e4c28189f19d488c058bf16f564016aefac5d32bb1f3b51d5e9261` |
| zstd0.11.2+zstd.1.5.2 | `20cc960326ece64f010d2d2107537f26dc589a6573a316bd5b1dba685fa5fde4` |
| zstd-safe5.0.2+zstd.1.5.2 | `1d2a5585e04f9eea4b2a3d1eca508c4dee9592a89ef6f450c11719da0726f4db` |

根拠は現Cargo.lockと取得済みregistry source。API箇所と全packageの照合結果は
`tmp/oct26-zstd0403-impact.md/json`、検出rawは
`tmp/oct26-fea653d0-normal-push.log`へ保持した。

## 承認を求める案

1. 追加するIDは**RUSTSEC-2026-0403だけ**。他の新規advisoryやライセンスは除外しない。
2. 期限は既存と同じ**2026-11-07 00:00 UTC（日本時間09:00）**。延長しない。
3. 上記の固定registry版と、KDV配下の監査した旧ZIP経路だけを許容する。
   別caller・別版・KDVを迂回する経路は拒否し、再評価する。
4. 正式な供給網入口で3manifestのlocked全feature graphと期限を検査した場合だけ、
   一時cargo-deny設定に1件を追加する。Git管理のdeny.tomlを恒久緩和しない。
5. 期限到来、利用API/依存経路の変化、修正版採用、新しい悪用証拠の発見で再評価し、
   自動延長しない。既存ZIP3件の範囲・期限を拡大しない。
6. 期限・未知ID・別caller・別版・欠損graphを拒否する自動回帰と実3manifest検査、
   通常review/CIを通す。Windows起動を含む他の必須検査は維持する。

承認がない間はこの案を本番policyへ有効化しない。承認判断そのものをIssueの
Close条件へ追加しない。例外の実装・自動検証・公開後の解除/再評価を追跡する。

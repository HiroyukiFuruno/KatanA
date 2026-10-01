# sample.md 実ホスト文字座標（2026-09-10）

本文1187×2225 logical point、font14、light、scroll0、倍率2で採取成功。再利用可能な `record_typography` ステップが本文寸法とスクロールを検証する。初回1280×2400ウィンドウでは本文1184.5×2238となり検査失敗。ウィンドウを1282.5×2387 logicalへ調整し、指定本文寸法で成功した。参照画像や95点閾値は変更していない。

## 来歴と限界

`sample-typography-metrics.json` に抽出結果と来歴を保存した。KatanA実装worktreeのコピーにKDV候補commit `3b4ecb13396c8f067e838cacb2a13087dd88309d` を診断目的で接続。KRR0.4.19、KUC0.3.7、V8 152.2.0はregistry版。正式worktreeのKDV依存は0.5.5のまま変更していない。公開依存でのリリース受入検証ではない。

入力は `assets/fixtures/sample.md`。KDVの `assets/fixtures/katana/sample.md` とSHA-256が一致する。指定されていたroot直下のsample.mdは現在存在しない。

## 計測値

画面原点のlogical point。本文原点=(44,134)。Labelのaccessibility boundsとpaint galley boundsの一致をJSON検査で確認した。

| 対象 | source line | font size | label y | label height | row top→font baseline | 次のlabel y |
|---|---:|---:|---:|---:|---:|---:|
| H1 Heading | 91 | 21 | 1608 | 31.5 | 18.5 | 1660.5 |
| H2 Heading | 93 | 19.845 | 1660.5 | 30 | 17.5 | 1711.5 |
| H3 Heading | 95 | 18.676 | 1711.5 | 28 | 16.5 | 1760.5 |

各label末尾から次のlabel上端までは21。これをKDVのblock outer heightと同一視しない。`heading_hit` は合成アンカーであり、label boundsも周辺のMarkdown空白を含むblock外形ではない。glyph bandはクリップ前のmesh quad範囲で、ラスタink領域ではない。タイトルは絵文字と複数のTextShapeに分割されるため、全文字列一個のgalleyとして扱わない。

## 検証

- 正式worktree: screenshot harness全target Clippy `-D warnings` 成功、cargo check成功、diff check成功。
- `just test-paint-metrics`: 実egui文字フレームの2テスト成功。
- 診断コピー: screenshot harness全32テスト成功、実ホスト計測ステップ成功。
- 計測中に既存 `click_node` の座標二重除算を発見。実button入力テストで修正前失敗（中心220,116が110,58へ変換される）、修正後は倍率1と2で入力成功。
- 最終採取JSONで本文1187×2225/font14/scroll0と、H1〜H4のaccessibility label boundsとgalley boundsの一致を検証。

KDV担当へ数値と来歴を引き渡した。KDV側の修正と95点ゲート、公開registry採用後のKatanA全受入・リリースは未完了。計測単体の成功をリリース完了扱いしない。

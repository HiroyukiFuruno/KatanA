# 追加の共通ライブラリ更新

## 更新内容と公開確認

最新の本人指示に沿い、KatanAが利用する公開registry版のパッチ更新を採用する。KDV `=0.5.12` とKRR `=0.4.23`、未公開依存を残さない条件は維持する。

| 依存 | 更新 | 公開版・VCS |
| --- | --- | --- |
| katana-markdown-model | 0.2.3 → 0.2.4 | [v0.2.4](https://github.com/HiroyukiFuruno/katana-markdown-model/releases/tag/v0.2.4)、`d5076b65935595274b95afa9365295b3fd9b987f` |
| katana-ast-lint | 0.5.2 → 0.5.3 | [v0.5.3](https://github.com/HiroyukiFuruno/katana-ast-lint/releases/tag/v0.5.3)、`01df506648f512319fcf97a210a6107dccc35a69` |
| katana-markdown-linter | 0.19.5 → 0.19.6 | [v0.19.6](https://github.com/HiroyukiFuruno/katana-markdown-linter/releases/tag/v0.19.6)、`c325648464a5afab2a3435966c32a1394aa0b0ef` |

crates.io APIのversionが存在し、3件ともyanked=false。公開checksum・両lockのchecksum・取得済みcrate archive SHA-256が一致し、archiveの `.cargo_vcs_info.json` と各公開tagのcommitも一致した。追加差分の元は4ファイルのみで、06:32 UTCの変更時刻は同日06:35 UTCのキャッシュ整理より前だった。変更者は未確定だが、本人の依存更新指示を根拠として差分を破棄せず監査・統合する。

## 互換性

KMMとKALにはRustソース差分がなく、依存更新が中心。KatanAが利用する公開APIのsignature差はない。KMLのoptional MCP featureの更新は、KatanAで有効化されていない。

KMLのMD043では、見出し構成が未指定またはnullなら従来どおり無効だが、明示的な `headings: []` は見出しを許可しない設定になる。リポジトリ内のKatanA設定に該当はなく、利用者のカスタム設定では診断結果が変わり得る。この変更は日英CHANGELOGへ記録する。

## 実検証と境界

- rootとscreenshotの `cargo metadata --locked --offline --all-features`: 成功、検査前後の両lock hash不変。
- 既存の `just type-check`: exit0、8 cratesを確認、19.25秒。`tmp/oct27-late-library-type-check.log`。
- 詳細な差分hash、checksum、取得記録: `tmp/oct27-late-katana-library-refresh-audit.md` と同名JSON。
- 元の4ファイルhash/mtime: `tmp/oct27-late-library-refresh-observed.json`。

旧HEAD `1177b30c600bc30f4826469f8ae2ae9ec0d43a7b` のCI run40455468177は、この追加更新前のgraphを検査している。その成功を更新後graphの成功へ流用せず、通常統合後にcurrent review・必須CI・供給網・配布検査と公開を回収する。型検査とmetadataだけで公開完了とは判定しない。

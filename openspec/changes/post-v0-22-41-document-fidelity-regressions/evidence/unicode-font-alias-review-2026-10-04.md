# Unicode書体aliasの追加P2

PR346 current761439e review5406447976、comment4177866298/threadPRRT_kwDORm09y86oy_ZO。

## 再現と修正

既存embedded Ubuntu SFNTの全Unicode family/typographic recordを元UTF16長のままÉcoleへ置換（余りを空白、canonicalは既存trimでÉcole）。要求はécole一件のみ。旧ASCII comparatorで実resolverが `font face unavailable: family="école"` と診断し、0pass/1fail/exit101を確認。ログ `tmp/unicode-font-alias-real-red-2026-10-04.log`、RTK full log1791121198。

最初の誤filterは0件実行、先行fixtureは異なるname record長24/12で停止したため、いずれも製品REDの根拠に使わない。重複normalized keyのÉcole/école二件を二faceと期待する案も既存dedup仕様に反するため採用しない。

FaceMetadataでraw canonicalを保持したままaliasを一度Unicode `to_lowercase()` 正規化。requestも同じ処理を使い、既存request_keyと統一する。重複alias排除も同戦略でÉcole/éCOLEを一件へまとめる。weight/style/payload抽出/共有・API・依存・レンダリング規則を変更しない。

全書体26件は修正後GREEN。最初のfmt-checkで改行を指摘され公式formatterで対処。ASTでは新実SFNT fixtureが既存test fileを319行にした責務境界違反を検出したため、name-table生成責務をtest-only sfnt_fixture.rsへ分離（203行）、assertionはmetadata_alias_tests.rs（115行）に残す。上限・lint無効化は行わない。分離後のformat/書体全26/AST23/全target Clippyと全coverageは再実行してから完了判定する。

正式commit/push・個別reply/resolve・fresh review・変更後実受入はまだ未完了。前のOffice/反復/canonical受入は435ac5b2のsourceとfresh runnerに対する証跡で、新Unicode修正後の受入へ流用しない。

分離後もfixture helperは200行規則で207行を指摘された。共通name-table layout/record readerの重複責務を統合し、公式formatter後198行へ修正。最終書体26件、AST23件、workspace全target strict Clippyは全exit0。ログ `tmp/unicode-font-alias-final-green-2026-10-04.log`、`tmp/unicode-font-alias-ast-final2-2026-10-04.log`、`tmp/unicode-font-alias-all-target-clippy-2026-10-04.log`。rule/閾値/skip変更なし。main自己レビューでprivate caller、request key正規化との一致、raw canonical保持、style/payload不変と実SFNT診断への到達を確認。全coverage・通常pushは次工程として残す。

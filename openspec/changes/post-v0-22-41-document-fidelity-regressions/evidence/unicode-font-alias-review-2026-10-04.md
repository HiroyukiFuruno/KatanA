# Unicode書体aliasの追加P2

PR346 current761439e review5406447976、comment4177866298/threadPRRT_kwDORm09y86oy_ZO。

## 再現と修正

既存embedded Ubuntu SFNTの全Unicode family/typographic recordを元UTF16長のままÉcoleへ置換（余りを空白、canonicalは既存trimでÉcole）。要求はécole一件のみ。旧ASCII comparatorで実resolverが `font face unavailable: family="école"` と診断し、0pass/1fail/exit101を確認。ログ `tmp/unicode-font-alias-real-red-2026-10-04.log`、RTK full log1791121198。

最初の誤filterは0件実行、先行fixtureは異なるname record長24/12で停止したため、いずれも製品REDの根拠に使わない。重複normalized keyのÉcole/école二件を二faceと期待する案も既存dedup仕様に反するため採用しない。

FaceMetadataでraw canonicalを保持したままaliasを一度Unicode `to_lowercase()` 正規化。requestも同じ処理を使い、既存request_keyと統一する。重複alias排除も同戦略でÉcole/éCOLEを一件へまとめる。weight/style/payload抽出/共有・API・依存・レンダリング規則を変更しない。

全書体26件は修正後GREEN。最初のfmt-checkで改行を指摘され公式formatterで対処。ASTでは新実SFNT fixtureが既存test fileを319行にした責務境界違反を検出したため、name-table生成責務をtest-only sfnt_fixture.rsへ分離（203行）、assertionはmetadata_alias_tests.rs（115行）に残す。上限・lint無効化は行わない。分離後のformat/書体全26/AST23/全target Clippyと全coverageは再実行してから完了判定する。

正式commit/push・個別reply/resolve・fresh review・変更後実受入はまだ未完了。前のOffice/反復/canonical受入は435ac5b2のsourceとfresh runnerに対する証跡で、新Unicode修正後の受入へ流用しない。

分離後もfixture helperは200行規則で207行を指摘された。共通name-table layout/record readerの重複責務を統合し、公式formatter後198行へ修正。最終書体26件、AST23件、workspace全target strict Clippyは全exit0。ログ `tmp/unicode-font-alias-final-green-2026-10-04.log`、`tmp/unicode-font-alias-ast-final2-2026-10-04.log`、`tmp/unicode-font-alias-all-target-clippy-2026-10-04.log`。rule/閾値/skip変更なし。main自己レビューでprivate caller、request key正規化との一致、raw canonical保持、style/payload不変と実SFNT診断への到達を確認。全coverage・通常pushは次工程として残す。

source16d70b90/証跡8f989151を署名Gで正式履歴へ保存。使用中Cargo lockなし、clean checkpoint後にこのworktreeの再生成可能なkatana-ui dev生成物だけを公式Cargo dry-run（4.2GiB）→clean（2626files/3.6GiB）で整理。source/history/私有原本/release runnerとworker/sibling生成物は不変、stash0/master cleanを確認。

変更後の公式just coverageはexit0。UI1040/既存ignore2、main17、実export13（250.98s）、parallel143/既存ignore2（23.76s）、serial18（2.76s）、meaningful未実行0、strict document surface100%/uncovered0。ログ `tmp/unicode-font-alias-full-coverage-2026-10-04.log`。JSONを別途保存し、旧435/f8 coverageを流用しない。通常push/個別replyresolve/current rereview/新source実受入/配布公開は未完了。

ライブregistry APIはHTTP403、webページも取得不能だったため、公開sparse indexで確認。KDV0.5.11 nonyanked/checksum865b7ed6...、KRR依存^0.4.22/office2pdf=0.8.1、registry null（crates.io）を確認。KRR0.4.22 nonyanked、0.4.23 entryなし。GitHub latestも0.4.22のままで、未公開版の採用や上流Issue95 closureを先取りしない。

e554497bまで通常push exit0、GitHub PR346 HEAD一致・upstream差0/0、Draft維持を確認。native/Linux/Windows/preflightを省略せず通過し、current review5980969666を依頼。変更後coverage JSON保存・署名済clean checkpoint・active Cargoなしを確認してから、このworktreeのcoverage katana-ui dev生成物だけを公式dry-run後にclean（1405files/3.0GiB、再生成可能）。source/release/私有原本/siblingは不変。ログtmp/unicode-coverage-ui-clean-2026-10-04.log。

mainのconsumer追跡で追加欠落を確認。cell request投影はASCII lowercase、resolver keyはUnicode lowercase、lease family_forはASCII一致のため、同document内のÉcole/école混在時はresolver dedupで残らなかった表記のlookupがNoneとなる。metadata単独の実SFNT回帰ではこの後段をカバーしない。実cell投影・実Ubuntu payload leaseの回帰RED→最小修正→GREENを継続し、当該threadはまだreply/resolveしない。KRR PR105は最新live HEAD73773067でDraft、latest公開版0.4.22のまま。

実consumer回帰は旧製品で4件中2件FAIL/2件PASS/exit101。cell投影request2vs1とlease family_for Noneを再現（tmp/unicode-lease-consumer-real-red-2026-10-04.log）。最初の実行はstrip=none指定漏れでprofiling proc macro compile失敗となり、製品REDへ流用しない。投影とlookupの2箇所をUnicode lowercaseへ統一後、同4件GREEN/exit0。後からalias同一性/regular-bold区別assertを補強し、全font_loader/font_requests/AST/format/impacted lintを既存justターゲットで再実行中。

same_faceのASCII比較も調べたが、same_sharedがepochを保持するためcasing変更epoch回帰は旧製品でもPASS。この実証範囲では不要な再構築が外部可観測な不具合とは断定できず、今回の修正にdefinitions変更は含めない。追加sourceの全coverage/正式push/current review/原本・配布受入は未完了、前source成功で置き換えない。

補強後の既存justターゲットはfont_loader67件・font_requests7件・AST23件すべてexit0。追加alias同一性assertの改行をfmtが検出したため公式just fmt後にfmt-check再実行成功、lint-impactedもexit0。self-reviewで実caller→投影→resolver→lease→painter経路、alias/style同一性と変更後の行数規則を確認。製品変更は投影1箇所・lease lookup1箇所に限定。残る正式push/変更後全coverage/current review/原本・全配布受入は継続。

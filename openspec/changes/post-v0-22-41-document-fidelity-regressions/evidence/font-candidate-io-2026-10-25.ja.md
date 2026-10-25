# 字体候補の不要な全量読込

## 原因の切り分け

`b0ba969b` のARM診断実行 `tmp/macos-arm64-diagnostic-qa.arvEt8` では、候補フォントを1579回、計2,143,697,148bytes読み込んでいた。最初の2回のlookupは各437候補・600,141,688bytesを読み、存在しない字体の診断も記録した。残り2回の走査打切りは取消経路と整合するが、見つからなかったと断定しない。

KatanAのresolverはfamily/styleを照合する前に、候補を最大32MiBのVecへ全量読込していた。platformのキャッシュは候補path一覧であり、字体metadataのキャッシュではない。別の文書・generationで同じ候補を再び走査する。字体選択に不要なglyph/outlineも読むため、これはKatanA所有の不要IO・一時allocationである。

一方、解放後のRSS増加だけをリークと断定しない。独立した小さなCプロセスでも、macOS allocatorは大きな領域をfree後に保持し、次の同サイズallocationで再利用した。`malloc_zone_pressure_relief(NULL, 0)` のgoalはSDK契約上正しく、複数goalの対照でも返却0だった。新allocator・返却goal変更・閾値緩和で検証を通さない。

## metadataの意味一致対照

現行skrifa 0.49.0のmatcherが参照するのは `name`・`OS/2`・`head`・`post`。同じ437ファイルからこれらだけをseek/readした独立対照は、全827字体について全量版とlocalized family・優先名・attributes・style flags・post属性が一致し、0失敗・実終了0だった。

診断と同じ走査回数へ換算したmetadata読込量は99,121,620bytesで、候補全量読込より約95.4%少ない見込み。ただし採用フォントの全量読込は別途残るため、この値を製品のIO・メモリ・起動時間の改善実測として扱わない。対照最大physicalは118,391,240bytesだった。

## 修復方針と残る検証

候補ファイルは既存の上限・取消検査で開き、同じhandleから必要metadataを境界検査付きで読み、既存matcherで採用可能な候補だけ全量読込する。採用時は実payloadを再解析し、古いmetadataだけで採用しない。候補順・同距離の優先・次候補fallback・TTC face・不正入力の診断を維持する。公開API・依存・unsafe・永続font cacheは追加しない。

修復後の実module回帰、正式AST/Clippy/fmt、独立差分レビュー、通常commit/push、新配布版の反復描画・close・RSS/physical予算・通常終了の回収を継続する。本証跡は原因と対照の記録であり、性能合格・独立95点・公開完了の証明ではない。

## 実装後の対象検証

metadata読取と採用候補だけのpayload読取を実装した。正式 `just JOBS=1 T=font_loader::office_faces test-specific` は43件成功・実終了0。正式AST30件、既存の対象Clippy、fmt、差分検査も成功した。独立harness62件は製品module61件と437候補miss対照1件の集計であり、正式Cargo43件へ加算して重複した成功件数を作らない。437ファイル・827字体のmiss対照では全量payloadのready読込は0回だった。

不採用の大payload非読込、localized metadata、TTCの一部破損と拡張version、採用失敗後の次候補、同じopen handleでのpath置換、実payload変更後の再照合、途中取消を検査した。独立レビューにP0/P1はなく、破損診断の互換性所見は実parser対照で修復した。不正sfnt versionは従来の `InvalidFontFile`、正しいheaderだけでdirectoryが欠ける場合は既存parserどおり空metadataから通常のunavailable、TTC内の不正faceは `InvalidFontFace` を維持する。truncatedも一律InvalidFontFileになるという途中の仮定は、実対照が否定したため採用していない。

正式対象ログは `tmp/oct25-font-metadata-main-targeted-tests.log`、ASTは `tmp/oct25-relief-probe.font-fix-ast.log`、独立harness・対象Clippy/fmtは `tmp/oct25-relief-probe.font-fix.report.ja.md` で追跡する。対象検証の成功だけで配布版のcold/steady予算を完了にせず、新packageで測定する。

ローカル診断原本は `tmp/oct25-relief-probe.font-audit.ja.md`、`.font-audit.json`、`.font-metadata.log`、allocator対照は `tmp/oct25-relief-probe.report.ja.md`。私的文書の内容や撮影画像は公開しない。

# TTC face payloadの複製修正

対象はPR346のP2 comment4173709245。公開・全体受入とは別のKatanA局所修正。

## 原因と設計

- 旧append_matchesは複数faceにcollection全Vecをcloneし、face indexが異なるため既存lease共有でも統合できなかった。
- egui0.36.2のFontData.fontはCow<'static, [u8]>で、所有バイトのArc共有には対応しない。unsafe/lifetime偽装/static leakは使用しない。
- 公開write-fonts0.54.0のraw FontBuilderだけを利用し、選択faceの全tableを単独SFNTへ再構築する。字形・GSUB/GPOS・未知tableも削らない。header offsetsとhead.checkSumAdjustmentはbuilderが再計算する。
- ResolvedFontFace.face_indexは元collection indexを維持し、FontData.indexは単独fontの0。通常TTF/OTFは元allocationをmoveし、コピーを増やさない。
- SFNTの検索範囲u16を表現できないtable数、重複tag、壊れたtable範囲、短いhead、header/padding込み32MiB超過をbuild前に拒否する。InvalidFontFaceとsearch_incompleteを伝播する。
- 新source WHYはこの証跡へ記録。最新の日本語コメント規約と既存no-japanese/comment-style gateが衝突するため、コードコメントを追加せず、gateや除外を変更しない。

## TDDと検証

- RED: 実インストールregular/bold TTFのbytesから有効な2-face TTCを作成。旧コードでpayload先頭ttcfの非一致assertionが失敗。
- GREEN: focused Office font23件pass。全table byte一致（head checksum adjustmentのみ除外）、元index0/1、glyph Aのadvance/outline、style、Weak解放、破損face診断を検証。
- 実保持量: source1524240bytes、2face旧3048480bytes→抽出1524220bytes。約50%削減。アプリ全体RSSの半減を意味しない。
- CFF2/GSUB table保持、padding/overflow/巨大重複範囲、4096table拒否、単独font allocation moveの単体回帰を追加。
- format/impacted Clippy/AST23件pass。最初のAST失敗はコメント形式とmagic numbersで、意味付き定数へ修正しgateを維持。
- 独立read-onlyレビューでP0/P1なし。全table維持・index分離・builderの算術境界を親も確認。
- write-fonts0.54.0は最新公開版、default-features=false。既存read-fonts0.45/font-types0.12.6を再利用し、両lock差分は追加crateとwrite-fonts既存版の名称分離のみ。
- 追加all-target Clippyは共有host空き1.4GiBへの低下を確認してowned commandをSIGINT終了（exit130）。成功として扱わない。再実行・通常push/full gate・新sourceの配布/実原本受入・reply/resolve/current reviewは未完了。

## 残DoD

現sourceの全target lint/test/coverage/supply-chain/packaging、正式履歴/通常push、comment4173709245へのreply/resolveと全review再取得。HTML正常close・Office fidelity・全OS clean-machine受入・公開は引き続き未完了。

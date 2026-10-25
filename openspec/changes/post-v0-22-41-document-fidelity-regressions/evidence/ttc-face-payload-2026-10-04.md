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

- 4aee907eの通常pushはexit0。pre-pushのjust check（native通常test、Linux実worker付きworkspace、Windows test-inclusive cross-check）を通過した。追加supply-chainもadvisories/bans/licenses/sourcesが全て成功。
- 別途all-target Clippyを再実行し、test fixtureの剰余判定にmanual_is_multiple_ofを1件検出（exit101）。標準is_multiple_ofへ置換後、全target Clippy・format・focused23件がexit0。focused linkでは既存debug __eh_frame警告が出たため、配布版の検証を代替する結果とはしない。失敗・中断を成功として扱わない。
- 補正8a3ab0afは署名Gで正式commit、通常pushもexit0。native通常test、Linux実worker付き全workspace、Windows test-inclusive cross-check、native fixture8件、全coverage（UI1030/既存ignore2・parallel143/既存ignore2・serial18、strict document surface100%/未実行0）が成功。
- raw coverageの新face_payload.rsは40/40行・4/4関数。regionは105/106で、64bit対象で到達しないusize→u64変換失敗closureは実行済みと主張しない。既存除外・閾値は変更していない。
- comment4173709245へreply4174093967後、当該threadを個別resolve。全review/commentsを再取得してresolvedを確認し、残る未解決はmanual target policyのみ。8a3ab0afのcurrent再reviewはissuecomment5971694812で依頼、結果未確認。
- 8a3ab0afのmacOS universal main/workerを再生成し、両architectureとstrict ad-hoc codesign検証に成功。main SHA256は561c26f5b4b07070382231a01b75e18836e5a6e583abf68297da0192f52eed4d、workerはba2d89aa47f7188b04f0b466a908471d099c5018ba2484f242147a816b380650。
- fresh configのarm64 startup smokeは実kernel image/SHA一致・継続UI・peak RSS242512KiB・font27093388bytes・Office worker0でexit0。証跡tmp/trash/2026-10-04-023302-startup.M7prNf。Termsは自動承認せず、原本Office/HTMLのclean-machine受入や全OS実機成功とは区別する。

current HEADのcloud/review結果、HTML正常close・Office fidelity・全OS clean-machine原本受入・全配布成果物・公開は引き続き未完了。KRR公開修正版→既存KDV担当の公開版→KatanA固定registry採用の順序を維持する。

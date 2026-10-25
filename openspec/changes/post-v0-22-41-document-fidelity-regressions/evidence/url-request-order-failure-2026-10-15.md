# 非同期URL要求の順序・失敗先の修復候補

対象: [Issue351](https://github.com/HiroyukiFuruno/KatanA/issues/351)

## 証跡と修復範囲

- 既存review5526397875の2件を採用。別targetのready応答をpending応答で止めず、同targetの順序を維持する。queueをstate内に保持し、形式移行時の後続target更新を受け取る。
- エラーはmutable URL入力ではなく要求のsource URLとtargetへ結び付ける。閉じたtargetは順序判定より先に破棄する。未知形式のHTML診断をbinary文書とは表示しない。
- 追加redirect回帰10947はactual exit101、6件成功・2件assert失敗。alias要求が既存targetと合流する順序違反と、初回PDF要求が拡張子無しloginへredirectして403になった際の診断消失を再現。生ログ: `/Users/hiroyuki_furuno/Library/Application Support/rtk/tee/1792026612_cargo_test.log`。
- 未確定targetはredirectで既存文書へ合流し得るため順序を維持し、確定した異なるtarget同士は独立して処理する。診断identityは要求URLを保持し、最終redirect URLはエラーcauseに残す。
- 修復後46988 actual exit0、URL関連35件成功・41 suites。実HTTPの既存redirect/resource/連続OpenUrl回帰を含む。追加channel回帰はunit検証であり、native GUIや実HTTPで追加境界を証明したとは扱わない。
- 82260正式ASTは22成功・1失敗。テスト内30秒のmagic numberを既存URL_SOURCE_TIMEOUTへ揃えた。66814再検査・61562impacted lintの終了回収と最終source再検査が必要。
- 66814正式ASTはactual0/23件、61562impacted lintはactual0。続くreload回帰追加後の28242はmagic HTTP status/受信buffer値で22成功1失敗となり、最終修復・回収を継続する。旧合格は新sourceへ流用しない。
- 独立reviewで同source・異targetの形式合流が先行要求を追い越すP2を確認。61124 actual101/queue1対期待2のassert RED→同sourceのFIFOを追加。collision時は既存元文書を保持する契約があり、文書数1のfixture誤期待は2へ訂正した。削除契約を製品へ追加していない。
- 初回403の仮想PDFは手動reloadで再取得しないことを29382 actual101で再現し、fetch失敗の要求URLを成功済みURLより優先する入口を追加。最終50733 actual0/実TCP1件成功、50538 actual0/URL全37件2.54秒。実代表PDFをHTTP200で受信しURL登録と失敗表示解除、要求先の一致を検証した。KDV Ready/frame・GUI描画は未検証。
- 初期HTTPfixtureのnonblocking accepted socket/403 early-abort本文送信は実WouldBlock/BrokenPipeで失敗し、accepted socket blocking＋元2秒read期限、403本文空へ修正。PDF URLにHTMLを返す候補は既存認証ページ拒否により正しく失敗したため、製品判定を緩めず代表PDFへ変更した。
- 77394 UI all-targets・external-fixture-acceptanceのstrict Clippy actual0。workerによるテストhelper分離後は最終再検査を回収する。
- 最終helper整理後35167/21872/81322はPathBuf参照比較のcompile errorでactual101となり、mainがPath参照へ修復した。最終63212 actual0/URL37件2.76秒、12274 actual0/既存失敗表示2件、13532 UI all-targets・external feature strict Clippy actual0、58422正式AST actual0/23件13.22秒、94716fmt actual0、diff check成功を回収。KML bare URLをリンクへ修復し対象2文書成功。失敗結果を合格へ上書きしない。

## 残作業

sourceは通常commit96026 actual0、`0fc9b15e`へ統合。通常hookを迂回していない。公開upload/current reviewは別未完。

- 最終sourceの対象検証、正式AST・strict lint・自己レビュー、関心別通常commitと通常push一回。
- review本文への証跡返信とcurrent review全取得。今回URL指摘にはinline threadが無く、架空threadをresolveしない。
- 失敗文書のreload導線を確認する。既存本文の保存、同target順序、形式移行、cancel、期限の契約を維持する。
- 異source URLのredirectが同じbinary targetへ合流する順序はまだ未検証。同source衝突の限定回帰をこの境界の証明へ代用せず、Issue351の残調査として保持する。
- current HEAD全gate、Ready CI、packaged/全配布/公開後処理は別未完。旧HEADの合格や限定35件を全releaseの完了へ流用しない。

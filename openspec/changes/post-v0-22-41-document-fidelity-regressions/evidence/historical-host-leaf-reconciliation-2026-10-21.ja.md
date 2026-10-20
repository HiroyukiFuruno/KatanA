# 古いhost指摘の状態照合

## 対応済みと確認できた範囲

修復が現HEADと公開PR HEADの祖先にあり、元の検査結果とレビュー返信を照合できた3件だけを未対応から対応済みの`[/]`へ訂正する。DoDや元指摘本文は削除しない。

- 大型画像の表示所有: `efe2ebf0`でreuse cacheと表示中textureを分離。元thread `PRRT_kwDORm09y86pR5n-`への返信は`4191207109`。先行17textureの訪問前に18枚目が届く旧経路のRED、同TextureId保持・grace frame・非表示後解放・resetを記録している。小画像17枚の後続修復`3b2a0341`/`d39245dd`と返信`4191620938`は別指摘であり、元threadの返信と取り違えない。1pixelによる構造的所有回帰を17大型画像の実GPU総量の測定とは扱わない。
- PPTX二重Resize: `371aeb67`でmodal中のPPTX背面描画だけを止め、Markdownの背景pollは維持。thread `PRRT_kwDORm09y86pQ8GT`、返信`4190466526`。大型PNGの互換退行は`e8e98caa`で入力拒否とreuse上限を分離し、thread `PRRT_kwDORm09y86pQ8GX`、返信`4190466916`。
- PDF Tools: `bd652bfb`/`c520076f`でToolsをMarkdown専用Export/Storyから分離。旧実装のREDと公式5件PASSを`host-feedback-integration-2026-10-06.md`に保存済み。元指摘は通常push失敗であり、存在しない専用review threadの返信を新たな条件にはしない。

主査は上記3threadを個別GraphQLで再取得し、全件`isResolved=true`、各commentsの`hasNextPage=false`を確認した。PDF/PPTX/画像の修復commitが公開`origin/release/v0.22.42`の祖先であることも確認した。

## 実行証跡の範囲

同HEAD fa9の既存normal全実行は終了0。raw `tmp/oct21-normal-fa9-canonical.log`には、PDF Toolsのdirect dispatch/command inventory、PPTX背面抑止とMarkdown背景poll、small display texture所有の5件が名前付きで成功している。旧検査の実行結果を現在HEAD全体の合格へ転用せず、対象指摘の修復履歴と最終gateを分離する。

## 未対応を維持する項目

本人のnative reload無反応は未解決。sidebar pointer→action→local HTML新session/RGB、changed/force/dirty、URL refreshは個別回帰で確認できるが、Cocoa実menu eventの受信やpackaged動作の証明ではない。virtual文書のmanual refreshは意図した図の再描画であり、外部HTML再読込とは異なる。新しい製品不具合を証明せずに再修正せず、正式native/packaged受入へ残す。

GPU総量・原本性能・全OS・新HEAD全gate・公開は未完了のまま。今回の変更は状態訂正だけで、品質条件・テスト・実装コードは変更しない。

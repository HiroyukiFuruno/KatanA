# HTML受入記録と実描画の結合

## 原因と修復

実操作記録を照合したところ、旧producerはHTML reloadの `frame_session` を実receiptへ照合していなかった。packet内の識別値だけが整合していれば、欠測・ゼロ・古いframeの値を見逃すP1である。初回表示のcaseにも同じ関連づけの欠落があった。

KatanAがframeを受理した地点で、そのframeへKatanA内のrenderer epochを保持する。receiptは保持した値を読み取り、現在のsurface値から後付けしない。これは上流SDKのsession IDではない。producerは初回表示のframe/current epoch一致と、reload前後両方の実receipt/packet一致を要求する。既存のURI・adapter・source・描画・idle・操作・終了・メモリ条件は維持する。

frame構築は既存の専用moduleへ集約し、既存KDVのviewport型と所有値だけを渡す。公開API・新依存・制御port・原本への書込・閾値緩和は追加していない。

## 検証

- 修復前の実producerへ新しい単体回帰を適用した5件は10ケースの見逃しで終了1。入力は明示した単体テスト専用合成データであり、実配布受入へ使用しない。
- 修復後のhost契約34件は終了0。欠測・ゼロ・不一致・自己整合packetによる古い実frameのすり替えを拒否する。
- 公式 `test-specific` の関連7件、42suite・2241件除外は終了0。実browserの受理frameとreloadを使い、surface値だけを後から変更しても保存済みframe値が変わらないことを確認した。
- 公式AST30件・影響範囲Clippy全4crate・fmtは終了0。初回ASTのコメント書式と201行違反、途中の直接KRR型参照によるcompile失敗101は修復後の成功と区別する。
- 他作者の独立レビューで必須指摘なし。主査が最終8ファイルのSHAを照合した。実配布版の描画・操作・終了の証明ではない。

## 通常pushで回収した旧fixtureの不整合

`b5c7891` の通常pushは601件が成功した後、旧形式の比較証跡を新host入口へ渡す1件で失敗した。旧証跡で新入口を認可する期待を削除し、旧host-action検証とstrict比較の失敗を別々に検証する。隣接する負例は、正常baselineが同じvalidatorを通る確認後、各破損に固有のエラーを照合する。対象10件は終了0であり、元の公開条件を緩めていない。

## 正式統合

frame結合8ファイルは署名付き通常commit `ec60264b`、旧形式fixture修復は別の署名付き通常commit `d484630` へ統合した。追加でstrict受入55件と公開gate32件も終了0。未公開commitの分離時に一度混在したため、作業内容を保持してamendで8ファイルへ戻し、別commit化した。force push・stash・hook迂回は行っていない。

## 未完了

最終sourceの全構成配布生成と実文書受入、current review/Ready後の必須CI、全5assetの公開とchecksum/公開後Mac受入は未完了。旧配布の大型XLSX steadyとPPTX coldのメモリ未達を解消・免除した変更ではない。本人への限定例外の質問は未回答で、元基準を保持する。

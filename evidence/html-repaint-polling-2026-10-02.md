# HTML画面更新監視の独立回帰

## 原因と変更範囲

KatanAのHTML surfaceは、更新取得後に2秒の期限内だけ16ms後のrepaintを予約していた。
公開KDVのbrowser adapterは更新をCondvarへ通知するが、eguiへのrepaint通知は持たない。
そのため期限より長い処理中にUIがidleへ入ると、完了frameを取得するための描画が予約されない。
これはKRRの実layout遅延とは別のKatanA側の更新経路である。

既存2秒windowは変更せず、実adapterが非idleの間は16ms後のrepaintを予約する。
更新取得とidle観測の間の公開を取りこぼさないよう、idle切替後に最終取得を1回予約する。
以後は停止し、0msの同期busy loopにはしない。

## 決定的な回帰

商用の`image_html_surface_polling.rs`を直接コンパイルした3件で、旧ポリシーは
すべて`None`と`Some(16ms)`の不一致により失敗し、変更後は3件成功した。
対象は期限超過中のbusy、deadline無しからのbusy→idle、期限内と期限到達後の最終取得。
固定sleep、renderer mock、受入期限の延長は使用していない。

この軽量検査は全UI build/test/coverageや実egui・実adapterの受入を代替しない。
locked release UI unit testも3件成功（1000件filtered、exit0）。既存の実browser
adapterを使用する回帰を含む全HTML surface29件も成功（974件filtered、exit0）。
strict all-target release Clippyも`-D warnings`で成功（exit0）。formatとdiff-checkも成功。
全coverage、通常hook付き正式統合、最新HEADのCIと実原本の再受入は別に残る。
読み取り専用の独立レビューではP0/P1無し。実callback接続の回帰は残る。

## 残る受入

原本HTMLの初回49.37秒、navigation45.11秒、全タブ正常close5秒超過の実失敗は残す。
この画面更新修正だけで描画性能や正常closeが解消したとは扱わない。
Officeのcold RSS超過、packaged各OS受入、canonical fidelityと公開も未達である。

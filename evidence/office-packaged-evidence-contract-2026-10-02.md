# Officeのpackaged実行証跡と性能上限

## 対象

PR #346のP1 comment4162738590とP2 comment4162738593。
従来はOfficeの`release_worker: true`と正の初回表示時間だけで受理し、空workspaceの
packaged smokeとin-process Office結果の混合でもstrict gateを通過できた。

## 検査の修正

Office入力ごとに`packaged_target`と`packaged_run`を必須化した。
runはpackaged main、clean machine、正常close、main/sidecar PID、heartbeat進行、CPU/RSS、
実行pathと宣言・実測SHAを既存packaged検査と同じ関数で確認する。
さらに入力SHA、一意で前後空白のないrun ID、targetと同一のmain/sidecar path・SHAを確認する。
各runのcold RSSとclose後RSSは正整数の実測値を要求し、既存192MiB増分上限を維持する。
初回frameは既存15秒上限を適用し、15,000msちょうどを許容、15,000.001msを拒否する。

既存の全4target、原本HTML、source tree/lock hash、公開registryグラフ検査は維持した。
実際のOffice close後資源・warm cycle等の残る受入は、物理tasksと実ファイル受入で継続する。

## 回帰と限界

旧validatorでは追加回帰の17条件が`AcceptanceEvidenceError not raised`でRED。
修正後は有効な合成unit fixtureを含む検査26件とstrict release契約15件が成功した。
自己宣言と観測SHAを共に変更した別artifact、別path、欠落/異常なrun情報、停止heartbeat、
in-processの混入、RSS増分境界・境界+1、表示時間境界・超過も検査する。
独立の読み取り専用レビューはP0/P1無し。run ID空白差分のP2は拒否する回帰を追加した。

合成fixtureはvalidatorのunit test専用で、実配布受入の証跡ではない。
実際のdocument-acceptance-v0.22.42.jsonは未生成のまま保持し、製品性能・メモリ・
packaged全OS受入が成功したことにはしない。実ファイルの失敗を成功へ書き換えていない。

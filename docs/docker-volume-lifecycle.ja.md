# Docker検証用volumeの保持と回収

KatanAが新たに作成するvolumeには所有repositoryと用途のラベルを付けます。既存のラベルなしvolumeや他repositoryのvolumeは、自動的に所有対象へ取り込みません。

## 確認と対象指定の回収

```sh
rtk proxy just docker-volumes-preview
rtk proxy just docker-volume-delete-ui-run docker-ui-functional.実行ID
rtk proxy just docker-volume-delete-cache volume名
```

最初のコマンドは削除しません。後二つは指定した実行またはキャッシュ一つだけが対象です。キャッシュの削除には対象名の再入力が必要です。全体pruneとforce削除は行いません。

## 固定キャッシュ

Linux/Windowsのtargetは再利用する固定キャッシュです。検証ごとに消さず、不要になった場合に、停止中を含む全コンテナから参照されていない対象だけを明示的に削除します。プレビューはDockerが返す使用量と参照コンテナID・状態を表示し、エンジンが使用量を返さない場合は `unknown` と表示します。Cargo registry・xwinも共有キャッシュとして容量と参照状況を表示しますが、削除コマンドでは受け付けません。容量が必要な場合も、所有と参照を調べずに全キャッシュを一括削除しません。

## 実行ごとの画面検証データ

正常終了と証跡取り出しが成功した実行は、従来どおりその実行の資源を回収します。失敗した実行は調査用に残します。

失敗時も、アプリ停止後の証跡をhostへ取り出せた場合だけ、内容のSHA-256を含む保存記録を作ります。後日の回収には、所有・用途・実行IDの一致、X11/evidenceの組、コンテナ参照なし、保存記録とhost上の内容の一致、保存後30日経過をすべて要求します。30日は調査資料を残すための既定保持期間であり、自動削除の予定やリリース承認期限ではありません。期間を過ぎても対象指定コマンドを実行するまで削除しません。

保存失敗・内容変更・不完全な組・不明な所有は保持します。ラベルのない過去のvolumeも、この仕組みで削除しません。

## 検証

```sh
rtk proxy just test-docker-volume-lifecycle
rtk proxy just check-docker-volume-lifecycle
```

後者は専用名の小さな実Docker fixtureで、target/shared cacheの表示、cache削除ガードと削除直前の再確認、作成済み・稼働中・停止中コンテナからの参照、証跡未保存、保存済み期限経過、再実行、producerの正常終了・強制停止・証跡取り出し失敗を検査します。既存volumeの名前一覧が変わらないことも確認します。アプリ本体のLinux/Windows検証や画面操作の受入検査を代替しません。

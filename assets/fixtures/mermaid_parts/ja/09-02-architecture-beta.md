# 9.2. アーキテクチャ図（マルチサービス）

~~~mermaid
architecture-beta
    group api(cloud)[API]

    service db(database)[データベース] in api
    service disk1(disk)[ストレージ] in api
    service disk2(disk)[ストレージ] in api
    service server(server)[サーバー] in api

    db:L -- R:server
    disk1:T -- B:server
    disk2:T -- B:db
~~~

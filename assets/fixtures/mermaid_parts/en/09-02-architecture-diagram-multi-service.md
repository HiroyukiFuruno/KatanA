# 9.2. Architecture Diagram (Multi-service)

~~~mermaid
architecture-beta
    group api(cloud)[API]

    service db(database)[Database] in api
    service disk1(disk)[Storage] in api
    service disk2(disk)[Storage] in api
    service server(server)[Server] in api

    db:L -- R:server
    disk1:T -- B:server
    disk2:T -- B:db
~~~

<!-- katana-mermaid-official:start -->

## 公式Mermaid.js描画

![公式Mermaid.js描画: 9.2. Architecture Diagram (Multi-service)](official-dark/09-02-architecture-diagram-multi-service.png)

<!-- katana-mermaid-official:end -->

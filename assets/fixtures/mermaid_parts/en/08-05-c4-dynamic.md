# 8.5. C4 Dynamic

~~~mermaid
C4Dynamic
    title Dynamic diagram for API Application
    Container(spa, "Single Page Application", "javascript and react")
    Container(api, "API Application", "Java and Spring Boot")
    Rel(spa, api, "Uses", "JSON/HTTPS")
~~~

<!-- katana-mermaid-official:start -->

## 公式Mermaid.js描画

![公式Mermaid.js描画: 8.5. C4 Dynamic](official-dark/08-05-c4-dynamic.png)

<!-- katana-mermaid-official:end -->

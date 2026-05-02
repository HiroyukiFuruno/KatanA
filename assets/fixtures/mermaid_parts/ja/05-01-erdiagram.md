# 5.1. ER図（シンプル）

~~~mermaid
erDiagram
    DOCUMENT ||--o{ SECTION : "含む"
    SECTION ||--o| DIAGRAM : "レンダリングする"
    DOCUMENT {
        string path
        string title
    }
    SECTION {
        int ordinal
        string kind
    }
~~~

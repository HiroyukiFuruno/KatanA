# 5.1. ER Diagram (Simple)

~~~mermaid
erDiagram
    DOCUMENT ||--o{ SECTION : contains
    SECTION ||--o| DIAGRAM : renders
    DOCUMENT {
        string path
        string title
    }
    SECTION {
        int ordinal
        string kind
    }
~~~

<!-- katana-mermaid-official:start -->

## 公式Mermaid.js描画

![公式Mermaid.js描画: 5.1. ER Diagram (Simple)](official-dark/05-01-er-diagram-simple.png)

<!-- katana-mermaid-official:end -->

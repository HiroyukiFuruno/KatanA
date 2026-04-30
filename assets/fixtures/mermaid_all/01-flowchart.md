# 01. Flowchart / Graph

~~~mermaid
graph TD
    Start[開始] --> Choice{分岐}
    Choice -->|はい| Process[処理]
    Choice -->|いいえ| Stop[停止]
    Process --> Stop
~~~

<!-- katana-mermaid-official:start -->

## 公式Mermaid.js描画

![公式Mermaid.js描画: 01. Flowchart / Graph](official/01-flowchart.png)

<!-- katana-mermaid-official:end -->

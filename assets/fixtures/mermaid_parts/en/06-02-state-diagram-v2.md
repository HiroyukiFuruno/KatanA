# 6.2. State Diagram v2

~~~mermaid
stateDiagram-v2
    [*] --> Still
    Still --> [*]
    Still --> Moving
    Moving --> Still
    Moving --> Crash
    Crash --> [*]
~~~

<!-- katana-mermaid-official:start -->

## 公式Mermaid.js描画

![公式Mermaid.js描画: 6.2. State Diagram v2](official-dark/06-02-state-diagram-v2.png)

<!-- katana-mermaid-official:end -->

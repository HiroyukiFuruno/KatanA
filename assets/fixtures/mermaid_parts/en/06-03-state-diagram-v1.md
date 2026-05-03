# 6.3. State Diagram v1

~~~mermaid
stateDiagram
    [*] --> Still
    Still --> [*]
    Still --> Moving
    Moving --> Still
    Moving --> Crash
    Crash --> [*]
~~~

<!-- katana-mermaid-official:start -->

## 公式Mermaid.js描画

![公式Mermaid.js描画: 6.3. State Diagram v1](official-dark/06-03-state-diagram-v1.png)

<!-- katana-mermaid-official:end -->

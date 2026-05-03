# 19.2. 要件図（複数）

~~~mermaid
requirementDiagram
    requirement independent_runtime {
        id: R1
        text: OSに依存しないランタイム
        risk: high
        verifymethod: test
    }
    requirement accurate_rendering {
        id: R2
        text: 高速かつ正確なレンダリング
        risk: high
        verifymethod: inspection
    }
    independent_runtime - satisfies -> accurate_rendering
~~~

<!-- katana-mermaid-official:start -->

## 公式Mermaid.js描画

![公式Mermaid.js描画: 19.2. 要件図（複数）](official-dark/19-02-requirement-diagram.png)

<!-- katana-mermaid-official:end -->

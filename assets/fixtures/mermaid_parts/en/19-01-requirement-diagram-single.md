# 19.1. Requirement Diagram (Single)

~~~mermaid
requirementDiagram

    requirement test_req {
    id: 1
    text: the test text.
    risk: high
    verifymethod: test
    }

    element test_entity {
    type: simulation
    }

    test_entity - satisfies -> test_req
~~~

<!-- katana-mermaid-official:start -->

## 公式Mermaid.js描画

![公式Mermaid.js描画: 19.1. Requirement Diagram (Single)](official-dark/19-01-requirement-diagram-single.png)

<!-- katana-mermaid-official:end -->

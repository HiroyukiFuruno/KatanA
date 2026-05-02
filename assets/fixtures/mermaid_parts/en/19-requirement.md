# 19. Requirement Diagram

## 19.1. Requirement Diagram (Single)

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

## 19.2. Requirement Diagram (Multi)

~~~mermaid
requirementDiagram
    requirement independent_runtime {
        id: R1
        text: OS independent runtime
        risk: high
        verifymethod: test
    }
    requirement accurate_rendering {
        id: R2
        text: Fast accurate rendering
        risk: high
        verifymethod: inspection
    }
    independent_runtime - satisfies -> accurate_rendering
~~~

<!-- katana-mermaid-official:start -->

## Official Mermaid.js Rendering

![Official Mermaid.js Rendering: 19. Requirement Diagram](../official/19-requirement.png)

<!-- katana-mermaid-official:end -->

# 06. State Diagram

## 6.1. State Diagram v2 (Failure path)

~~~mermaid
stateDiagram-v2
    [*] --> Pending
    Pending --> Image : success
    Pending --> Error : failure
    Image --> [*]
    Error --> [*]
~~~

## 6.2. State Diagram v2

~~~mermaid
stateDiagram-v2
    [*] --> Still
    Still --> [*]
    Still --> Moving
    Moving --> Still
    Moving --> Crash
    Crash --> [*]
~~~

## 6.3. State Diagram v1

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

## Official Mermaid.js Rendering

![Official Mermaid.js Rendering: 06. State Diagram](../official/06-state.png)

<!-- katana-mermaid-official:end -->

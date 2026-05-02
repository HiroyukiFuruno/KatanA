# 04. Sequence Diagram

## 4.1. Sequence Diagram (Simple)

~~~mermaid
sequenceDiagram
    participant User as User
    participant App as KatanA
    User->>App: Open Markdown
    App-->>User: Update Preview
~~~

## 4.2. Sequence Diagram (Activate/Deactivate)

~~~mermaid
sequenceDiagram
    Alice->>+John: Hello John, how are you?
    Alice->>+John: John, can you hear me?
    John-->>-Alice: Hi Alice, I can hear you!
    John-->>-Alice: I feel great!
~~~

<!-- katana-mermaid-official:start -->

## Official Mermaid.js Rendering

![Official Mermaid.js Rendering: 04. Sequence Diagram](../official/04-sequence.png)

<!-- katana-mermaid-official:end -->

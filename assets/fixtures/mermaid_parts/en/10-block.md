# 10. Block Diagram

## 10.1. Block Diagram (Horizontal)

~~~mermaid
block-beta
    columns 3
    source["Markdown"] parser["Parser"] renderer["Renderer"]
    source --> parser
    parser --> renderer
~~~

## 10.2. Block Diagram (Vertical)

~~~mermaid
block-beta
columns 1
  db(("DB"))
  blockArrowId6<["&nbsp;&nbsp;&nbsp;"]>(down)
  block:ID
    A
    B["A wide one in the middle"]
    C
  end
  space
  D
  ID --> D
  C --> D
  style B fill:#969,stroke:#333,stroke-width:4px
~~~

<!-- katana-mermaid-official:start -->

## Official Mermaid.js Rendering

![Official Mermaid.js Rendering: 10. Block Diagram](../official/10-block.png)

<!-- katana-mermaid-official:end -->

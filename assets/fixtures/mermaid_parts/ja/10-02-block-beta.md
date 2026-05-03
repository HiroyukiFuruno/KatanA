# 10.2. ブロック図（縦方向）

~~~mermaid
block-beta
columns 1
  db(("DB"))
  blockArrowId6<["&nbsp;&nbsp;&nbsp;"]>(down)
  block:ID
    A
    B["中央の広いブロック"]
    C
  end
  space
  D
  ID --> D
  C --> D
  style B fill:#969,stroke:#333,stroke-width:4px
~~~

<!-- katana-mermaid-official:start -->

## 公式Mermaid.js描画

![公式Mermaid.js描画: 10.2. ブロック図（縦方向）](official-dark/10-02-block-beta.png)

<!-- katana-mermaid-official:end -->

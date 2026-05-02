# 25. Venn Diagram

## 25.1. Venn Diagram (2 sets)

~~~mermaid
venn-beta
    title Renderer scope
    set official ["Official Mermaid.js"]: 40
    set rust ["Rust-managed runtime"]: 35
    union official, rust: 25
~~~

## 25.2. Venn Diagram (3 sets with styles)

~~~mermaid
venn-beta
    title "Three overlapping sets"
    set A
    set B
    set C
    union A,B["AB"]
    union B,C["BC"]
    union A,C["AC"]
    union A,B,C["ABC"]
    style A,B fill:skyblue
    style B,C fill:orange
    style A,C fill:lightgreen
    style A,B,C fill:white, color:red
~~~

<!-- katana-mermaid-official:start -->

## Official Mermaid.js Rendering

![Official Mermaid.js Rendering: 25. Venn Diagram](../official/25-venn.png)

<!-- katana-mermaid-official:end -->

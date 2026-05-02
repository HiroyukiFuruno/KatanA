# 25.2. ベン図（3集合・スタイル付き）

~~~mermaid
venn-beta
    title "3つの重なり合う集合"
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

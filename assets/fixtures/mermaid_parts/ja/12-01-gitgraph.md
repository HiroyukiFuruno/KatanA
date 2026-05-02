# 12.1. Gitグラフ（シンプル）

~~~mermaid
gitGraph
    commit id: "base"
    branch feature
    checkout feature
    commit id: "rust-js"
    checkout main
    merge feature
~~~

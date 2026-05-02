# 12. Git Graph

## 12.1. Git Graph (Simple)

~~~mermaid
gitGraph
    commit id: "base"
    branch feature
    checkout feature
    commit id: "rust-js"
    checkout main
    merge feature
~~~

## 12.2. Git Graph (Multi-branch)

~~~mermaid
gitGraph
    commit
    branch develop
    checkout develop
    commit
    commit
    checkout main
    merge develop
    commit
    branch feature
    checkout feature
    commit
    commit
    checkout main
    merge feature
~~~

<!-- katana-mermaid-official:start -->

## Official Mermaid.js Rendering

![Official Mermaid.js Rendering: 12. Git Graph](../official/12-git-graph.png)

<!-- katana-mermaid-official:end -->

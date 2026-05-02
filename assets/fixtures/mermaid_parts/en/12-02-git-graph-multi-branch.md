# 12.2. Git Graph (Multi-branch)

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

## 公式Mermaid.js描画

![公式Mermaid.js描画: 12.2. Git Graph (Multi-branch)](official-dark/12-02-git-graph-multi-branch.png)

<!-- katana-mermaid-official:end -->

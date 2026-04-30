# Mermaid 全パターン確認用

この fixture は、KatanA が読み込む `mermaid.min.js` で検出される Mermaid 図形種別をまとめて確認するためのもの。

---

## 1. Flowchart / Graph

```mermaid
graph TD
    Start[開始] --> Choice{分岐}
    Choice -->|はい| Process[処理]
    Choice -->|いいえ| Stop[停止]
    Process --> Stop
```

## 2. Sequence Diagram

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant App as KatanA
    User->>App: Markdownを開く
    App-->>User: Previewを更新
```

## 3. Class Diagram

```mermaid
classDiagram
    class PreviewPane {
        +full_render(source)
        +show_content(ui)
    }
    class RenderedSection {
        <<enumeration>>
        Markdown
        Image
        Error
    }
    PreviewPane --> RenderedSection
```

## 4. State Diagram

```mermaid
stateDiagram-v2
    [*] --> Pending
    Pending --> Image : 成功
    Pending --> Error : 失敗
    Image --> [*]
    Error --> [*]
```

## 5. Entity Relationship Diagram

```mermaid
erDiagram
    DOCUMENT ||--o{ SECTION : contains
    SECTION ||--o| DIAGRAM : renders
    DOCUMENT {
        string path
        string title
    }
    SECTION {
        int ordinal
        string kind
    }
```

## 6. User Journey

```mermaid
journey
    title Diagram preview
    section 編集
      Markdownを書く: 5: User
      図形を確認する: 4: User, KatanA
    section 出力
      HTMLへ書き出す: 3: KatanA
```

## 7. Gantt Chart

```mermaid
gantt
    title Mermaid renderer schedule
    dateFormat YYYY-MM-DD
    todayMarker off
    section Spike
    DOM shim: done, 2026-04-01, 7d
    section Integration
    Production path: active, 2026-04-08, 14d
```

## 8. Pie Chart

```mermaid
pie title Rendering ownership
    "Rust-managed JS" : 70
    "SVG rasterize" : 20
    "Export runtime" : 10
```

## 9. Requirement Diagram

```mermaid
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
```

## 10. Git Graph

```mermaid
gitGraph
    commit id: "base"
    branch feature
    checkout feature
    commit id: "rust-js"
    checkout main
    merge feature
```

## 11. C4 Diagram

```mermaid
C4Context
    title KatanA renderer context
    Person(user, "ユーザー")
    System(katana, "KatanA")
    System_Ext(files, "Markdown files")
    Rel(user, katana, "編集する")
    Rel(katana, files, "読み書きする")
```

## 12. Mindmap

```mermaid
mindmap
  root((Mermaid))
    Runtime
      V8
      DOM shim
    Output
      SVG
      Rasterize
    Quality
      Layout
      Color
```

## 13. Timeline

```mermaid
timeline
    title Mermaid runtime adoption
    Spike : DOM shim
          : SVG generation
    Integration : Preview path
                : Cache profile
    Review : Fixture coverage
           : Performance check
```

## 14. Quadrant Chart

```mermaid
quadrantChart
    title Runtime evaluation
    x-axis Slow --> Fast
    y-axis OS dependent --> OS independent
    quadrant-1 Candidate
    quadrant-2 Needs work
    quadrant-3 Rejected
    quadrant-4 Overkill
    Rust-managed JS: [0.82, 0.86]
    OS Chrome: [0.35, 0.20]
```

## 15. XY Chart

```mermaid
xychart-beta
    title "Render time"
    x-axis [1, 2, 3, 4]
    y-axis "ms" 0 --> 100
    line [80, 62, 48, 42]
```

## 16. Sankey

```mermaid
sankey-beta
Markdown,Parser,10
Parser,Mermaid,4
Parser,HTML,6
Mermaid,SVG,4
SVG,Preview,4
```

## 17. Block Diagram

```mermaid
block-beta
    columns 3
    source["Markdown"] parser["Parser"] renderer["Renderer"]
    source --> parser
    parser --> renderer
```

## 18. Packet Diagram

```mermaid
packet-beta
0-15: "source hash"
16-31: "theme"
32-63: "renderer profile"
```

## 19. Kanban

```mermaid
kanban
    Todo
      [export runtime]
    Doing
      [Rust-managed Mermaid]
    Done
      [OS Chrome経路削除]
```

## 20. Architecture Diagram

```mermaid
architecture-beta
    group app(cloud)[KatanA]
    service markdown(server)[Markdown] in app
    service renderer(server)[Renderer] in app
    service svg(database)[SVG cache] in app
    markdown:R -- L:renderer
    renderer:R -- L:svg
```

## 21. Radar Chart

```mermaid
radar-beta
    title Mermaid runtime
    axis Speed, Accuracy, Portability, Maintainability
    curve Current {4, 4, 5, 3}
    curve Target {5, 5, 5, 4}
    max 5
```

## 22. Tree View

```mermaid
treeView-beta
    "Root"
        "Runtime"
            "V8"
            "DOM shim"
        "Output"
            "SVG"
            "Rasterize"
```

## 23. Ishikawa Diagram

```mermaid
ishikawa-beta
  Diagram quality
    Runtime
      DOM API
      SVG API
    Layout
      Text measurement
      ViewBox
    Color
      Theme
      Background
```

## 24. Venn Diagram

```mermaid
venn-beta
    title Renderer scope
    set official ["Official Mermaid.js"]: 40
    set rust ["Rust-managed runtime"]: 35
    union official, rust: 25
```

## 25. Treemap

```mermaid
treemap
    title Runtime cost
    "Mermaid" : 45
    "DOM shim" : 25
    "Rasterize" : 20
    "Cache" : 10
```

## 26. Wardley Map

```mermaid
wardley-beta
    title Renderer adoption
    anchor User [0.95, 0.62]
    component Preview [0.78, 0.55]
    component MermaidJS [0.62, 0.42]
    component DOMShim [0.38, 0.35]
    User->Preview
    Preview->MermaidJS
    MermaidJS->DOMShim
```

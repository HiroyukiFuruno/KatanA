# 18. Radar Chart

## 18.1. Radar Chart (4 axes)

~~~mermaid
radar-beta
    title Mermaid runtime
    axis Speed, Accuracy, Portability, Maintainability
    curve Current {4, 4, 5, 3}
    curve Target {5, 5, 5, 4}
    max 5
~~~

## 18.2. Radar Chart (6 axes)

~~~mermaid
---
title: "Grades"
---
radar-beta
  axis m["Math"], s["Science"], e["English"]
  axis h["History"], g["Geography"], a["Art"]
  curve a["Alice"]{85, 90, 80, 70, 75, 90}
  curve b["Bob"]{70, 75, 85, 80, 90, 85}

  max 100
  min 0
~~~

<!-- katana-mermaid-official:start -->

## Official Mermaid.js Rendering

![Official Mermaid.js Rendering: 18. Radar Chart](../official/18-radar.png)

<!-- katana-mermaid-official:end -->

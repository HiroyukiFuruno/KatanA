# 11. Gantt Chart

## 11.1. Gantt Chart (Status colors)

~~~mermaid
gantt
    title Mermaid renderer schedule
    dateFormat YYYY-MM-DD
    todayMarker off
    section Spike
    DOM shim: done, 2026-04-01, 7d
    section Integration
    Production path: active, 2026-04-08, 14d
~~~

## 11.2. Gantt Chart (Sections)

~~~mermaid
gantt
    title A Gantt Diagram
    dateFormat  YYYY-MM-DD
    section Section
    A task           :a1, 2014-01-01, 30d
    Another task     :after a1  , 20d
    section Another
    Task in sec      :2014-01-12  , 12d
    another task      : 24d
~~~

<!-- katana-mermaid-official:start -->

## Official Mermaid.js Rendering

![Official Mermaid.js Rendering: 11. Gantt Chart](../official/11-gantt.png)

<!-- katana-mermaid-official:end -->

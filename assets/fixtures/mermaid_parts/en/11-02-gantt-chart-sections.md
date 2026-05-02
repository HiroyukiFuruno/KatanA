# 11.2. Gantt Chart (Sections)

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

## 公式Mermaid.js描画

![公式Mermaid.js描画: 11.2. Gantt Chart (Sections)](official-dark/11-02-gantt-chart-sections.png)

<!-- katana-mermaid-official:end -->

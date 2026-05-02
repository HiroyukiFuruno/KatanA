# 3.1. Class Diagram (Enumeration)

~~~mermaid
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
~~~

<!-- katana-mermaid-official:start -->

## 公式Mermaid.js描画

![公式Mermaid.js描画: 3.1. Class Diagram (Enumeration)](official-dark/03-01-class-diagram-enumeration.png)

<!-- katana-mermaid-official:end -->

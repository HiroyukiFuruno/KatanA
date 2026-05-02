# 9.1. アーキテクチャ図（シンプル）

~~~mermaid
architecture-beta
    group app(cloud)[KatanA]
    service markdown(server)[Markdown] in app
    service renderer(server)[レンダラー] in app
    service svg(database)[SVGキャッシュ] in app
    markdown:R -- L:renderer
    renderer:R -- L:svg
~~~

<!-- katana-mermaid-official:start -->

## 公式Mermaid.js描画

![公式Mermaid.js描画: 9.1. アーキテクチャ図（シンプル）](official-dark/09-01-architecture-beta.png)

<!-- katana-mermaid-official:end -->

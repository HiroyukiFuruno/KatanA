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

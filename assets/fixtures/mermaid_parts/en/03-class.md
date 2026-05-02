# 03. Class Diagram

## 3.1. Class Diagram (Enumeration)

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

## 3.2. Class Diagram (Inheritance)

~~~mermaid
classDiagram
    Animal <|-- Duck
    Animal <|-- Fish
    Animal <|-- Zebra
    Animal : +int age
    Animal : +String gender
    Animal: +isMammal()
    Animal: +mate()
    class Duck{
      +String beakColor
      +swim()
      +quack()
    }
    class Fish{
      -int sizeInFeet
      -canEat()
    }
    class Zebra{
      +bool is_wild
      +run()
    }
~~~

<!-- katana-mermaid-official:start -->

## Official Mermaid.js Rendering

![Official Mermaid.js Rendering: 03. Class Diagram](../official/03-class.png)

<!-- katana-mermaid-official:end -->

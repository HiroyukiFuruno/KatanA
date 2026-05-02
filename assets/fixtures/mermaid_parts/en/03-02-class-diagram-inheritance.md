# 3.2. Class Diagram (Inheritance)

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

## 公式Mermaid.js描画

![公式Mermaid.js描画: 3.2. Class Diagram (Inheritance)](official-dark/03-02-class-diagram-inheritance.png)

<!-- katana-mermaid-official:end -->

# 🧪 KatanA Rendering — Basic Markdown

This fixture exercises core Markdown rendering: headings, text decoration,
lists, code blocks, tables, blockquotes, note blocks, accordion, math, and edge cases.

<p align="center">
  English | <a href="sample_basic.ja.md">日本語</a>
</p>

---

## 1. Heading Levels

# H1 Heading

## H2 Heading

### H3 Heading

#### H4 Heading

##### H5 Heading

###### H6 Heading

---

## 2. Text Decoration

- **Bold text**
- *Italic text*
- ~~Strikethrough~~
- <u>Underline</u>
- `Inline code`
- <mark>Highlight</mark>
- **Bold and *italic* mixed**

### Links

- [Normal link](https://github.com)
- [Email link](mailto:test@example.com)
- Autolink: <https://github.com>

### Horizontal Rule

Text above

---

Text below

---

## 3. Lists

### 3.1 Unordered Lists

- Item 1
- Item 2
  - Nested item 2-1
  - Nested item 2-2
    - Deeply nested 2-2-1
- Item 3

### 3.2 Ordered Lists

1. First item
2. Second item
   3. Nested 2-1
   4. Nested 2-2
5. Third item

### 3.3 Task Lists

- [x] Completed task
- [-] Pending task
- [x] Another completed task
  - [/] Nested pending
  - [ ] Nested completed

---

## 4. Code Blocks

### 4.1 Basic Code Block

```rust
fn main() {
    println!("Hello, KatanA!");
}
```

### 4.2 Code Block Without Language

```text
This is a code block without language specification.
No syntax highlighting should be applied.
```

### 4.3 Multiple Languages

```python
def hello():
    print("Hello from Python")
```

```javascript
function hello() {
    console.log("Hello from JavaScript");
}
```

```json
{
  "name": "katana",
  "version": "0.0.2",
  "features": ["markdown", "diagrams"]
}
```

```yaml
name: KatanA
version: 0.0.2
features:
  - markdown
  - diagrams
```

### 4.4 Code Block Inside List (Past Bug: Incorrect Rendering)

1. First step:

   ```sh
   cargo build --release
   ```

2. Second step:

   ```sh
   ./target/release/KatanA
   ```

3. Verify:
  - Sub-item A
  - Sub-item B

↑ Code blocks should be correctly indented within list items.

### 4.5 Code Block Inside Nested List

- Outer item
  - Inner item

    ```rust
    let x = 42;
    println!("{}", x);
    ```

  - Continuation

↑ Code block should not break the nested list layout.

---

## 5. Tables (GFM)

### 5.1 Basic Table

| Feature | Status | Notes |
| --- | --- | --- |
| Markdown | ✅ | Full support |
| Mermaid | ✅ | Requires mmdc |
| PlantUML | ✅ | Requires jar |
| DrawIo | ✅ | Pure Rust |

### 5.2 Table with Alignment

| Left | Center | Right |
| :--- | :---: | ---: |
| text | text | text |
| longer text | short | 12345 |

### 5.3 Single Row Table

| Header |
| --- |
| Content |

---

## 6. Blockquotes

### 6.1 Basic Quote

> This is a blockquote.
> It can span multiple lines.

### 6.2 Nested Quotes

> Outer quote
> > Inner quote
> > > Even deeper

### 6.3 Decorated Quote

> **Bold quote**
>
> - List item 1
> - List item 2
>
> ```rust
> let quoted_code = true;
> ```

---

## 7. Note Blocks (Legacy `> **Type**` style)

### 7.1 Note

> **Note**
> GitHub では note 系ブロックを blockquote として表現する。

### 7.2 Tip

> **Tip**
> 短い補足は、読みやすい 2 行程度にまとめると確認しやすい。

### 7.3 Important

> **Important**
> 強調したい注意は、見出し2 配下で独立した blockquote にする。

### 7.4 Warning

> **Warning**
> 危険な操作や壊れやすい表現は、引用ブロックで明示する。

### 7.5 Caution

> **Caution**
> 読者に作業前提を意識させたい場合に使う。

---

## 8. Alert Blocks (GFM `[!TYPE]` style)

> [!NOTE]
> Highlights information that users should take into account, even when skimming.

> [!TIP]
> Optional information to help a user be more successful.

> [!IMPORTANT]
> Crucial information necessary for users to succeed.

> [!WARNING]
> Critical content demanding immediate user attention due to potential risks.

> [!CAUTION]
> Negative potential consequences of an action.

---

## 9. Accordion

<details><summary>Show details</summary><div>

- Swords
  - Muramasa
  - Masamune
  - Kotetsu

</div></details>

---

## 10. Math

### 10.1 Block Math

```math
f(x) = \int_{0}^{x} \frac{t^2}{1 + t^4} \, dt
```

### 10.2 Inline Math

Mass-energy equivalence: $E = mc^2$

### 10.3 Single-line Math

$ \sum_{k=1}^{n} k = \frac{n(n+1)}{2} $$

---

## 11. Mixed Content (Past Bug: Section Boundary Breaking)

Markdown text, code blocks, and tables mixed together.
Verify no layout overlap between sections.

### Architecture Overview

| Component | Role |
| --- | --- |
| `PreviewPane` | Section management |
| `show_content` | UI rendering |

Proper spacing between the table above and the code block below.

```rust
enum RenderedSection {
    Markdown(String),
    Image { svg_data: RasterizedSvg, alt: String },
    Error { kind: String, message: String },
    CommandNotFound { tool_name: String },
    NotInstalled { kind: String },
    Pending { kind: String },
}
```

↑ All sections should render correctly without overlapping.

---

## 12. Edge Cases

### 12.1 Empty Code Block

```empty
```

### 12.2 Very Long Line

`This is a very long line to verify horizontal scrolling or word wrapping. ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 repeated. ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789`

### 12.3 Special Characters

- HTML entities: &amp; &lt; &gt; &quot;
- Japanese: こんにちは世界 🌍
- Emoji: 🦀 ⚡ 📝 🔧 ✅ ❌ ⚠️ 💡
- Math symbols: α β γ δ ∑ ∫ √ ∞

### 12.4 Footnotes

This text has a footnote[^1]. Here's another[^2].

[^1]: First footnote content.
[^2]: Second footnote content.

### 12.5 Consecutive Different Block Elements

> A blockquote

```rust
let code = "directly after quote";
```

- A list item directly after code block

| Header |
| --- |
| Table after list |

↑ Proper spacing between each block element.

---

## ✅ Verification Complete

If all sections above render correctly, basic Markdown rendering is working.

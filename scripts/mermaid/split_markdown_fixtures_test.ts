import { expect, test } from "bun:test";
import { MarkdownMermaidFixtures } from "./split_markdown_fixtures_core";

test("見出し3ごとにMermaid fixtureを分割する", () => {
  const fixtures = MarkdownMermaidFixtures.from(`## 8. c4

### C4Context simple

\`\`\`mermaid
C4Context
Person(user, "User")
\`\`\`

### C4Deployment

~~~mermaid
C4Deployment
Deployment_Node(node, "Node")
~~~
`);

  expect(fixtures.map((it) => it.fileName)).toEqual([
    "08-01-c4-context-simple.md",
    "08-02-c4-deployment.md",
  ]);
});

test("日本語タイトルだけでも図形種別から安定したslugを作る", () => {
  const fixtures = MarkdownMermaidFixtures.from(`## 14. カンバン

~~~mermaid
---
config:
  kanban:
    ticketBaseUrl: 'https://example.test/#TICKET#'
---
kanban
  Todo
    id1[ドキュメント作成]
~~~
`);

  expect(fixtures[0].fileName).toBe("14-kanban.md");
});

test("見出し3の小数点つき番号はslugに残さない", () => {
  const fixtures = MarkdownMermaidFixtures.from(`## 4. Sequence Diagram

### 4.1. Sequence Diagram Simple

~~~mermaid
sequenceDiagram
  A->>B: Hello
~~~
`);

  expect(fixtures[0].fileName).toBe("04-01-sequence-diagram-simple.md");
});

test("空のMermaidブロックはemptyとして残す", () => {
  const fixtures = MarkdownMermaidFixtures.from(`## 29. 空

~~~mermaid
~~~
`);

  expect(fixtures[0].fileName).toBe("29-empty.md");
  expect(fixtures[0].source).toBe("");
});

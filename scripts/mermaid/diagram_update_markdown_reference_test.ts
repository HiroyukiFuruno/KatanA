import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { expect, test } from "bun:test";
import type { Fixture } from "./diagram_update_fixtures";
import { MarkdownReferenceWriter } from "./diagram_update_markdown_reference";

test("Markdownから同じ階層の公式PNGを参照する", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "katana-mermaid-reference-"));
  try {
    const filePath = path.join(root, "01-flowchart.md");
    const markdown = "# 1. Flowchart\n\n```mermaid\nflowchart TD\n  A-->B\n```\n";
    fs.writeFileSync(filePath, markdown, "utf8");

    const fixture: Fixture = {
      fileName: "01-flowchart.md",
      filePath,
      slug: "01-flowchart",
      title: "1. Flowchart",
      markdown,
      source: "flowchart TD\n  A-->B",
      fenceEnd: markdown.length,
    };

    new MarkdownReferenceWriter(path.join(root, "official-dark")).write(fixture);

    const updated = fs.readFileSync(filePath, "utf8");
    expect(updated).toContain(
      "![公式Mermaid.js描画: 1. Flowchart](official-dark/01-flowchart.png)",
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

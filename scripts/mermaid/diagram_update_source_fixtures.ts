import fs from "node:fs";
import path from "node:path";
import type { Fixture } from "./diagram_update_fixtures";
import { MarkdownMermaidFixtures } from "./split_markdown_fixtures_core";

export class SourceFixtureRepository {
  constructor(private inputPath: string) {}

  list(): Fixture[] {
    const markdown = fs.readFileSync(this.inputPath, "utf8");
    return MarkdownMermaidFixtures.fromMarkdown(markdown).map((fixture) => ({
      fileName: fixture.fileName,
      filePath: this.inputPath,
      slug: path.basename(fixture.fileName, ".md"),
      title: fixture.title,
      markdown: "",
      source: fixture.source,
      fenceEnd: 0,
    }));
  }
}

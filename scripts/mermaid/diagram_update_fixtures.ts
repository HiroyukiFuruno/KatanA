import fs from "node:fs";
import path from "node:path";
import type { RenderFixture } from "./official-renderer";

export interface Fixture extends RenderFixture {
  filePath: string;
  markdown: string;
  fenceEnd: number;
}

interface MermaidBlockResult {
  source: string;
  end: number;
}

export class FixtureRepository {
  private fixturesDir: string;

  constructor(fixturesDir: string) {
    this.fixturesDir = fixturesDir;
  }

  list(): Fixture[] {
    return fs
      .readdirSync(this.fixturesDir)
      .filter((fileName) => fileName.endsWith(".md"))
      .sort()
      .map((fileName) => this.read(fileName));
  }

  read(fileName: string): Fixture {
    const filePath = path.join(this.fixturesDir, fileName);
    const markdown = fs.readFileSync(filePath, "utf8");
    const block = MermaidBlock.extract(markdown, filePath);
    return {
      fileName: fileName,
      filePath,
      slug: path.basename(fileName, ".md"),
      title: MarkdownTitle.extract(markdown, fileName),
      markdown,
      source: block.source,
      fenceEnd: block.end,
    };
  }
}

class MermaidBlock {
  static extract(markdown: string, filePath: string): MermaidBlockResult {
    const pattern = /(^|\n)(`{3,}|~{3,})mermaid[^\n]*\n([\s\S]*?)\n\2[ \t]*(?=\n|$)/m;
    const match = pattern.exec(markdown);
    if (!match) {
      throw new Error(`Mermaid block not found: ${filePath}`);
    }
    return {
      source: match[3].trim(),
      end: match.index + match[0].length,
    };
  }
}

class MarkdownTitle {
  static extract(markdown: string, fileName: string): string {
    return markdown.match(/^#\s+(.+)$/m)?.[1] || path.basename(fileName, ".md");
  }
}

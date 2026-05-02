import fs from "node:fs";
import path from "node:path";
import type { Fixture } from "./diagram_update_fixtures";

const MARKER_START = "<!-- katana-mermaid-official:start -->";
const MARKER_END = "<!-- katana-mermaid-official:end -->";

export class MarkdownReferenceWriter {
  private outputDirName: string;

  constructor(outputDir: string) {
    this.outputDirName = path.basename(outputDir);
  }

  write(fixture: Fixture) {
    const withoutOld = this.removeOldSection(fixture.markdown);
    const section = this.section(fixture);
    const next =
      withoutOld.slice(0, fixture.fenceEnd) + section + withoutOld.slice(fixture.fenceEnd);
    fs.writeFileSync(fixture.filePath, `${next.trimEnd()}\n`, "utf8");
  }

  private removeOldSection(markdown: string): string {
    const pattern = new RegExp(
      `\\n*${escapeRegExp(MARKER_START)}[\\s\\S]*?${escapeRegExp(MARKER_END)}\\n*`,
      "m",
    );
    return markdown.replace(pattern, "\n");
  }

  private section(fixture: Fixture): string {
    const src = `${this.outputDirName}/${fixture.slug}.png`;
    const alt = escapeMarkdownImageAlt(`公式Mermaid.js描画: ${fixture.title}`);
    return `\n\n${MARKER_START}\n\n## 公式Mermaid.js描画\n\n![${alt}](${src})\n\n${MARKER_END}\n`;
  }
}

function escapeMarkdownImageAlt(value: string): string {
  return value.replaceAll("[", "\\[").replaceAll("]", "\\]");
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

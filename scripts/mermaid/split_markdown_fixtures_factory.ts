import type { MarkdownMermaidFixture } from "./split_markdown_fixtures_core";

export interface HeadingState {
  level2Index: number;
  level3Index: number;
  level2Title: string;
  level3Title: string;
}

type Level2NumberReader = (heading: HeadingState, match: RegExpMatchArray | null) => number;

type MermaidSourceReader = (source: string, lines: string[]) => string;

type MermaidFrontmatterBodyReader = (source: string, lines: string[], endOffset: number) => string;

export class MarkdownMermaidFixtureFactory {
  static from(heading: HeadingState, source: string): MarkdownMermaidFixture {
    const title = FixtureTitle.from(heading);
    return {
      fileName: FixtureFileName.from(heading, title, source),
      title,
      source,
    };
  }
}

class FixtureTitle {
  static from(heading: HeadingState): string {
    return [heading.level3Title, heading.level2Title, "Mermaid"].find((it) => it.length > 0) ?? "";
  }
}

class FixtureFileName {
  static from(heading: HeadingState, title: string, source: string): string {
    const prefix = FixtureIndex.from(heading);
    return `${prefix}-${FixtureSlug.from(title, source)}.md`;
  }
}

class FixtureIndex {
  static from(heading: HeadingState): string {
    const level2 = String(FixtureIndex.level2Number(heading)).padStart(2, "0");
    return [`${level2}-${String(heading.level3Index).padStart(2, "0")}`, level2][
      Number(heading.level3Index === 0)
    ];
  }

  private static level2Number(heading: HeadingState): number {
    const match = heading.level2Title.match(/^(\d+)\./);
    return FixtureIndex.level2NumberReaders()[Number(Boolean(match))](heading, match);
  }

  private static level2NumberReaders(): Level2NumberReader[] {
    return [FixtureIndex.fallbackLevel2Number, FixtureIndex.parsedLevel2Number];
  }

  private static fallbackLevel2Number(heading: HeadingState, _match: RegExpMatchArray | null) {
    return heading.level2Index;
  }

  private static parsedLevel2Number(_heading: HeadingState, match: RegExpMatchArray | null) {
    return Number.parseInt(match?.[1] ?? "0", 10);
  }
}

class FixtureSlug {
  static from(title: string, source: string): string {
    return (
      [title, DiagramKindName.from(source)]
        .map((value) => FixtureSlug.slug(value))
        .find((value) => value.length > 0) ?? "diagram"
    );
  }

  private static slug(value: string): string {
    return value
      .replace(/^\d+(?:\.\d+)*\.\s*/, "")
      .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "");
  }
}

class DiagramKindName {
  static from(source: string): string {
    return DiagramKindName.firstToken(MermaidSource.withoutFrontmatter(source));
  }

  private static firstToken(source: string): string {
    const kind = DiagramKindName.rawFirstToken(source);
    return [kind, "empty"][Number(kind.length === 0)];
  }

  private static rawFirstToken(source: string): string {
    return DiagramKindName.firstLine(source).trim().split(/\s+/).concat([""])[0];
  }

  private static firstLine(source: string): string {
    return source.split(/\r?\n/).concat([""])[0];
  }
}

class MermaidSource {
  static withoutFrontmatter(source: string): string {
    const lines = source.split(/\r?\n/);
    return MermaidSource.frontmatterReaders()[Number(MermaidSource.hasFrontmatter(lines))](
      source,
      lines,
    );
  }

  private static hasFrontmatter(lines: string[]): boolean {
    return lines[0]?.trim() === "---";
  }

  private static frontmatterReaders(): MermaidSourceReader[] {
    return [MermaidSource.original, MermaidSource.afterFrontmatter];
  }

  private static original(source: string, _lines: string[]): string {
    return source;
  }

  private static afterFrontmatter(source: string, lines: string[]): string {
    const endIndex = lines.slice(1).findIndex((line) => line.trim() === "---");
    return MermaidSource.frontmatterBodyReaders()[Number(endIndex >= 0)](source, lines, endIndex);
  }

  private static frontmatterBodyReaders(): MermaidFrontmatterBodyReader[] {
    return [MermaidSource.originalWithEndOffset, MermaidSource.bodyAfterFrontmatter];
  }

  private static originalWithEndOffset(source: string, _lines: string[], _endOffset: number) {
    return source;
  }

  private static bodyAfterFrontmatter(_source: string, lines: string[], endOffset: number) {
    return lines.slice(endOffset + 2).join("\n");
  }
}

import fs from "node:fs";
import path from "node:path";

interface CliParsedOptions {
  inputPath: string;
  outputDir: string;
}

interface MermaidFixture {
  title: string;
  source: string;
  index: number;
}

class CliOptions {
  static parse(argv: string[]): CliParsedOptions {
    CliOptions.exitIfHelp(argv);
    return {
      inputPath: path.resolve(CliOptions.get(argv, "--input", "assets/fixtures/mermaid.md")),
      outputDir: path.resolve(CliOptions.get(argv, "--output", "tmp/mermaid-sample-fixtures")),
    };
  }

  private static get(argv: string[], name: string, fallback: string): string {
    const index = argv.indexOf(name);
    return index >= 0 ? argv[index + 1] : fallback;
  }

  private static exitIfHelp(argv: string[]) {
    if (argv.includes("--help")) {
      console.log(
        "Usage: bun run scripts/mermaid/split-markdown-fixtures.ts [--input FILE] [--output DIR]",
      );
      process.exit(0);
    }
  }
}

class MarkdownMermaidExtractor {
  private markdown: string;

  constructor(markdown: string) {
    this.markdown = markdown;
  }

  extract(): MermaidFixture[] {
    return Array.from(this.markdown.matchAll(MarkdownMermaidPattern.value())).map((match, index) =>
      MermaidMatchFixture.from(match, index),
    );
  }
}

class MarkdownMermaidPattern {
  static value(): RegExp {
    return /^##\s+(.+)\n[\s\S]*?^(`{3,}|~{3,})mermaid[^\n]*\n([\s\S]*?)^\2[ \t]*$/gm;
  }
}

class MermaidMatchFixture {
  static from(match: RegExpMatchArray, index: number): MermaidFixture {
    return {
      title: match[1].trim(),
      source: match[3].trim(),
      index: index + 1,
    };
  }
}

class FixtureFileName {
  static from(fixture: MermaidFixture): string {
    const titleSlug = FixtureFileName.slug(fixture.title);
    return `${String(fixture.index).padStart(2, "0")}-${titleSlug}.md`;
  }

  private static slug(value: string): string {
    const normalized = value
      .toLowerCase()
      .replace(/^\d+\.\s*/, "")
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "");
    return normalized.length > 0 ? normalized : "diagram";
  }
}

class FixtureWriter {
  constructor(private outputDir: string) {}

  write(fixtures: MermaidFixture[]) {
    this.prepareOutputDir();
    for (const fixture of fixtures) {
      this.writeFixture(fixture);
    }
  }

  private prepareOutputDir() {
    fs.mkdirSync(this.outputDir, { recursive: true });
    this.markdownFileNames().forEach((fileName) => {
      fs.unlinkSync(path.join(this.outputDir, fileName));
    });
  }

  private markdownFileNames(): string[] {
    return fs.readdirSync(this.outputDir).filter((fileName) => fileName.endsWith(".md"));
  }

  private writeFixture(fixture: MermaidFixture) {
    const filePath = path.join(this.outputDir, FixtureFileName.from(fixture));
    const markdown = `# ${fixture.title}\n\n~~~mermaid\n${fixture.source.trimEnd()}\n~~~\n`;
    fs.writeFileSync(filePath, markdown, "utf8");
  }
}

class MarkdownFixtureSplit {
  constructor(private options: CliParsedOptions) {}

  run() {
    this.ensureInputExists();
    const markdown = fs.readFileSync(this.options.inputPath, "utf8");
    const fixtures = new MarkdownMermaidExtractor(markdown).extract();
    this.ensureFixturesExist(fixtures);
    new FixtureWriter(this.options.outputDir).write(fixtures);
    console.log(`split ${fixtures.length} diagrams into ${this.options.outputDir}`);
  }

  private ensureInputExists() {
    if (!fs.existsSync(this.options.inputPath)) {
      throw new Error(`Input markdown not found: ${this.options.inputPath}`);
    }
  }

  private ensureFixturesExist(fixtures: MermaidFixture[]) {
    if (fixtures.length === 0) {
      throw new Error(`Mermaid block not found: ${this.options.inputPath}`);
    }
  }
}

new MarkdownFixtureSplit(CliOptions.parse(process.argv.slice(2))).run();

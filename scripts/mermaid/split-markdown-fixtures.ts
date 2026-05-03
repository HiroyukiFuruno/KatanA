import fs from "node:fs";
import path from "node:path";
import {
  MarkdownMermaidFixtures,
  type MarkdownMermaidFixture,
} from "./split_markdown_fixtures_core";

interface CliParsedOptions {
  inputPath: string;
  outputDir: string;
}

class CliOptions {
  static parse(argv: string[]): CliParsedOptions {
    CliOptions.exitIfHelp(argv);
    return {
      inputPath: path.resolve(CliOptions.get(argv, "--input", "assets/fixtures/sample_mermaid.md")),
      outputDir: path.resolve(CliOptions.get(argv, "--output", "assets/fixtures/mermaid_parts/en")),
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

class FixtureWriter {
  constructor(private outputDir: string) {}

  write(fixtures: MarkdownMermaidFixture[]) {
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

  private writeFixture(fixture: MarkdownMermaidFixture) {
    const filePath = path.join(this.outputDir, fixture.fileName);
    const markdown = `# ${fixture.title}\n\n~~~mermaid\n${fixture.source.trimEnd()}\n~~~\n`;
    fs.writeFileSync(filePath, markdown, "utf8");
  }
}

class MarkdownFixtureSplit {
  constructor(private options: CliParsedOptions) {}

  run() {
    this.ensureInputExists();
    const markdown = fs.readFileSync(this.options.inputPath, "utf8");
    const fixtures = MarkdownMermaidFixtures.from(markdown);
    this.ensureFixturesExist(fixtures);
    new FixtureWriter(this.options.outputDir).write(fixtures);
    console.log(`split ${fixtures.length} diagrams into ${this.options.outputDir}`);
  }

  private ensureInputExists() {
    if (!fs.existsSync(this.options.inputPath)) {
      throw new Error(`Input markdown not found: ${this.options.inputPath}`);
    }
  }

  private ensureFixturesExist(fixtures: MarkdownMermaidFixture[]) {
    if (fixtures.length === 0) {
      throw new Error(`Mermaid block not found: ${this.options.inputPath}`);
    }
  }
}

new MarkdownFixtureSplit(CliOptions.parse(process.argv.slice(2))).run();

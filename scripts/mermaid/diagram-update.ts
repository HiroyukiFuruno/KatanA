import fs from "node:fs";
import path from "node:path";
import { FixtureRepository, type Fixture } from "./diagram_update_fixtures";
import { MarkdownReferenceWriter } from "./diagram_update_markdown_reference";
import { OfficialMermaidRenderer, expandHome, type RendererOptions } from "./official-renderer";

interface CliParsedOptions extends RendererOptions {
  fixturesDir: string;
  writeMarkdown: boolean;
  skipErrors: boolean;
}

class CliOptions {
  static parse(argv: string[]): CliParsedOptions {
    CliOptions.exitIfHelp(argv);
    const fixturesDir = path.resolve(
      CliOptions.get(argv, "--fixtures", "assets/fixtures/mermaid_all"),
    );
    return {
      fixturesDir,
      outputDir: path.resolve(CliOptions.get(argv, "--output", path.join(fixturesDir, "official"))),
      mermaidJs: path.resolve(
        expandHome(
          CliOptions.get(
            argv,
            "--mermaid-js",
            process.env.MERMAID_JS || "~/.local/katana/mermaid.min.js",
          ),
        ),
      ),
      writeMarkdown: !argv.includes("--no-write-md"),
      skipErrors: argv.includes("--skip-errors"),
    };
  }

  private static get(argv: string[], name: string, fallback: string): string {
    const index = argv.indexOf(name);
    return index >= 0 ? argv[index + 1] : fallback;
  }

  private static exitIfHelp(argv: string[]) {
    if (argv.includes("--help")) {
      console.log(
        "Usage: bun run scripts/mermaid/diagram-update.ts [--fixtures DIR] [--output DIR] [--mermaid-js FILE]",
      );
      process.exit(0);
    }
  }
}

class MermaidDiagramUpdate {
  private options: CliParsedOptions;

  constructor(options: CliParsedOptions) {
    this.options = options;
  }

  async run() {
    this.validate();
    fs.mkdirSync(this.options.outputDir, { recursive: true });
    const fixtures = new FixtureRepository(this.options.fixturesDir).list();
    const renderer = new OfficialMermaidRenderer(this.options);
    const writer = new MarkdownReferenceWriter(this.options.outputDir);
    await this.renderAll(renderer, writer, fixtures);
  }

  private async renderAll(
    renderer: OfficialMermaidRenderer,
    writer: MarkdownReferenceWriter,
    fixtures: Fixture[],
  ) {
    await renderer.start();
    try {
      await this.renderFixtures(renderer, writer, fixtures);
    } finally {
      await renderer.stop();
    }
  }

  private async renderFixtures(
    renderer: OfficialMermaidRenderer,
    writer: MarkdownReferenceWriter,
    fixtures: Fixture[],
  ) {
    for (const fixture of fixtures) {
      await this.renderFixture(renderer, writer, fixture);
    }
  }

  private async renderFixture(
    renderer: OfficialMermaidRenderer,
    writer: MarkdownReferenceWriter,
    fixture: Fixture,
  ) {
    if (this.options.skipErrors) {
      await this.renderOptionalFixture(renderer, writer, fixture);
      return;
    }
    await this.renderRequiredFixture(renderer, writer, fixture);
  }

  private async renderOptionalFixture(
    renderer: OfficialMermaidRenderer,
    writer: MarkdownReferenceWriter,
    fixture: Fixture,
  ) {
    try {
      await this.renderRequiredFixture(renderer, writer, fixture);
    } catch (error) {
      this.removeOutputFiles(fixture);
      console.warn(`skipped ${fixture.slug}: ${ErrorSummary.fromString(String(error))}`);
    }
  }

  private async renderRequiredFixture(
    renderer: OfficialMermaidRenderer,
    writer: MarkdownReferenceWriter,
    fixture: Fixture,
  ) {
    await renderer.render(fixture);
    this.writeMarkdownReference(writer, fixture);
    console.log(`updated ${fixture.slug}`);
  }

  private removeOutputFiles(fixture: Fixture) {
    this.outputPaths(fixture)
      .filter((outputPath) => fs.existsSync(outputPath))
      .forEach((outputPath) => {
        fs.unlinkSync(outputPath);
      });
  }

  private outputPaths(fixture: Fixture): string[] {
    return [".png", ".svg"].map((extension) =>
      path.join(this.options.outputDir, `${fixture.slug}${extension}`),
    );
  }

  private writeMarkdownReference(writer: MarkdownReferenceWriter, fixture: Fixture) {
    if (this.options.writeMarkdown) {
      writer.write(fixture);
    }
  }

  private validate() {
    if (!fs.existsSync(this.options.mermaidJs)) {
      throw new Error(`mermaid.min.js not found: ${this.options.mermaidJs}`);
    }
  }
}

class ErrorSummary {
  static fromString(value: string): string {
    return ErrorSummary.truncate(value.split("\n")[0]);
  }

  private static truncate(value: string): string {
    return value.length > 180 ? `${value.slice(0, 180)}...` : value;
  }
}

new MermaidDiagramUpdate(CliOptions.parse(process.argv.slice(2))).run().catch((error: Error) => {
  console.error(error.message);
  process.exit(1);
});

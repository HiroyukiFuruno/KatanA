import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  expandHome,
  OfficialDrawioRenderer,
  type DrawioRendererOptions,
  type DrawioRenderFixture,
} from "./official-renderer";

interface CliParsedOptions extends DrawioRendererOptions {
  fixturesDir: string;
}

class CliOptions {
  static parse(argv: string[]): CliParsedOptions {
    CliOptions.exitIfHelp(argv);
    const fixturesDir = path.resolve(
      CliOptions.get(argv, "--fixtures", "assets/fixtures/drawio/basic"),
    );
    return {
      fixturesDir,
      outputDir: path.resolve(CliOptions.get(argv, "--output", path.join(fixturesDir, "official"))),
      drawioJs: path.resolve(
        expandHome(
          CliOptions.get(
            argv,
            "--drawio-js",
            process.env.DRAWIO_JS || "~/.local/katana/drawio.min.js",
          ),
        ),
      ),
      resourcesDir: path.resolve(
        CliOptions.get(
          argv,
          "--resources",
          "crates/katana-core/src/markdown/drawio_renderer/js_runtime/resources",
        ),
      ),
      resourceManifest: path.resolve(
        CliOptions.get(
          argv,
          "--resource-manifest",
          "crates/katana-core/src/markdown/drawio_renderer/js_runtime/resources/drawio-resource-manifest.json",
        ),
      ),
    };
  }

  private static get(argv: string[], name: string, fallback: string): string {
    const index = argv.indexOf(name);
    return index >= 0 ? argv[index + 1] : fallback;
  }

  private static exitIfHelp(argv: string[]) {
    if (argv.includes("--help")) {
      console.log(
        "Usage: bun run scripts/drawio/diagram-update.ts [--fixtures DIR] [--output DIR] [--drawio-js FILE]",
      );
      process.exit(0);
    }
  }
}

class DrawioFixtureRepository {
  constructor(private fixturesDir: string) {}

  list(): DrawioRenderFixture[] {
    return fs
      .readdirSync(this.fixturesDir)
      .filter((fileName) => fileName.endsWith(".drawio"))
      .sort()
      .map((fileName) => this.read(fileName));
  }

  private read(fileName: string): DrawioRenderFixture {
    const filePath = path.join(this.fixturesDir, fileName);
    return {
      slug: path.basename(fileName, ".drawio"),
      source: fs.readFileSync(filePath, "utf8"),
    };
  }
}

class DrawioDiagramUpdate {
  constructor(private options: CliParsedOptions) {}

  async run() {
    this.validate();
    fs.mkdirSync(this.options.outputDir, { recursive: true });
    const renderer = new OfficialDrawioRenderer(this.options);
    await this.renderFixtures(
      renderer,
      new DrawioFixtureRepository(this.options.fixturesDir).list(),
    );
  }

  private async renderFixtures(renderer: OfficialDrawioRenderer, fixtures: DrawioRenderFixture[]) {
    await renderer.start();
    await this.renderStartedFixtures(renderer, fixtures).finally(() => renderer.stop());
  }

  private async renderStartedFixtures(
    renderer: OfficialDrawioRenderer,
    fixtures: DrawioRenderFixture[],
  ) {
    for (const fixture of fixtures) {
      await renderer.render(fixture);
      console.log(`updated ${fixture.slug}`);
    }
  }

  private validate() {
    this.validateDrawioJs();
    this.validateResourcesDir();
    this.validateResourceManifest();
  }

  private validateDrawioJs() {
    assert(
      fs.existsSync(this.options.drawioJs),
      `drawio.min.js not found: ${this.options.drawioJs}`,
    );
  }

  private validateResourcesDir() {
    assert(
      fs.existsSync(this.options.resourcesDir),
      `Draw.io resources directory not found: ${this.options.resourcesDir}`,
    );
  }

  private validateResourceManifest() {
    assert(
      fs.existsSync(this.options.resourceManifest),
      `Draw.io resource manifest not found: ${this.options.resourceManifest}`,
    );
  }
}

new DrawioDiagramUpdate(CliOptions.parse(process.argv.slice(2))).run().catch((error: Error) => {
  console.error(error.message);
  process.exit(1);
});

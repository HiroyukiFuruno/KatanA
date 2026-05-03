import fs from "node:fs";
import path from "node:path";
import { DiagramTheme, type DiagramThemeName } from "./diagram_theme";
import { PlaywrightLoader } from "./official-renderer";
import type { BrowserHandle, FontReadyDocument, PageHandle } from "./official-renderer-types";

interface CliParsedOptions {
  inputDir: string;
  outputDir: string;
  theme: DiagramThemeName;
}

class CliOptions {
  static parse(argv: string[]): CliParsedOptions {
    CliOptions.exitIfHelp(argv);
    return {
      inputDir: path.resolve(CliOptions.get(argv, "--input", "tmp/mermaid-sample-official")),
      outputDir: path.resolve(CliOptions.get(argv, "--output", "tmp/mermaid-sample-browser")),
      theme: DiagramTheme.parse(CliOptions.get(argv, "--theme", "dark")).name,
    };
  }

  private static get(argv: string[], name: string, fallback: string): string {
    const index = argv.indexOf(name);
    return index >= 0 ? argv[index + 1] : fallback;
  }

  private static exitIfHelp(argv: string[]) {
    if (argv.includes("--help")) {
      console.log(
        "Usage: bun run scripts/mermaid/rasterize-svg-dir.ts --input DIR --output DIR [--theme dark|light]",
      );
      process.exit(0);
    }
  }
}

class SvgFixtureRepository {
  constructor(private inputDir: string) {}

  list(): string[] {
    return fs
      .readdirSync(this.inputDir)
      .filter((fileName) => fileName.endsWith(".svg"))
      .sort();
  }

  read(fileName: string): string {
    return fs.readFileSync(path.join(this.inputDir, fileName), "utf8");
  }
}

class SvgBrowserRasterizer {
  private browser: BrowserHandle | null = null;
  private theme: DiagramTheme;

  constructor(private options: CliParsedOptions) {
    this.theme = DiagramTheme.parse(options.theme);
  }

  async run() {
    this.prepareOutputDir();
    await this.start();
    try {
      await this.rasterizeAll(new SvgFixtureRepository(this.options.inputDir));
    } finally {
      await this.stop();
    }
  }

  private prepareOutputDir() {
    fs.rmSync(this.options.outputDir, { recursive: true, force: true });
    fs.mkdirSync(this.options.outputDir, { recursive: true });
  }

  private async start() {
    const { chromium } = await new PlaywrightLoader().load();
    this.browser = await chromium.launch({ headless: true });
  }

  private async stop() {
    await this.browser?.close();
  }

  private async rasterizeAll(repository: SvgFixtureRepository) {
    for (const fileName of repository.list()) {
      await this.rasterize(fileName, repository.read(fileName));
    }
  }

  private async rasterize(fileName: string, svg: string) {
    const page = await this.currentBrowser().newPage({
      viewport: { width: 1520, height: 845 },
      deviceScaleFactor: 1,
    });
    try {
      await page.setContent(this.baseHtml(), { waitUntil: "load" });
      await this.capture(page, fileName, svg);
      console.log(`rasterized ${fileName}`);
    } finally {
      await page.close();
    }
  }

  private async capture(page: PageHandle, fileName: string, svg: string) {
    await page.evaluate(
      (input) => {
        const diagramElement = document.getElementById("diagram") as HTMLElement;
        diagramElement.innerHTML = input.svg;
      },
      { svg },
    );
    await SvgCaptureSizer.resize(page);
    await page.evaluate(() => (document as FontReadyDocument).fonts.ready);
    await page.locator("#capture").screenshot({
      path: path.join(this.options.outputDir, fileName.replace(/\.svg$/, ".png")),
      omitBackground: false,
    });
  }

  private currentBrowser(): BrowserHandle {
    if (this.browser === null) {
      throw new Error("Browser rasterizer is not started");
    }
    return this.browser;
  }

  private baseHtml() {
    return `<!doctype html><html><head><meta charset="utf-8"><style>
html,body{margin:0;background:${this.theme.canvasBackground};color:${this.theme.text};color-scheme:${this.theme.colorScheme()};font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif}
#capture{display:flex;align-items:center;justify-content:center;overflow:hidden;padding:12px;box-sizing:border-box;background:${this.theme.canvasBackground}}
#diagram{max-width:100%;max-height:100%;display:flex;align-items:center;justify-content:center}
#diagram svg{max-width:100%;max-height:100%}
</style></head><body><div id="capture"><div id="diagram"></div></div></body></html>`;
  }
}

class SvgCaptureSizer {
  static resize(page: PageHandle): Promise<void> {
    return page.evaluate(() => {
      const svgElement = document.querySelector("#diagram svg") as SVGSVGElement;
      const viewBox = String(svgElement.getAttribute("viewBox"))
        .split(/\s+/)
        .map((value) => Number(value));
      const width = Math.ceil(viewBox[2]);
      const height = Math.ceil(viewBox[3]);
      svgElement.setAttribute("width", String(width));
      svgElement.setAttribute("height", String(height));
      svgElement.style.maxWidth = `${width}px`;
      const capture = document.getElementById("capture") as HTMLElement;
      capture.style.width = `${width + 24}px`;
      capture.style.height = `${height + 24}px`;
    });
  }
}

new SvgBrowserRasterizer(CliOptions.parse(process.argv.slice(2))).run().catch((error: Error) => {
  console.error(error.message);
  process.exit(1);
});

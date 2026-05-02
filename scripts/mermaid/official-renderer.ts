import { execSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import type {
  BrowserHandle,
  FontReadyDocument,
  MermaidWindow,
  PageHandle,
  PlaywrightModule,
} from "./official-renderer-types";

const OFFICIAL_MERMAID_CONFIG = {
  startOnLoad: false,
  securityLevel: "loose",
  htmlLabels: false,
  theme: "dark",
  flowchart: { htmlLabels: false, useMaxWidth: false },
  sequence: { useMaxWidth: false },
  themeVariables: {
    background: "transparent",
    mainBkg: "#2D2D2D",
    primaryColor: "#2D2D2D",
    primaryTextColor: "#E0E0E0",
    primaryBorderColor: "#888888",
    secondaryColor: "#2D2D2D",
    secondaryTextColor: "#E0E0E0",
    secondaryBorderColor: "#888888",
    tertiaryColor: "#2D2D2D",
    tertiaryTextColor: "#E0E0E0",
    tertiaryBorderColor: "#888888",
    nodeTextColor: "#E0E0E0",
    lineColor: "#AAAAAA",
    textColor: "#E0E0E0",
    edgeLabelBackground: "#2D2D2D",
    actorBkg: "#2D2D2D",
    actorTextColor: "#E0E0E0",
    actorBorder: "#888888",
    signalColor: "#AAAAAA",
    signalTextColor: "#E0E0E0",
    labelTextColor: "#E0E0E0",
    noteBkgColor: "#2D2D2D",
    noteTextColor: "#E0E0E0",
    noteBorderColor: "#888888",
    clusterBkg: "transparent",
    clusterBorder: "#888888",
    titleColor: "#E0E0E0",
  },
} as const;

export class PlaywrightLoader {
  async load(): Promise<PlaywrightModule> {
    try {
      return (await import("playwright")) as PlaywrightModule;
    } catch {
      const cliPath = this.findPlaywrightCli();
      const realPath = fs.realpathSync(cliPath);
      const modulePath = path.join(path.dirname(realPath), "index.mjs");
      return import(pathToFileURL(modulePath).href) as Promise<PlaywrightModule>;
    }
  }

  findPlaywrightCli() {
    try {
      return execSync("which playwright", { encoding: "utf8" }).trim();
    } catch {
      throw new Error(
        "playwright command not found. Install it, then run `make mermaid-diagram-browser-install`.",
      );
    }
  }
}

export interface RenderFixture {
  slug: string;
  source: string;
  title: string;
}

export interface RendererOptions {
  outputDir: string;
  mermaidJs: string;
}

export class OfficialMermaidRenderer {
  private browser: BrowserHandle | null = null;
  private options: RendererOptions;

  constructor(options: RendererOptions) {
    this.options = options;
  }

  async start() {
    const { chromium } = await new PlaywrightLoader().load();
    this.browser = await chromium
      .launch({ headless: true })
      .catch((error) => this.rethrowLaunchError(error));
  }

  async stop() {
    await this.browser?.close();
  }

  async render(fixture: RenderFixture) {
    const page = await this.newPage();
    try {
      await this.renderPage(page, fixture);
    } finally {
      await page.close();
    }
  }

  private rethrowLaunchError(error: unknown): never {
    if (String(error).includes("Executable doesn't exist")) {
      throw new Error(
        "Playwright browser is missing. Run `make mermaid-diagram-browser-install` first.",
      );
    }
    throw error;
  }

  private newPage(): Promise<PageHandle> {
    return this.currentBrowser().newPage({
      viewport: { width: 1520, height: 845 },
      deviceScaleFactor: 1,
    });
  }

  private async renderPage(page: PageHandle, fixture: RenderFixture) {
    await page.setContent(this.baseHtml(), { waitUntil: "load" });
    await this.installDeterministicRandom(page);
    await page.addScriptTag({ path: this.options.mermaidJs });
    await this.capture(page, fixture, await this.renderSvg(page, fixture));
  }

  private installDeterministicRandom(page: PageHandle): Promise<void> {
    return page.evaluate(() => {
      let state = 0x12345678;
      Math.random = () => {
        state = (1664525 * state + 1013904223) >>> 0;
        return state / 0x100000000;
      };
    });
  }

  private async capture(page: PageHandle, fixture: RenderFixture, svg: string) {
    this.writeSvg(fixture, svg);
    await this.resizeCapture(page);
    await page.evaluate(() => (document as FontReadyDocument).fonts.ready);
    await page.locator("#capture").screenshot({
      path: path.join(this.options.outputDir, `${fixture.slug}.png`),
      omitBackground: false,
    });
  }

  private renderSvg(page: PageHandle, fixture: RenderFixture): Promise<string> {
    return page.evaluate(
      ({ config, input }) => {
        const mermaidValue = (window as MermaidWindow).mermaid;
        mermaidValue.initialize(config);
        return mermaidValue.render(`official-${input.slug}`, input.source).then((result) => {
          const diagramElement = document.getElementById("diagram") as HTMLElement;
          diagramElement.innerHTML = result.svg;
          return result.svg;
        });
      },
      { config: OFFICIAL_MERMAID_CONFIG, input: fixture },
    );
  }

  private writeSvg(fixture: RenderFixture, svg: string) {
    fs.writeFileSync(path.join(this.options.outputDir, `${fixture.slug}.svg`), svg, "utf8");
  }

  private resizeCapture(page: PageHandle): Promise<void> {
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

  private currentBrowser(): BrowserHandle {
    if (this.browser === null) {
      throw new Error("Official Mermaid renderer is not started");
    }
    return this.browser;
  }

  baseHtml() {
    return `<!doctype html><html><head><meta charset="utf-8"><style>
html,body{margin:0;background:#1e1e1e;color:#e0e0e0;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif}
#capture{display:flex;align-items:center;justify-content:center;overflow:hidden;padding:12px;box-sizing:border-box;background:#1e1e1e}
#diagram{max-width:100%;max-height:100%;display:flex;align-items:center;justify-content:center}
#diagram svg{max-width:100%;max-height:100%}
</style></head><body><div id="capture"><div id="diagram"></div></div></body></html>`;
  }
}

export function expandHome(value: string) {
  return value.startsWith("~/") ? path.join(os.homedir(), value.slice(2)) : value;
}

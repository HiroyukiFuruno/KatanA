import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { DiagramTheme, type MermaidThemeName, type DiagramThemeName } from "./diagram_theme";
import { OfficialSourceNormalizer } from "./official-source-normalizer";
import { OfficialRendererDeterminism } from "./official-renderer-determinism";
import { MermaidRuntimeScripts } from "./official-renderer-i18n";
import type {
  BrowserHandle,
  FontReadyDocument,
  MermaidConfig,
  MermaidI18nWindow,
  PageHandle,
  PlaywrightModule,
} from "./official-renderer-types";

export { expandHome } from "./official-renderer-i18n";

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
        "playwright command not found. Install it, then run `just mermaid-diagram-browser-install`.",
      );
    }
  }
}

export interface RenderFixture {
  slug: string;
  source: string;
  title: string;
}

interface MermaidNormalizationRequest {
  theme: MermaidThemeName;
  text: string;
  fill: string;
  stroke: string;
  arrow: string;
  source: string;
  background: string;
}

export interface RendererOptions {
  outputDir: string;
  mermaidJs: string;
  theme: DiagramThemeName;
}

export class OfficialMermaidRenderer {
  private browser: BrowserHandle | null = null;
  private options: RendererOptions;
  private theme: DiagramTheme;

  constructor(options: RendererOptions) {
    this.options = options;
    this.theme = DiagramTheme.parse(options.theme);
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

  private rethrowLaunchError(error: Error): never {
    if (error.message.includes("Executable doesn't exist")) {
      throw new Error(
        "Playwright browser is missing. Run `just mermaid-diagram-browser-install` first.",
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
    await OfficialRendererDeterminism.install(page);
    await this.installRuntime(page);
    await page.addScriptTag({ path: this.options.mermaidJs });
    await this.capture(page, fixture, await this.renderSvg(page, fixture));
  }

  private async installRuntime(page: PageHandle) {
    for (const scriptPath of MermaidRuntimeScripts.paths()) {
      await page.addScriptTag({ path: scriptPath });
    }
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
    const input = {
      ...fixture,
      source: OfficialSourceNormalizer.normalize(fixture.source),
    };
    return page.evaluate(
      ({ config, input, request }) => {
        const i18nWindow = window as MermaidI18nWindow;
        const normalized = i18nWindow.katanaNormalizeMermaidSourceI18n(input.source);
        const mermaidValue = i18nWindow.mermaid;
        mermaidValue.initialize(config);
        return mermaidValue.render(`official-${input.slug}`, normalized.source).then((result) => {
          const restored = i18nWindow.katanaRestoreMermaidI18nText(
            result.svg,
            normalized.replacements,
          );
          const normalizedSvg = i18nWindow.katanaNormalizeMermaidSvg(restored, {
            ...request,
            source: normalized.source,
          });
          const diagramElement = document.getElementById("diagram") as HTMLElement;
          diagramElement.innerHTML = normalizedSvg;
          return normalizedSvg;
        });
      },
      {
        config: this.mermaidConfig(),
        input,
        request: this.normalizationRequest(),
      },
    );
  }

  private normalizationRequest(): MermaidNormalizationRequest {
    return {
      theme: this.theme.mermaidTheme,
      text: this.theme.text,
      fill: this.theme.getFill(),
      stroke: this.theme.getStroke(),
      arrow: this.theme.getArrow(),
      background: "transparent",
      source: "",
    };
  }

  private mermaidConfig(): MermaidConfig {
    return {
      startOnLoad: false,
      securityLevel: "loose",
      htmlLabels: false,
      theme: this.theme.mermaidTheme,
      flowchart: { htmlLabels: false, useMaxWidth: false },
      sequence: { useMaxWidth: false },
      themeVariables: this.theme.variables(),
    };
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
html,body{margin:0;background:${this.theme.canvasBackground};color:${this.theme.text};color-scheme:${this.theme.colorScheme()};font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif}
#capture{display:flex;align-items:center;justify-content:center;overflow:hidden;padding:12px;box-sizing:border-box;background:${this.theme.canvasBackground}}
#diagram{max-width:100%;max-height:100%;display:flex;align-items:center;justify-content:center}
#diagram svg{max-width:100%;max-height:100%}
</style></head><body><div id="capture"><div id="diagram"></div></div></body></html>`;
  }
}

import fs from "node:fs";
import path from "node:path";
import { expandHome, PlaywrightLoader } from "../mermaid/official-renderer";
import type {
  BrowserHandle,
  FontReadyDocument,
  PageHandle,
} from "../mermaid/official-renderer-types";
import { OfficialDrawioResourceResolver } from "./official-resource-resolver";

export interface DrawioRendererOptions {
  outputDir: string;
  drawioJs: string;
  resourcesDir: string;
  resourceManifest: string;
}

export interface DrawioRenderFixture {
  slug: string;
  source: string;
}

interface DrawioGraph {
  getSvg(): SVGSVGElement;
}

interface DrawioViewer {
  graph: DrawioGraph;
}

interface DrawioGraphViewer {
  createViewerForElement(element: HTMLElement, callback: (viewer: DrawioViewer) => void): void;
}

interface DrawioWindow extends Window {
  GraphViewer: DrawioGraphViewer;
}

export class OfficialDrawioRenderer {
  private browser: BrowserHandle | null = null;
  private readonly resourceResolver: OfficialDrawioResourceResolver;

  constructor(private options: DrawioRendererOptions) {
    this.resourceResolver = new OfficialDrawioResourceResolver(
      options.resourcesDir,
      options.resourceManifest,
    );
  }

  async start() {
    const { chromium } = await new PlaywrightLoader().load();
    this.browser = await chromium
      .launch({ headless: true })
      .catch((error: Error) => this.rethrowLaunchError(error));
  }

  async stop() {
    await this.browser?.close();
  }

  async render(fixture: DrawioRenderFixture) {
    const page = await this.currentBrowser().newPage({
      viewport: { width: 1520, height: 845 },
      deviceScaleFactor: 1,
    });
    await this.renderPage(page, fixture);
  }

  private async renderPage(page: PageHandle, fixture: DrawioRenderFixture) {
    await page.setContent(this.baseHtml(), { waitUntil: "load" });
    await page.addScriptTag({ path: this.options.drawioJs });
    const svg = this.resourceResolver.resolveSvg(await this.renderSvg(page, fixture));
    await this.capture(page, fixture, svg);
    await page.close();
  }

  private renderSvg(page: PageHandle, fixture: DrawioRenderFixture): Promise<string> {
    return page.evaluate(
      (input) =>
        new Promise<string>((resolve) => {
          const diagram = document.getElementById("diagram") as HTMLElement;
          const container = document.createElement("div");
          container.setAttribute("class", "mxgraph");
          container.setAttribute(
            "data-mxgraph",
            JSON.stringify({
              xml: input.source,
              toolbar: "",
              "auto-fit": false,
              "auto-crop": false,
              "auto-origin": true,
              "check-visible-state": false,
            }),
          );
          diagram.appendChild(container);
          (window as DrawioWindow).GraphViewer.createViewerForElement(container, (viewer) => {
            const svg = viewer.graph.getSvg();
            diagram.innerHTML = new XMLSerializer().serializeToString(svg);
            resolve(diagram.innerHTML);
          });
        }),
      {
        slug: fixture.slug,
        source: DrawioSource.prepare(fixture.source),
      },
    );
  }

  private async capture(page: PageHandle, fixture: DrawioRenderFixture, svg: string) {
    fs.writeFileSync(path.join(this.options.outputDir, `${fixture.slug}.svg`), svg, "utf8");
    await page.evaluate((markup) => {
      const diagram = document.getElementById("diagram") as HTMLElement;
      diagram.innerHTML = markup;
    }, svg);
    await this.waitForSvgImages(page);
    await this.resizeCapture(page);
    await page.evaluate(() => (document as FontReadyDocument).fonts.ready);
    await page.locator("#capture").screenshot({
      path: path.join(this.options.outputDir, `${fixture.slug}.png`),
      omitBackground: false,
    });
  }

  private waitForSvgImages(page: PageHandle): Promise<void> {
    return page.evaluate(() =>
      Promise.all(
        Array.from(document.querySelectorAll("#diagram image")).map(
          (element) =>
            new Promise<void>((resolve) => {
              const href = element.getAttribute("href") ?? element.getAttribute("xlink:href") ?? "";
              const image = new Image();
              image.onload = () => resolve();
              image.onerror = () => resolve();
              image.src = href;
            }),
        ),
      ).then(() => undefined),
    );
  }

  private resizeCapture(page: PageHandle): Promise<void> {
    return page.evaluate(() => {
      const svg = document.querySelector("#diagram svg") as SVGSVGElement;
      const viewBox = svg.viewBox.baseVal;
      const box = svg.getBoundingClientRect();
      const width = Math.ceil([viewBox.width, box.width, 1].find((size) => size > 0));
      const height = Math.ceil([viewBox.height, box.height, 1].find((size) => size > 0));
      svg.setAttribute("width", String(width));
      svg.setAttribute("height", String(height));
      const capture = document.getElementById("capture") as HTMLElement;
      capture.style.width = `${width + 24}px`;
      capture.style.height = `${height + 24}px`;
    });
  }

  private rethrowLaunchError(error: Error): never {
    if (String(error).includes("Executable doesn't exist")) {
      throw new Error(
        "Playwright browser is missing. Run `just drawio-diagram-browser-install` first.",
      );
    }
    throw error;
  }

  private currentBrowser(): BrowserHandle {
    if (this.browser === null) {
      throw new Error("Official Draw.io renderer is not started");
    }
    return this.browser;
  }

  private baseHtml() {
    return `<!doctype html><html><head><meta charset="utf-8"><style>
html,body{margin:0;background:#1e1e1e;color:#e0e0e0;color-scheme:dark;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif}
#capture{display:flex;align-items:center;justify-content:center;overflow:hidden;padding:12px;box-sizing:border-box;background:#1e1e1e}
#diagram{max-width:100%;max-height:100%;display:flex;align-items:center;justify-content:center}
#diagram svg{max-width:100%;max-height:100%}
</style></head><body><div id="capture"><div id="diagram"></div></div></body></html>`;
  }
}

class DrawioSource {
  static prepare(source: string): string {
    const xml = source.trim().replace(/^<\?xml[^>]*>\s*/i, "");
    return DrawioSource.uncompressedModel(xml) ?? xml;
  }

  private static uncompressedModel(xml: string): string | null {
    const start = xml.indexOf("<mxGraphModel");
    const end = xml.indexOf("</mxGraphModel>");
    if (DrawioSource.hasModelRange(start, end)) {
      return xml.slice(start, end + "</mxGraphModel>".length);
    }
    return null;
  }

  private static hasModelRange(start: number, end: number): boolean {
    return [start >= 0, end >= start].every(Boolean);
  }
}

export { expandHome };

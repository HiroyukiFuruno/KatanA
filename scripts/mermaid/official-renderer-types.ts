import type { MermaidThemeName, MermaidThemeVariables } from "./diagram_theme";

export interface BrowserLauncher {
  launch(options: { headless: boolean }): Promise<BrowserHandle>;
}

export interface PlaywrightModule {
  chromium: BrowserLauncher;
}

export interface BrowserHandle {
  newPage(options: {
    viewport: { width: number; height: number };
    deviceScaleFactor: number;
  }): Promise<PageHandle>;
  close(): Promise<void>;
}

export interface PageHandle {
  setContent(html: string, options: { waitUntil: "load" }): Promise<void>;
  addScriptTag(options: { path: string }): Promise<void>;
  evaluate<Result>(callback: () => Result | Promise<Result>): Promise<Result>;
  evaluate<Result, Argument>(
    callback: (argument: Argument) => Result | Promise<Result>,
    argument: Argument,
  ): Promise<Result>;
  locator(selector: string): LocatorHandle;
  close(): Promise<void>;
}

export interface LocatorHandle {
  screenshot(options: { path: string; omitBackground: boolean }): Promise<void>;
}

export interface MermaidConfig {
  startOnLoad: boolean;
  securityLevel: "loose";
  htmlLabels: boolean;
  theme: MermaidThemeName;
  flowchart: { htmlLabels: boolean; useMaxWidth: boolean };
  sequence: { useMaxWidth: boolean };
  themeVariables: MermaidThemeVariables;
}

export interface MermaidRenderResult {
  svg: string;
}

export interface MermaidGlobal {
  initialize(config: MermaidConfig): void;
  render(id: string, source: string): Promise<MermaidRenderResult>;
}

export interface MermaidWindow extends Window {
  mermaid: MermaidGlobal;
}

export interface MermaidI18nReplacement {
  placeholder: string;
  text: string;
}

export interface MermaidI18nNormalizeResult {
  source: string;
  replacements: MermaidI18nReplacement[];
}

export interface MermaidI18nWindow extends MermaidWindow {
  katanaNormalizeMermaidSourceI18n(source: string): MermaidI18nNormalizeResult;
  katanaRestoreMermaidI18nText(svg: string, replacements: MermaidI18nReplacement[]): string;
  katanaNormalizeMermaidSvg(
    svg: string,
    request: {
      theme: "dark" | "default";
      text: string;
      fill: string;
      stroke: string;
      arrow: string;
      source: string;
      background: string;
    },
  ): string;
}

export interface FontReadyDocument extends Document {
  fonts: { ready: Promise<void> };
}

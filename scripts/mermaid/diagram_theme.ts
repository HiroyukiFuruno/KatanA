import assert from "node:assert/strict";

export type DiagramThemeName = "dark" | "light";
export type MermaidThemeName = "dark" | "default";

export interface MermaidThemeVariables {
  background: string;
  mainBkg: string;
  primaryColor: string;
  primaryTextColor: string;
  primaryBorderColor: string;
  secondaryColor: string;
  secondaryTextColor: string;
  secondaryBorderColor: string;
  tertiaryColor: string;
  tertiaryTextColor: string;
  tertiaryBorderColor: string;
  nodeTextColor: string;
  lineColor: string;
  textColor: string;
  edgeLabelBackground: string;
  actorBkg: string;
  actorTextColor: string;
  actorBorder: string;
  signalColor: string;
  signalTextColor: string;
  labelTextColor: string;
  noteBkgColor: string;
  noteTextColor: string;
  noteBorderColor: string;
  clusterBkg: string;
  clusterBorder: string;
  titleColor: string;
}

export class DiagramTheme {
  static parse(value: string): DiagramTheme {
    const theme = DiagramTheme.byName().get(value);
    assert(theme, `theme must be light or dark: ${value}`);
    return theme;
  }

  private static byName(): Map<string, DiagramTheme> {
    return new Map([
      ["dark", DiagramTheme.dark()],
      ["light", DiagramTheme.light()],
    ]);
  }

  private static dark(): DiagramTheme {
    return new DiagramTheme("dark", "dark", "#1e1e1e", "#e0e0e0", "#2D2D2D", "#888888", "#AAAAAA");
  }

  private static light(): DiagramTheme {
    return new DiagramTheme(
      "light",
      "default",
      "#ffffff",
      "#333333",
      "#fff2cc",
      "#d6b656",
      "#555555",
    );
  }

  private constructor(
    public name: DiagramThemeName,
    public mermaidTheme: MermaidThemeName,
    public canvasBackground: string,
    public text: string,
    private fill: string,
    private stroke: string,
    private arrow: string,
  ) {}

  getFill(): string {
    return this.fill;
  }

  getStroke(): string {
    return this.stroke;
  }

  getArrow(): string {
    return this.arrow;
  }

  colorScheme(): string {
    return this.name;
  }

  variables(): MermaidThemeVariables {
    return {
      background: "transparent",
      mainBkg: this.fill,
      primaryColor: this.fill,
      primaryTextColor: this.text,
      primaryBorderColor: this.stroke,
      secondaryColor: this.fill,
      secondaryTextColor: this.text,
      secondaryBorderColor: this.stroke,
      tertiaryColor: this.fill,
      tertiaryTextColor: this.text,
      tertiaryBorderColor: this.stroke,
      nodeTextColor: this.text,
      lineColor: this.arrow,
      textColor: this.text,
      edgeLabelBackground: this.fill,
      actorBkg: this.fill,
      actorTextColor: this.text,
      actorBorder: this.stroke,
      signalColor: this.arrow,
      signalTextColor: this.text,
      labelTextColor: this.text,
      noteBkgColor: this.fill,
      noteTextColor: this.text,
      noteBorderColor: this.stroke,
      clusterBkg: "transparent",
      clusterBorder: this.stroke,
      titleColor: this.text,
    };
  }
}

import path from "node:path";
import { DiagramTheme, type DiagramThemeName } from "./diagram_theme";

export interface CliParsedOptions {
  officialDir: string;
  katanaDir: string;
  outputDir: string;
  katanaCrop: CropRect | null;
  baselineScoresPath: string | null;
  minScore: number;
  theme: DiagramThemeName;
}

export class CliOptions {
  static parse(argv: string[]): CliParsedOptions {
    return {
      officialDir: path.resolve(
        CliOptions.get(argv, "--official", "tmp/mermaid-parts-en-dark-official-browser"),
      ),
      katanaDir: path.resolve(
        CliOptions.get(argv, "--katana", "tmp/mermaid-parts-en-dark-katana-browser"),
      ),
      outputDir: path.resolve(
        CliOptions.get(argv, "--output", "tmp/mermaid-parts-en-dark-comparison"),
      ),
      katanaCrop: CropRect.parseOptional(CliOptions.get(argv, "--katana-crop", "none")),
      baselineScoresPath: CliOptions.getOptionalPath(argv, "--baseline"),
      minScore: CliOptions.number(argv, "--min-score", 99),
      theme: DiagramTheme.parse(CliOptions.get(argv, "--theme", "dark")).name,
    };
  }

  private static get(argv: string[], name: string, fallback: string): string {
    const index = argv.indexOf(name);
    return index >= 0 && index + 1 < argv.length ? argv[index + 1] : fallback;
  }

  private static getOptionalPath(argv: string[], name: string): string | null {
    const index = argv.indexOf(name);
    return index >= 0 && index + 1 < argv.length ? argv[index + 1] : null;
  }

  private static number(argv: string[], name: string, fallback: number): number {
    const value = Number(CliOptions.get(argv, name, String(fallback)));
    if (!Number.isFinite(value)) {
      throw new Error(`Invalid number option: ${name}`);
    }
    return value;
  }
}

export class CropRect {
  static parseOptional(value: string): CropRect | null {
    return value === "none" ? null : CropRect.parse(value);
  }

  static parse(value: string): CropRect {
    const parts = value.split(",").map((it) => Number.parseInt(it, 10));
    if (CropRect.isInvalidParts(parts)) {
      throw new Error(`Invalid crop rect: ${value}`);
    }
    return new CropRect(parts[0], parts[1], parts[2], parts[3]);
  }

  private static isInvalidParts(parts: number[]): boolean {
    return [parts.length !== 4, parts.some((it) => Number.isNaN(it))].includes(true);
  }

  constructor(
    public x: number,
    public y: number,
    public width: number,
    public height: number,
  ) {}

  toMagickArg(): string {
    return `${this.width}x${this.height}+${this.x}+${this.y}`;
  }
}

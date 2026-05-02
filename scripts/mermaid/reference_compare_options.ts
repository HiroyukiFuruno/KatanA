import path from "node:path";

export interface CliParsedOptions {
  officialDir: string;
  katanaDir: string;
  outputDir: string;
  katanaCrop: CropRect | null;
  minScore: number;
}

export class CliOptions {
  static parse(argv: string[]): CliParsedOptions {
    return {
      officialDir: path.resolve(
        CliOptions.get(argv, "--official", "assets/fixtures/mermaid_all/official"),
      ),
      katanaDir: path.resolve(CliOptions.get(argv, "--katana", "tmp/mermaid-katana-rendered")),
      outputDir: path.resolve(CliOptions.get(argv, "--output", "tmp/mermaid-official-comparison")),
      katanaCrop: CropRect.parseOptional(CliOptions.get(argv, "--katana-crop", "none")),
      minScore: CliOptions.number(argv, "--min-score", 99),
    };
  }

  private static get(argv: string[], name: string, fallback: string): string {
    const index = argv.indexOf(name);
    return index >= 0 ? argv[index + 1] : fallback;
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

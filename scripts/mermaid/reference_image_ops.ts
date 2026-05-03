import { execFileSync, spawnSync, type SpawnSyncReturns } from "node:child_process";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { DiagramTheme, type DiagramThemeName } from "./diagram_theme";
import type { CropRect } from "./reference_compare_options";
import type { ReferencePair } from "./reference_pair_repository";

export interface NormalizedPair {
  pair: ReferencePair;
  officialCanvasPath: string;
  katanaCanvasPath: string;
  officialContentPath: string;
  katanaContentPath: string;
  pairImagePath: string;
}

export class MagickOps {
  private workDir: string;
  private theme: DiagramTheme;

  constructor(
    private outputDir: string,
    private crop: CropRect | null,
    themeName: DiagramThemeName,
  ) {
    this.workDir = path.join(outputDir, "work");
    this.theme = DiagramTheme.parse(themeName);
  }

  prepare() {
    fs.rmSync(this.outputDir, { recursive: true, force: true });
    fs.mkdirSync(path.join(this.outputDir, "pairs"), { recursive: true });
    fs.mkdirSync(this.workDir, { recursive: true });
  }

  renderPair(pair: ReferencePair): NormalizedPair {
    const normalized = this.normalizedPair(pair);
    this.normalizeOfficial(pair.officialPath, normalized.officialCanvasPath);
    this.normalizeKatana(pair.katanaPath, normalized.katanaCanvasPath);
    this.normalizeContent(normalized.officialCanvasPath, normalized.officialContentPath);
    this.normalizeContent(normalized.katanaCanvasPath, normalized.katanaContentPath);
    this.magick([
      normalized.officialCanvasPath,
      normalized.katanaCanvasPath,
      "+append",
      normalized.pairImagePath,
    ]);
    return normalized;
  }

  renderContactSheet(pairImages: string[]): string {
    const rows: string[] = [];
    for (let index = 0; index < pairImages.length; index += 2) {
      rows.push(this.renderContactRow(pairImages, index, rows.length));
    }
    const output = path.join(this.outputDir, "contact-sheet.png");
    this.magick([...rows, "-background", this.theme.canvasBackground, "-append", output]);
    return output;
  }

  compareNormalizedError(metric: string, left: string, right: string): number {
    const result = spawnSync("magick", ["compare", "-metric", metric, left, right, "null:"], {
      encoding: "utf8",
    });
    return new ImageMetricResult(result, metric).normalizedError();
  }

  imageSize(input: string): ImageSize {
    const result = execFileSync("magick", ["identify", "-format", "%w %h", input], {
      encoding: "utf8",
    });
    return ImageSize.parse(result);
  }

  private normalizedPair(pair: ReferencePair): NormalizedPair {
    return {
      pair,
      officialCanvasPath: path.join(this.workDir, `${pair.slug}-official.png`),
      katanaCanvasPath: path.join(this.workDir, `${pair.slug}-katana.png`),
      officialContentPath: path.join(this.workDir, `${pair.slug}-official-content.png`),
      katanaContentPath: path.join(this.workDir, `${pair.slug}-katana-content.png`),
      pairImagePath: path.join(this.outputDir, "pairs", `${pair.slug}.png`),
    };
  }

  private normalizeOfficial(input: string, output: string) {
    this.normalizeCanvas(input, [], output);
  }

  private normalizeKatana(input: string, output: string) {
    const cropArgs = this.crop === null ? [] : ["-crop", this.crop.toMagickArg(), "+repage"];
    this.normalizeCanvas(input, cropArgs, output);
  }

  private normalizeCanvas(input: string, preArgs: string[], output: string) {
    this.magick([input, ...preArgs, ...this.canvasArgs(), output]);
  }

  private normalizeContent(input: string, output: string) {
    this.magick([input, "-fuzz", "4%", "-trim", "+repage", ...this.canvasArgs(), output]);
  }

  private canvasArgs(): string[] {
    return [
      "-resize",
      "760x423",
      "-background",
      this.theme.canvasBackground,
      "-gravity",
      "center",
      "-extent",
      "760x423",
    ];
  }

  private renderContactRow(pairImages: string[], index: number, rowIndex: number): string {
    const row = path.join(this.workDir, `row-${String(rowIndex).padStart(2, "0")}.png`);
    this.magick([
      pairImages[index],
      this.secondImage(pairImages, index),
      "-background",
      this.theme.canvasBackground,
      "+append",
      row,
    ]);
    return row;
  }

  private secondImage(pairImages: string[], index: number): string {
    return pairImages[index + 1] ?? pairImages[index];
  }

  private magick(args: string[]) {
    execFileSync("magick", args, { stdio: "inherit" });
  }
}

export class ImageSize {
  static parse(value: string): ImageSize {
    const [width, height] = value.trim().split(/\s+/).map(Number);
    assert(Number.isFinite(width), `ImageMagick identify parse failed: ${value.trim()}`);
    assert(Number.isFinite(height), `ImageMagick identify parse failed: ${value.trim()}`);
    return new ImageSize(width, height);
  }

  constructor(
    public readonly width: number,
    public readonly height: number,
  ) {}

  similarityTo(other: ImageSize): number {
    return Math.min(
      ImageSize.axisSimilarity(this.width, other.width),
      ImageSize.axisSimilarity(this.height, other.height),
    );
  }

  coverageBy(candidate: ImageSize): number {
    return Math.min(
      ImageSize.coverageAxisSimilarity(this.width, candidate.width),
      ImageSize.coverageAxisSimilarity(this.height, candidate.height),
    );
  }

  private static axisSimilarity(left: number, right: number): number {
    return Math.min(left, right) / Math.max(left, right);
  }

  private static coverageAxisSimilarity(expected: number, actual: number): number {
    return Math.min(1, actual / expected);
  }
}

class ImageMetric {
  static parseNormalizedError(value: string): number {
    const match = value.match(/\(([-+]?\d*\.?\d+(?:e[-+]?\d+)?)\)/i);
    if (!match) {
      throw new Error(`ImageMagick metric parse failed: ${value.trim()}`);
    }
    return Number(match[1]);
  }
}

class ImageMetricResult {
  constructor(
    private result: SpawnSyncReturns<string>,
    private metric: string,
  ) {}

  normalizedError(): number {
    this.throwIfFailed();
    return ImageMetric.parseNormalizedError(this.metricOutput());
  }

  private throwIfFailed() {
    if (!this.acceptedStatus()) {
      throw new Error(this.failureMessage());
    }
  }

  private acceptedStatus(): boolean {
    return [0, 1].includes(Number(this.result.status));
  }

  private failureMessage(): string {
    return (
      [this.result.stderr.trim(), `ImageMagick compare failed: ${this.metric}`].find(
        (it) => it.length > 0,
      ) ?? `ImageMagick compare failed: ${this.metric}`
    );
  }

  private metricOutput(): string {
    return [this.result.stderr, this.result.stdout].find((it) => it.length > 0) ?? "";
  }
}

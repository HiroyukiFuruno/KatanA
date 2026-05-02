import { execFileSync, spawnSync, type SpawnSyncReturns } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
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

  constructor(
    private outputDir: string,
    private crop: CropRect | null,
  ) {
    this.workDir = path.join(outputDir, "work");
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
    this.magick([...rows, "-background", "#1e1e1e", "-append", output]);
    return output;
  }

  compareNormalizedError(metric: string, left: string, right: string): number {
    const result = spawnSync("magick", ["compare", "-metric", metric, left, right, "null:"], {
      encoding: "utf8",
    });
    return new ImageMetricResult(result, metric).normalizedError();
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
      "#1e1e1e",
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
      "#1e1e1e",
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

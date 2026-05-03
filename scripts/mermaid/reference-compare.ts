import fs from "node:fs";
import { CliOptions, type CliParsedOptions } from "./reference_compare_options";
import { MagickOps } from "./reference_image_ops";
import { ReferencePairRepository } from "./reference_pair_repository";
import { ReferenceCompareReport } from "./reference_report";
import { type ReferenceScoreRow, ReferenceScorer, ReferenceScores } from "./reference_score";
import type { ReferenceScoreBaseline } from "./reference_score_policy";

class ReferenceCompare {
  constructor(private options: CliParsedOptions) {}

  run() {
    const pairs = new ReferencePairRepository(this.options).list();
    const magick = new MagickOps(
      this.options.outputDir,
      this.options.katanaCrop,
      this.options.theme,
    );
    magick.prepare();

    const normalized = pairs.map((pair) => magick.renderPair(pair));
    const baselines = this.options.baselineScoresPath
      ? this.loadBaselines(this.options.baselineScoresPath)
      : [];
    const scores = new ReferenceScorer(magick, this.options.minScore, baselines).score(normalized);
    const contactSheet = magick.renderContactSheet(normalized.map((it) => it.pairImagePath));

    new ReferenceCompareReport(this.options.outputDir).write(pairs, scores, contactSheet);
    this.printSummary(scores, contactSheet);
    process.exitCode = this.exitCode(scores);
  }

  private printSummary(scores: ReferenceScoreRow[], contactSheet: string) {
    console.log(`pairs: ${scores.length}`);
    console.log(`minimum score: ${ReferenceScores.minimum(scores).toFixed(2)}`);
    console.log(`contact: ${contactSheet}`);
  }

  private exitCode(scores: ReferenceScoreRow[]): number {
    return scores.every((score) => score.passed) ? 0 : 1;
  }

  private loadBaselines(filePath: string): ReferenceScoreBaseline[] {
    const file = fs.readFileSync(filePath, "utf8");
    const parsed: unknown = JSON.parse(file);
    if (!Array.isArray(parsed)) {
      throw new Error(`Invalid baseline score file: expected array at ${filePath}`);
    }
    return parsed.map((entry) => this.parseBaselineScore(entry, filePath));
  }

  private parseBaselineScore(entry: unknown, filePath: string): ReferenceScoreBaseline {
    const source = this.assertBaselineScoreObject(entry, filePath);
    return {
      slug: this.assertBaselineSlug(source.slug, filePath),
      score: this.assertBaselineScore(source.score, filePath),
    };
  }

  private assertBaselineScoreObject(entry: unknown, filePath: string): Record<string, unknown> {
    if (Object.prototype.toString.call(entry) !== "[object Object]") {
      throw new Error(`Invalid baseline score entry: not an object in ${filePath}`);
    }
    return entry;
  }

  private assertBaselineSlug(slug: unknown, filePath: string): string {
    if (typeof slug !== "string") {
      throw new Error(`Invalid baseline score entry in ${filePath}: ${JSON.stringify(slug)}`);
    }
    return slug;
  }

  private assertBaselineScore(score: unknown, filePath: string): number {
    if (typeof score !== "number") {
      throw new Error(`Invalid baseline score entry in ${filePath}: ${JSON.stringify(score)}`);
    }
    return score;
  }
}

new ReferenceCompare(CliOptions.parse(process.argv.slice(2))).run();

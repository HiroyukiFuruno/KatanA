import path from "node:path";
import { CropRect, type CliParsedOptions } from "../mermaid/reference_compare_options";
import { MagickOps } from "../mermaid/reference_image_ops";
import { ReferencePairRepository } from "../mermaid/reference_pair_repository";
import { ReferenceCompareReport, ReferenceCompareReportLabels } from "../mermaid/reference_report";
import {
  DrawioReferenceScorer,
  DrawioReferenceScores,
  type DrawioReferenceScoreRow,
} from "./reference-score";

class CliOptions {
  static parse(argv: string[]): CliParsedOptions {
    return {
      officialDir: path.resolve(
        CliOptions.get(argv, "--official", "assets/fixtures/drawio/basic/official"),
      ),
      katanaDir: path.resolve(CliOptions.get(argv, "--katana", "tmp/drawio-katana-rendered")),
      outputDir: path.resolve(CliOptions.get(argv, "--output", "tmp/drawio-official-comparison")),
      katanaCrop: CropRect.parseOptional(CliOptions.get(argv, "--katana-crop", "none")),
      minScore: CliOptions.number(argv, "--min-score", 99),
      theme: "dark",
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
    const scores = new DrawioReferenceScorer(magick, this.options.minScore).score(normalized);
    const contactSheet = magick.renderContactSheet(normalized.map((it) => it.pairImagePath));

    new ReferenceCompareReport(this.options.outputDir, ReferenceCompareReportLabels.drawio()).write(
      pairs,
      scores,
      contactSheet,
    );
    this.printSummary(scores, contactSheet);
    process.exitCode = this.exitCode(scores);
  }

  private printSummary(scores: DrawioReferenceScoreRow[], contactSheet: string) {
    console.log(`pairs: ${scores.length}`);
    console.log(`minimum score: ${DrawioReferenceScores.minimum(scores).toFixed(2)}`);
    console.log(`contact: ${contactSheet}`);
  }

  private exitCode(scores: DrawioReferenceScoreRow[]): number {
    return scores.every((score) => score.passed) ? 0 : 1;
  }
}

new ReferenceCompare(CliOptions.parse(process.argv.slice(2))).run();

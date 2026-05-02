import { CliOptions, type CliParsedOptions } from "./reference_compare_options";
import { MagickOps } from "./reference_image_ops";
import { ReferencePairRepository } from "./reference_pair_repository";
import { ReferenceCompareReport } from "./reference_report";
import { ReferenceScorer, ReferenceScores, type ReferenceScoreRow } from "./reference_score";

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
    const scores = new ReferenceScorer(magick, this.options.minScore).score(normalized);
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
}

new ReferenceCompare(CliOptions.parse(process.argv.slice(2))).run();

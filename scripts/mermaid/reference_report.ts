import fs from "node:fs";
import path from "node:path";
import type { ReferencePair } from "./reference_pair_repository";
import type { ReferenceScoreRow } from "./reference_score";

export class ReferenceCompareReport {
  constructor(
    private outputDir: string,
    private labels: ReferenceCompareReportLabels = ReferenceCompareReportLabels.mermaid(),
  ) {}

  write(pairs: ReferencePair[], scores: ReferenceScoreRow[], contactSheet: string) {
    const contactSheetPath = path.relative(this.outputDir, contactSheet);
    fs.writeFileSync(
      path.join(this.outputDir, "scores.json"),
      JSON.stringify(scores, null, 2),
      "utf8",
    );
    fs.writeFileSync(
      path.join(this.outputDir, "README.md"),
      this.markdown(pairs, scores, contactSheetPath),
      "utf8",
    );
  }

  private markdown(
    pairs: ReferencePair[],
    scores: ReferenceScoreRow[],
    contactSheetPath: string,
  ): string {
    return [
      this.labels.title,
      "",
      this.labels.description,
      "",
      "## 一致度スコア",
      "",
      "| 図 | 総合 | 下限 | 全体RMSE | 全体MAE | 内容RMSE | 内容MAE | 判定 | 例外理由 |",
      "| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |",
      ...scores.map((score) => this.scoreRow(score)),
      "",
      `![全体比較](${contactSheetPath})`,
      "",
      ...pairs.flatMap((pair) => this.pairSection(pair)),
      "",
    ].join("\n");
  }

  private scoreRow(score: ReferenceScoreRow): string {
    const columns = [
      score.slug,
      score.score.toFixed(2),
      score.minScore.toFixed(2),
      score.canvasRmseScore.toFixed(2),
      score.canvasMaeScore.toFixed(2),
      score.contentRmseScore.toFixed(2),
      score.contentMaeScore.toFixed(2),
      score.passed ? "OK" : "NG",
      score.exceptionReason,
    ];
    return `| ${columns.join(" | ")} |`;
  }

  private pairSection(pair: ReferencePair): string[] {
    return [`## ${pair.slug}`, "", `![${pair.slug}](pairs/${pair.slug}.png)`, ""];
  }
}

export class ReferenceCompareReportLabels {
  static mermaid(): ReferenceCompareReportLabels {
    return new ReferenceCompareReportLabels(
      "# Mermaid 公式参照比較",
      "左が公式 Mermaid.js の SVG、右が KatanA 生成 SVG です。どちらも同じブラウザー条件で画像化（rasterize）して比較しています。",
    );
  }

  static drawio(): ReferenceCompareReportLabels {
    return new ReferenceCompareReportLabels(
      "# Draw.io 公式参照比較",
      "左が公式 Draw.io JavaScript の実ブラウザー描画、右が KatanA preview のスクリーンショットです。",
    );
  }

  constructor(
    public title: string,
    public description: string,
  ) {}
}

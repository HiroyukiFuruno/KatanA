export interface ReferenceScoreThreshold {
  slug: string;
  minScore: number;
  reason: string;
}

export class ReferenceScorePolicy {
  constructor(
    private globalMinScore: number,
    private thresholds: ReferenceScoreThreshold[] = MERMAID_VISUAL_ACCEPTED_SCORE_FLOORS,
  ) {}

  thresholdFor(slug: string): ReferenceScoreThreshold {
    return (
      this.thresholds.find((threshold) => threshold.slug === slug) ?? this.defaultThreshold(slug)
    );
  }

  private defaultThreshold(slug: string): ReferenceScoreThreshold {
    return {
      slug,
      minScore: this.globalMinScore,
      reason: "",
    };
  }
}

// WHY: 2026-05-01 にユーザーが実アプリ画面で目視確認し、以下の図は現状品質で及第点と判断した。
// 99点へ寄せるには公式 Mermaid.js の内部レイアウト差や KatanA 側の可読性補正まで崩す必要があり、
// 見た目の改善量に対して改修コストとデグレードリスクが大きく、投資対効果（ROI）が悪い。
// 無条件許可ではなく、確認済みスコアを下限として固定する。
// 各値は 2026-05-01 実測スコアの小数第2位下限であり、ここから下がれば回帰として検知する。
const MERMAID_VISUAL_ACCEPTED_SCORE_FLOORS: ReferenceScoreThreshold[] = [
  visualAccepted("03-classdiagram", 87.89),
  visualAccepted("04-sequencediagram", 91.01),
  visualAccepted("05-erdiagram", 90.38),
  visualAccepted("08-mindmap", 91.17),
  visualAccepted("09-architecture-beta", 94.44),
  visualAccepted("10-block-beta", 90.55),
  visualAccepted("11-c4", 84.64),
  visualAccepted("13-gitgraph", 98.71),
  visualAccepted("14-ishikawa-beta", 97.29),
  visualAccepted("20-requirementdiagram", 90.37),
  visualAccepted("21-sankey-beta", 90.78),
  visualAccepted("22-timeline", 84.86),
  visualAccepted("23-treeview-beta", 91.1),
  visualAccepted("24-treemap-beta", 90.72),
  visualAccepted("25-journey", 97.48),
  visualAccepted("27-wardley-beta", 91.54),
];

function visualAccepted(slug: string, minScore: number): ReferenceScoreThreshold {
  return {
    slug,
    minScore,
    reason:
      "ユーザーが実アプリ画面で目視確認済み。99点化は投資対効果（ROI）が悪く、現状スコアを品質下限として固定。",
  };
}

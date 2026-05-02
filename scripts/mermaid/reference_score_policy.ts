export interface ReferenceScoreThreshold {
  slug: string;
  minScore: number;
  reason: string;
}

export interface ReferenceScoreBaseline {
  slug: string;
  score: number;
}

export class ReferenceScorePolicy {
  private baselineBySlug = new Map<string, number>();
  private baselineByIdentity = new Map<string, ReferenceScoreBaseline>();

  constructor(
    private globalMinScore: number,
    private thresholds: ReferenceScoreThreshold[] = MERMAID_VISUAL_ACCEPTED_SCORE_FLOORS,
    baselines: ReferenceScoreBaseline[] = [],
  ) {
    baselines.forEach((baseline) => this.registerBaseline(baseline));
  }

  thresholdFor(slug: string): ReferenceScoreThreshold {
    return this.localizeThreshold(slug, this.matchingThreshold(slug));
  }

  private matchingThreshold(slug: string): ReferenceScoreThreshold {
    return (
      this.exactBaselineThreshold(slug) ??
      this.identityBaselineThreshold(slug) ??
      this.exactThreshold(slug) ??
      this.prefixThreshold(slug) ??
      this.defaultThreshold(slug)
    );
  }

  private registerBaseline(baseline: ReferenceScoreBaseline) {
    if (!Number.isFinite(baseline.score)) {
      throw new Error(`Invalid baseline score for ${baseline.slug}`);
    }
    this.baselineBySlug.set(baseline.slug, baseline.score);
    const identity = ReferenceScoreSlugIdentity.from(baseline.slug);
    const existing = this.baselineByIdentity.get(identity);
    if (existing !== undefined && existing.slug !== baseline.slug) {
      throw new Error(
        `Ambiguous baseline score identity: ${identity} -> ${existing.slug}, ${baseline.slug}`,
      );
    }
    this.baselineByIdentity.set(identity, baseline);
  }

  private exactBaselineThreshold(slug: string): ReferenceScoreThreshold | undefined {
    const score = this.baselineBySlug.get(slug);
    if (score === undefined) {
      return undefined;
    }
    return this.baselineThreshold(slug, slug, score);
  }

  private identityBaselineThreshold(slug: string): ReferenceScoreThreshold | undefined {
    const identity = ReferenceScoreSlugIdentity.from(slug);
    const baseline = this.baselineByIdentity.get(identity);
    if (baseline === undefined) {
      return undefined;
    }
    return this.baselineThreshold(slug, baseline.slug, baseline.score);
  }

  private baselineThreshold(
    slug: string,
    sourceSlug: string,
    score: number,
  ): ReferenceScoreThreshold {
    const currentThreshold = this.effectiveVisualThreshold(slug);
    return {
      ...this.localizedThreshold(slug, {
        slug: sourceSlug,
        minScore: Math.min(score, currentThreshold.minScore),
        reason: `EN比較結果を基準に採用 (${sourceSlug})`,
      }),
    };
  }

  private effectiveVisualThreshold(slug: string): ReferenceScoreThreshold {
    return (
      this.exactThreshold(slug) ??
      this.prefixThreshold(slug) ??
      this.defaultThreshold(slug)
    );
  }

  private exactThreshold(slug: string): ReferenceScoreThreshold | undefined {
    return this.thresholds.find((threshold) => threshold.slug === slug);
  }

  private prefixThreshold(slug: string): ReferenceScoreThreshold | undefined {
    const prefix = ReferenceScoreSlugPrefix.from(slug);
    return this.thresholds.find((threshold) =>
      ReferenceScoreSlugPrefix.matches(threshold.slug, prefix),
    );
  }

  private localizeThreshold(
    slug: string,
    threshold: ReferenceScoreThreshold,
  ): ReferenceScoreThreshold {
    return threshold.slug === slug ? threshold : this.localizedThreshold(slug, threshold);
  }

  private localizedThreshold(
    slug: string,
    threshold: ReferenceScoreThreshold,
  ): ReferenceScoreThreshold {
    return {
      ...threshold,
      slug,
    };
  }

  private defaultThreshold(slug: string): ReferenceScoreThreshold {
    return {
      slug,
      minScore: this.globalMinScore,
      reason: "",
    };
  }
}

class ReferenceScoreSlugPrefix {
  static from(slug: string): string {
    return slug.match(/^\d{2}-\d{2}/)?.[0] ?? "";
  }

  static matches(slug: string, prefix: string): boolean {
    return prefix.length > 0 && slug.startsWith(`${prefix}-`);
  }
}

class ReferenceScoreSlugIdentity {
  static from(slug: string): string {
    return slug.match(/^\d{2}-\d{2}/)?.[0] ?? slug.match(/^\d{2}/)?.[0] ?? slug;
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
  visualAccepted("03-01-class-diagram-enumeration", 89.06),
  visualAccepted("03-02-class-diagram-inheritance", 87.89),
  visualAccepted("04-01-sequence-diagram-simple", 91.51),
  visualAccepted("04-02-sequence-diagram-activate-deactivate", 91.01),
  visualAccepted("05-01-er-diagram-simple", 91.59),
  visualAccepted("05-02-er-diagram-multi-entity", 90.38),
  visualAccepted("06-01-state-diagram-v2-failure-path", 93.35),
  visualAccepted("07-01-mindmap-simple", 86.8),
  visualAccepted("07-02-mindmap-icons", 85.22),
  visualAccepted("08-01-c4-context-simple", 97.31),
  visualAccepted("08-02-c4-context-full", 84.64),
  visualAccepted("08-03-c4-container", 94.61),
  visualAccepted("08-04-c4-component", 95.57),
  visualAccepted("08-05-c4-dynamic", 94.24),
  visualAccepted("08-06-c4-deployment", 92.98),
  visualAccepted("09-01-architecture-diagram-simple", 95.44),
  visualAccepted("09-02-architecture-diagram-multi-service", 94.41),
  visualAccepted("10-01-block-diagram-horizontal", 88.85),
  visualAccepted("10-02-block-diagram-vertical", 85.96),
  visualAccepted("12-01-git-graph-simple", 89.84),
  visualAccepted("12-02-git-graph-multi-branch", 98.18),
  visualAccepted("13-01-ishikawa-diagram-3-categories", 93.3),
  visualAccepted("13-02-ishikawa-diagram-4-categories", 97.29),
  visualAccepted("14-01-kanban-simple", 90.54),
  visualAccepted("14-02-kanban-full", 96.45),
  visualAccepted("16-01-pie-chart-rendering-ownership", 98.05),
  visualAccepted("17-01-quadrant-chart-simple", 97.38),
  visualAccepted("17-02-quadrant-chart-campaigns", 97.39),
  visualAccepted("19-01-requirement-diagram-single", 90.37),
  visualAccepted("19-02-requirement-diagram-multi", 88.53),
  visualAccepted("20-02-sankey-large", 90.78),
  visualAccepted("21-01-timeline-phases", 81.8),
  visualAccepted("21-02-timeline-history", 84.42),
  visualAccepted("22-01-tree-view-simple", 89.76),
  visualAccepted("22-02-tree-view-file-system", 91.39),
  visualAccepted("23-01-treemap-flat", 91.8),
  visualAccepted("23-02-treemap-beta-nested", 90.62),
  visualAccepted("24-01-user-journey-diagram-preview", 96.6),
  visualAccepted("24-02-user-journey-working-day", 97.48),
  visualAccepted("25-01-venn-diagram-2-sets", 91.04),
  visualAccepted("25-02-venn-diagram-3-sets-with-styles", 94.17),
  visualAccepted("26-01-wardley-map-simple", 91.15),
  visualAccepted("26-02-wardley-map-full-with-notes", 91.54),
];

function visualAccepted(slug: string, minScore: number): ReferenceScoreThreshold {
  return {
    slug,
    minScore,
    reason:
      "ユーザーが実アプリ画面で目視確認済み。99点化は投資対効果（ROI）が悪く、現状スコアを品質下限として固定。",
  };
}

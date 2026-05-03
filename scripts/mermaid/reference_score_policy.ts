import { MERMAID_VISUAL_ACCEPTED_SCORE_FLOORS } from "./reference_score_floors";

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
    for (const baseline of baselines) {
      this.registerBaseline(baseline);
    }
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
    this.baselineBySlug.set(baseline.slug, this.validateScore(baseline));
    const identity = ReferenceScoreSlugIdentity.from(baseline.slug);
    this.assertUniqueIdentity(identity, baseline);
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
    return this.exactThreshold(slug) ?? this.prefixThreshold(slug) ?? this.defaultThreshold(slug);
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

  private validateScore(baseline: ReferenceScoreBaseline): number {
    if (!Number.isFinite(baseline.score)) {
      throw new Error(`Invalid baseline score for ${baseline.slug}`);
    }
    return baseline.score;
  }

  // biome-ignore lint/complexity/noExcessiveCognitiveComplexity: baseline競合時の判定で2段目分岐が必要。
  private assertUniqueIdentity(identity: string, baseline: ReferenceScoreBaseline): void {
    const existing = this.baselineByIdentity.get(identity);
    const existingSlug = existing?.slug ?? baseline.slug;
    switch (existingSlug) {
      case baseline.slug:
        return;
      default:
        this.throwAmbiguousBaselineIdentity(identity, existingSlug, baseline);
    }
  }

  private throwAmbiguousBaselineIdentity(
    identity: string,
    existingSlug: string,
    baseline: ReferenceScoreBaseline,
  ): never {
    throw new Error(
      `Ambiguous baseline score identity: ${identity} -> ${existingSlug}, ${baseline.slug}`,
    );
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

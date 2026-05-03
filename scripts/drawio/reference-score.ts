import type { MagickOps, NormalizedPair } from "../mermaid/reference_image_ops";

export interface DrawioReferenceScoreRow {
  slug: string;
  score: number;
  canvasRmseScore: number;
  canvasMaeScore: number;
  contentRmseScore: number;
  contentMaeScore: number;
  contentPhashScore: number;
  rawDimensionScore: number;
  minScore: number;
  exceptionReason: string;
  passed: boolean;
}

export class DrawioReferenceScores {
  static minimum(scores: DrawioReferenceScoreRow[]): number {
    return Math.min(...scores.map((it) => it.score));
  }
}

export class DrawioReferenceScorer {
  constructor(
    private magick: MagickOps,
    private minScore: number,
  ) {}

  score(normalized: NormalizedPair[]): DrawioReferenceScoreRow[] {
    return normalized.map((pair) => this.scorePair(pair));
  }

  private scorePair(normalized: NormalizedPair): DrawioReferenceScoreRow {
    const metrics = this.metricScores(normalized);
    const score = this.visualRecognitionScore(metrics);
    return {
      slug: normalized.pair.slug,
      score,
      canvasRmseScore: metrics.canvasRmseScore,
      canvasMaeScore: metrics.canvasMaeScore,
      contentRmseScore: metrics.contentRmseScore,
      contentMaeScore: metrics.contentMaeScore,
      contentPhashScore: metrics.contentPhashScore,
      rawDimensionScore: metrics.rawDimensionScore,
      minScore: this.minScore,
      exceptionReason: "",
      passed: score >= this.minScore,
    };
  }

  private metricScores(normalized: NormalizedPair): DrawioReferenceMetrics {
    return {
      canvasRmseScore: this.scoreMetric(
        "RMSE",
        normalized.officialCanvasPath,
        normalized.katanaCanvasPath,
      ),
      canvasMaeScore: this.scoreMetric(
        "MAE",
        normalized.officialCanvasPath,
        normalized.katanaCanvasPath,
      ),
      contentRmseScore: this.scoreMetric(
        "RMSE",
        normalized.officialContentPath,
        normalized.katanaContentPath,
      ),
      contentMaeScore: this.scoreMetric(
        "MAE",
        normalized.officialContentPath,
        normalized.katanaContentPath,
      ),
      contentPhashScore: this.scoreMetric(
        "PHASH",
        normalized.officialContentPath,
        normalized.katanaContentPath,
      ),
      rawDimensionScore: this.rawDimensionScore(normalized),
    };
  }

  private visualRecognitionScore(metrics: DrawioReferenceMetrics): number {
    return Math.min(
      Math.max(metrics.contentMaeScore, metrics.contentPhashScore),
      this.severeCropScore(metrics),
    );
  }

  private scoreMetric(metric: string, left: string, right: string): number {
    return Math.max(0, 100 * (1 - this.magick.compareNormalizedError(metric, left, right)));
  }

  private rawDimensionScore(normalized: NormalizedPair): number {
    return (
      100 *
      this.magick
        .imageSize(normalized.pair.officialPath)
        .coverageBy(this.magick.imageSize(normalized.pair.katanaPath))
    );
  }

  private severeCropScore(metrics: DrawioReferenceMetrics): number {
    return metrics.rawDimensionScore < 50 ? metrics.rawDimensionScore : 100;
  }
}

interface DrawioReferenceMetrics {
  readonly canvasRmseScore: number;
  readonly canvasMaeScore: number;
  readonly contentRmseScore: number;
  readonly contentMaeScore: number;
  readonly contentPhashScore: number;
  readonly rawDimensionScore: number;
}

import { expect, test } from "bun:test";
import { ReferenceScorePolicy } from "./reference_score_policy";

test("言語別slugでも同じ図形番号の目視確認済み下限を使う", () => {
  const policy = new ReferenceScorePolicy(99);

  const threshold = policy.thresholdFor("20-02-sankey-beta");

  expect(threshold.slug).toBe("20-02-sankey-beta");
  expect(threshold.minScore).toBe(90.78);
  expect(threshold.reason).toContain("目視確認済み");
});

test("同じ図形番号の下限がない場合は全体下限を使う", () => {
  const policy = new ReferenceScorePolicy(99);

  const threshold = policy.thresholdFor("99-99-no-pattern");

  expect(threshold.slug).toBe("99-99-no-pattern");
  expect(threshold.minScore).toBe(99);
  expect(threshold.reason).toBe("");
});

test("ベースラインの同一slugを優先して閾値を採用する", () => {
  const policy = new ReferenceScorePolicy(99, [], [{ slug: "20-02-sankey-beta", score: 91.23 }]);

  const threshold = policy.thresholdFor("20-02-sankey-beta");

  expect(threshold.slug).toBe("20-02-sankey-beta");
  expect(threshold.minScore).toBe(91.23);
  expect(threshold.reason).toBe("EN比較結果を基準に採用 (20-02-sankey-beta)");
});

test("ベースラインslugが一致しない場合は先頭番号で対応する", () => {
  const policy = new ReferenceScorePolicy(
    99,
    [],
    [
      { slug: "20-02-sankey-large", score: 91.23 },
      { slug: "27-02-xy-chart-bar-line", score: 98.7 },
    ],
  );

  const thresholdA = policy.thresholdFor("20-02-sankey-beta");
  const thresholdB = policy.thresholdFor("27-02-xy");

  expect(thresholdA.slug).toBe("20-02-sankey-beta");
  expect(thresholdA.minScore).toBe(91.23);
  expect(thresholdB.slug).toBe("27-02-xy");
  expect(thresholdB.minScore).toBe(98.7);
});

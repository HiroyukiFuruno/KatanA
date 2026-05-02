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

  const threshold = policy.thresholdFor("14-02-kanban");

  expect(threshold.slug).toBe("14-02-kanban");
  expect(threshold.minScore).toBe(99);
  expect(threshold.reason).toBe("");
});

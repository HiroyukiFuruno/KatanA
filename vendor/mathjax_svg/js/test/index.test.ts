import { expect, test } from "bun:test";
import "../src/index.ts";

declare global {
  var __katana_mathjax_render: ((latex: string, options: undefined) => string) | undefined;
}

const sourceRender = globalThis.__katana_mathjax_render;

if (!sourceRender) {
  throw new Error("MathJax renderer was not initialized");
}

const assertPhysicsExpression = (
  render: (latex: string, options: undefined) => string,
  expression: string,
  width: string,
) => {
  const svg = render(expression, undefined);

  expect(svg).toStartWith("<svg ");
  expect(svg).toContain(`data-latex="${expression}"`);
  expect(svg).toContain(`width="${width}`);
  expect(svg).not.toContain("data-mjx-error");
  expect(svg).not.toContain("merror");
};

const physicsExpressions = [
  ["\\qty{1}{m}", "5.757ex"],
  ["\\dv{f}{x}", "3.548ex"],
];

test.each(physicsExpressions)("renders Physics expression %s from source", (expression, width) => {
  assertPhysicsExpression(sourceRender, expression, width);
});

test.each(physicsExpressions)(
  "renders Physics expression %s from bundle",
  async (expression, width) => {
    await import("../out/index.mjs");
    const bundleRender = globalThis.__katana_mathjax_render;
    if (!bundleRender) {
      throw new Error("Generated MathJax bundle did not initialize the renderer");
    }
    expect(bundleRender).not.toBe(sourceRender);
    assertPhysicsExpression(bundleRender, expression, width);
  },
);

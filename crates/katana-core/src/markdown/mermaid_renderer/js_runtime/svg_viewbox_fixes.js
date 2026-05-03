function katanaNormalizeFlowchartViewBoxSvg(svg) {
  if (!svg.includes('aria-roledescription="flowchart-v2"')) {
    return svg;
  }
  return katanaNormalizeFlowchartViewBox(svg, katanaReadViewBox(svg), katanaContentBox(svg));
}

function katanaNormalizeFlowchartViewBox(svg, viewBox, contentBox) {
  if (!katanaShouldTrimFlowchartViewBox(viewBox, contentBox)) {
    return svg;
  }
  return katanaSetNormalizedSvgSize(svg, [
    0,
    0,
    katanaFormatSvgNumber(katanaFlowchartViewBoxWidth(contentBox)),
    katanaFormatSvgNumber(viewBox[3] + viewBox[1]),
  ]);
}

function katanaShouldTrimFlowchartViewBox(viewBox, contentBox) {
  return [
    viewBox,
    contentBox,
    Math.abs(viewBox?.[0] + 8) < 0.01,
    Math.abs(viewBox?.[1] + 8) < 0.01,
    contentBox?.[2] > 40,
  ].every(Boolean);
}

function katanaFlowchartViewBoxWidth(contentBox) {
  const trim = contentBox[2] > 450 ? 8.25 : 8;
  return contentBox[2] - trim;
}

function katanaNormalizeGitGraphSvg(svg) {
  if (!svg.includes('aria-roledescription="gitGraph"')) {
    return svg;
  }
  return katanaNormalizeGitGraphViewBox(svg, katanaReadViewBox(svg), katanaContentBox(svg));
}

function katanaNormalizeGitGraphViewBox(svg, viewBox, contentBox) {
  if (!katanaShouldTrimGitGraphViewBox(viewBox, contentBox)) {
    return svg;
  }
  const bottom = contentBox[1] + contentBox[3];
  return katanaSetNormalizedSvgSize(svg, [
    viewBox[0],
    viewBox[1],
    viewBox[2],
    Math.ceil(bottom - viewBox[1] - 11),
  ]);
}

function katanaShouldTrimGitGraphViewBox(viewBox, contentBox) {
  return [viewBox, contentBox, contentBox?.[3] < viewBox?.[3] - 8].every(Boolean);
}

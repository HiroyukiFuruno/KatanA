function katanaPrepareMermaidSource(source) {
  if (!katanaShouldAppendTodayMarkerOff(source)) {
    return source;
  }
  return katanaSourceWithTodayMarker(source);
}

function katanaShouldAppendTodayMarkerOff(source) {
  if (!source.trimStart().startsWith("gantt")) {
    return false;
  }
  return !/\btodayMarker\b/.test(source);
}

function katanaSourceWithTodayMarker(source) {
  return source.trim().endsWith("\n") ? `${source}todayMarker off` : `${source}\ntodayMarker off`;
}

function katanaNormalizeMermaidSvg(svg, request) {
  return katanaFinishNormalizedSvg(katanaNormalizeColoredMermaidSvg(svg, request), request);
}

function katanaNormalizeColoredMermaidSvg(svg, request) {
  return katanaNormalizeDiagramSpecificSvg(
    katanaNormalizeNativeSvgFallbacks(
      katanaNormalizeNegativeRectWidths(katanaNormalizeC4RelationColors(svg, request)),
    ),
    request,
  );
}

function katanaFinishNormalizedSvg(svg, request) {
  return katanaNormalizeFinalSvg(svg, request);
}

function katanaSetNormalizedSvgSize(svg, dimensions) {
  const width = katanaNormalizedSvgWidth(dimensions);
  const height = katanaNormalizedSvgHeight(dimensions);
  return katanaSetSvgMaxWidth(
    katanaSetSvgViewBox(
      katanaSetSvgDimension(katanaSetSvgDimension(svg, "width", width), "height", height),
      dimensions.join(" "),
    ),
    width,
  );
}

function katanaNormalizedSvgWidth(dimensions) {
  return String(Math.max(1, Math.ceil(dimensions[2] * katanaSvgDimensionScale())));
}

function katanaNormalizedSvgHeight(dimensions) {
  return String(Math.max(1, Math.ceil(dimensions[3] * katanaSvgDimensionScale())));
}

function katanaSvgDimensionScale() {
  return 1;
}

function katanaNormalizeFinalSvg(svg, request) {
  return katanaNormalizeIshikawaSvg(
    katanaEscapeTextMarkers(
      katanaNormalizeEmptyTextFill(katanaNormalizeFontFamilyAttributes(svg), request),
    ),
  );
}

function katanaNormalizeDiagramSpecificSvg(svg, request) {
  let normalized = svg;
  normalized = katanaNormalizeTreeViewSvg(normalized, request);
  normalized = katanaNormalizeTreemapSvg(normalized, request);
  normalized = katanaNormalizeClassSvg(normalized);
  normalized = katanaNormalizeErSvg(normalized);
  normalized = katanaNormalizeFlowchartEdgeLabelSvg(normalized, request);
  normalized = katanaNormalizeFlowchartViewBoxSvg(normalized);
  normalized = katanaNormalizeStateEdgeLabelSvg(normalized, request);
  normalized = katanaNormalizeRequirementEdgeLabelSvg(normalized);
  normalized = katanaNormalizeJourneySvg(normalized, request);
  normalized = katanaNormalizePieSvg(normalized);
  normalized = katanaNormalizeVennSvg(normalized, request);
  normalized = katanaNormalizeArchitectureSvg(normalized, request);
  normalized = katanaNormalizeRadarSvg(normalized);
  normalized = katanaNormalizeGitGraphSvg(normalized);
  normalized = katanaNormalizeKanbanSvg(normalized, request);
  normalized = katanaNormalizeSankeySvg(normalized);
  normalized = katanaNormalizeMindmapSvg(normalized);
  normalized = katanaNormalizeBlockSvg(normalized);
  normalized = katanaNormalizeWardleySvg(normalized, request);
  normalized = katanaNormalizeIshikawaSvg(normalized);
  normalized = katanaNormalizeReviewFeedbackSvg(normalized, request);
  return normalized;
}

function katanaNormalizeFontFamilyAttributes(svg) {
  return svg.replace(
    /font-family=""([^"]+)",\s*([^"]*)"/g,
    (_match, family, fallback) => `font-family="${family}, ${fallback}"`,
  );
}

function katanaEscapeTextMarkers(svg) {
  return svg.replace(/<<([^<>]+)>>/g, "&lt;&lt;$1&gt;&gt;");
}

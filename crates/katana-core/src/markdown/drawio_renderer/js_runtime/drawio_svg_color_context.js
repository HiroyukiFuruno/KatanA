const KATANA_DRAWIO_LARGE_WHITE_PATH_AREA = 10000;
const KATANA_DRAWIO_AWS_SHAPE_PREFIXES = [
  "mxgraph.aws.",
  "mxgraph.aws2.",
  "mxgraph.aws3.",
  "mxgraph.aws4.",
  "mxgraph.aws3d.",
];
const KATANA_DRAWIO_AZURE_SHAPE_PREFIXES = ["mxgraph.azure.", "mxgraph.mscae.", "mxgraph.office."];
const KATANA_DRAWIO_VISUAL_SHAPE_TAGS = new Set([
  "circle",
  "ellipse",
  "line",
  "path",
  "polygon",
  "polyline",
  "rect",
]);
const KATANA_DRAWIO_PAINT_ATTRIBUTES = new Set(["fill", "stroke", "stop-color"]);
const KATANA_DRAWIO_AZURE_DARK_ATTRIBUTES = new Set(["color", "fill", "stroke"]);
const KATANA_DRAWIO_AWS_DOCUMENT_ORIGINAL_COLORS = new Set(["#007cbd", "#545b64"]);
const KATANA_DRAWIO_AWS_ORIGINAL_COLORS = new Set([
  "#000000",
  "#007cbd",
  "#277116",
  "#292929",
  "#545b64",
  "#5e5e5e",
  "#60a337",
  "#5a30b5",
  "#945df2",
  "#bc1356",
  "#f34482",
  "#c7131f",
  "#f54749",
  "#d05c17",
  "#f78e04",
  "#ececec",
  "#f4b934",
  "#3334b9",
  "#4d72f3",
  "#4ab29a",
  "rgb(0, 0, 0)",
  "rgb(236, 236, 236)",
]);

function katanaDrawioContextOriginalColor(element, name, value) {
  return (
    katanaDrawioAzureDocumentDarkColor(element, name, value) ||
    katanaDrawioAzureDarkColor(element, name, value) ||
    katanaDrawioLegacyAwsDarkColor(element, name, value) ||
    katanaDrawioOriginalAwsGradientStopColor(element, name, value) ||
    katanaDrawioSimpleDiagramDarkColor(element, name, value) ||
    katanaDrawioLargeWhitePathColor(element, name, value) ||
    katanaDrawioOriginalAwsShapeColor(element, name, value)
  );
}

function katanaDrawioAzureDarkColor(element, name, value) {
  return (
    KATANA_DRAWIO_AZURE_DARK_COLOR.get(katanaDrawioAzureDarkColorKey(element, name, value)) ?? ""
  );
}

function katanaDrawioAzureDarkColorKey(element, name, value) {
  return [
    katanaDrawioIsDarkMode(),
    KATANA_DRAWIO_AZURE_DARK_ATTRIBUTES.has(name),
    katanaDrawioIsAzureShapeElement(element),
  ].every(Boolean)
    ? `${name}|${value}`
    : "";
}

function katanaDrawioLegacyAwsDarkColor(element, name, value) {
  return (
    KATANA_DRAWIO_AWS_LEGACY_DARK_COLOR.get(
      katanaDrawioLegacyAwsDarkColorKey(element, name, value),
    ) ?? ""
  );
}

function katanaDrawioLegacyAwsDarkColorKey(element, name, value) {
  return katanaDrawioShouldMapLegacyAwsDarkColor(element, name) ? `${name}|${value}` : "";
}

function katanaDrawioShouldMapLegacyAwsDarkColor(element, name) {
  return [
    katanaDrawioIsDarkMode(),
    katanaDrawioIsPaintAttribute(name),
    katanaDrawioIsVisualShapeTag(element),
    katanaDrawioIsLegacyAwsShapeElement(element),
  ].every(Boolean);
}

function katanaDrawioLargeWhitePathColor(element, name, value) {
  if (!katanaDrawioIsLargeWhitePath(element, name, value)) {
    return "";
  }
  return "#121212";
}

function katanaDrawioIsLargeWhitePath(element, name, value) {
  return [
    katanaDrawioIsDarkMode(),
    element.localName === "path",
    name === "fill",
    value === "#ffffff",
    katanaDrawioElementArea(element) > KATANA_DRAWIO_LARGE_WHITE_PATH_AREA,
  ].every(Boolean);
}

function katanaDrawioElementArea(element) {
  const box = element.getBBox();
  return box.width * box.height;
}

function katanaDrawioOriginalAwsShapeColor(element, name, value) {
  return katanaDrawioShouldKeepOriginalAwsShapeColor(element, name, value) ? value : "";
}

function katanaDrawioOriginalAwsGradientStopColor(element, name, value) {
  return katanaDrawioShouldKeepOriginalAwsGradientStopColor(element, name, value) ? value : "";
}

function katanaDrawioShouldKeepOriginalAwsShapeColor(element, name, value) {
  return [
    katanaDrawioIsDarkMode(),
    katanaDrawioIsPaintAttribute(name),
    katanaDrawioIsVisualShapeTag(element),
    KATANA_DRAWIO_AWS_ORIGINAL_COLORS.has(value),
    katanaDrawioIsAwsColorContext(element),
  ].every(Boolean);
}

function katanaDrawioIsPaintAttribute(name) {
  return KATANA_DRAWIO_PAINT_ATTRIBUTES.has(name);
}

function katanaDrawioIsVisualShapeTag(element) {
  return KATANA_DRAWIO_VISUAL_SHAPE_TAGS.has(element.localName);
}

function katanaDrawioIsAwsShapeElement(element) {
  return KATANA_DRAWIO_AWS_SHAPE_PREFIXES.some((it) =>
    katanaDrawioElementCellShape(element).startsWith(it),
  );
}

function katanaDrawioIsAzureShapeElement(element) {
  return KATANA_DRAWIO_AZURE_SHAPE_PREFIXES.some((it) =>
    katanaDrawioElementCellShape(element).startsWith(it),
  );
}

function katanaDrawioIsAwsColorContext(element) {
  return (
    katanaDrawioIsAwsShapeElement(element) ||
    KATANA_DRAWIO_AWS_DOCUMENT_ORIGINAL_COLORS.has(element.getAttribute("fill")) ||
    KATANA_DRAWIO_AWS_DOCUMENT_ORIGINAL_COLORS.has(element.getAttribute("stroke"))
  );
}

function katanaDrawioIsLegacyAwsShapeElement(element) {
  return ["mxgraph.aws.", "mxgraph.aws3.", "mxgraph.aws4."].some((it) =>
    katanaDrawioElementCellShape(element).startsWith(it),
  );
}

function katanaDrawioShouldKeepOriginalAwsGradientStopColor(element, name, value) {
  return [
    katanaDrawioIsDarkMode(),
    name === "stop-color",
    element.localName === "stop",
    KATANA_DRAWIO_AWS_ORIGINAL_COLORS.has(value),
  ].every(Boolean);
}

const KATANA_DRAWIO_AWS_LEGACY_DARK_COLOR = new Map([
  ["fill|#000000", "#ededed"],
  ["fill|#2e73b8", "#5f9bd6"],
  ["fill|#7d7c7c", "#848383"],
  ["fill|#d2d3d3", "#373838"],
  ["fill|#d9a741", "#835801"],
  ["fill|#e05243", "#ef7568"],
  ["fill|#f58534", "#bd5c17"],
  ["fill|#f58536", "#bd5c18"],
  ["fill|#ffffff", "#232f3e"],
  ["stroke|#000000", "#ffffff"],
  ["stroke|#e6e6e6", "#272727"],
]);

const KATANA_DRAWIO_AZURE_DARK_COLOR = new Map([
  ["color|#ffffff", "#121212"],
  ["color|rgb(255, 255, 255)", "rgb(18, 18, 18)"],
  ["fill|#ffffff", "#121212"],
  ["fill|rgb(255, 255, 255)", "rgb(18, 18, 18)"],
  ["stroke|#ffffff", "#121212"],
  ["stroke|rgb(255, 255, 255)", "rgb(18, 18, 18)"],
]);

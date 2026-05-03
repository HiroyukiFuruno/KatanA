function katanaDrawioAzureDocumentDarkColor(element, name, value) {
  return (
    KATANA_DRAWIO_AZURE_DOCUMENT_DARK_COLOR.get(
      katanaDrawioAzureDocumentDarkColorKey(element, name, value),
    ) ?? ""
  );
}

function katanaDrawioAzureDocumentDarkColorKey(element, name, value) {
  return [
    katanaDrawioIsDarkMode(),
    katanaDrawioIsAzureDiagramSource(),
    KATANA_DRAWIO_AZURE_DARK_ATTRIBUTES.has(name),
    katanaDrawioAzureDocumentAllowsAttribute(element, name),
  ].every(Boolean)
    ? `${name}|${value}`
    : "";
}

function katanaDrawioAzureDocumentAllowsAttribute(element, name) {
  return name === "color" || katanaDrawioIsVisualShapeTag(element);
}

function katanaDrawioIsAzureDiagramSource() {
  return KATANA_DRAWIO_AZURE_SHAPE_PREFIXES.some(katanaDrawioRequestSourceIncludes);
}

function katanaDrawioRequestSourceIncludes(value) {
  return katanaDrawioRequestSource().includes(value);
}

const KATANA_DRAWIO_AZURE_DOCUMENT_DARK_COLOR = new Map([
  ["color|#ffffff", "#121212"],
  ["color|rgb(255, 255, 255)", "rgb(18, 18, 18)"],
  ["fill|#ffffff", "#121212"],
  ["fill|rgb(255, 255, 255)", "rgb(18, 18, 18)"],
  ["stroke|#ffffff", "#121212"],
  ["stroke|rgb(255, 255, 255)", "rgb(18, 18, 18)"],
]);

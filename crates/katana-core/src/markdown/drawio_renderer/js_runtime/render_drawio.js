function katanaRenderDrawioSvg() {
  const request = katanaDrawioRenderRequest();
  const container = katanaCreateDrawioViewerContainer(request.source);
  const viewer = katanaCreateDrawioViewer(container);
  return katanaViewerSvgMarkup(viewer);
}

function katanaDrawioRenderRequest() {
  const request = globalThis.__katanaDrawioRequest;
  katanaAssertDrawioRequest(request);
  katanaAssertDrawioViewerRuntime();
  return request;
}

function katanaAssertDrawioRequest(request) {
  if (!request?.source) {
    throw new Error("Draw.io render request is missing");
  }
}

function katanaAssertDrawioViewerRuntime() {
  if (typeof GraphViewer !== "function") {
    throw new Error("Draw.io viewer runtime is not initialized");
  }
}

function katanaCreateDrawioViewerContainer(source) {
  const container = document.createElement("div");
  container.setAttribute("id", "graph-container");
  container.setAttribute("class", "mxgraph");
  container.setAttribute("data-mxgraph", JSON.stringify(katanaDrawioViewerConfig(source)));
  document.body.appendChild(container);
  return container;
}

function katanaDrawioViewerConfig(source) {
  return {
    xml: katanaDrawioSourceForViewer(source),
    toolbar: "",
    "auto-fit": false,
    "auto-crop": false,
    "auto-origin": true,
    "check-visible-state": false,
  };
}

function katanaDrawioSourceForViewer(source) {
  const xml = String(source)
    .trim()
    .replace(/^<\?xml[^>]*>\s*/i, "");
  return katanaDrawioUncompressedModel(xml) ?? xml;
}

function katanaDrawioUncompressedModel(xml) {
  const start = xml.indexOf("<mxGraphModel");
  const end = xml.indexOf("</mxGraphModel>");
  if (katanaHasDrawioModelRange(start, end)) {
    return xml.slice(start, end + "</mxGraphModel>".length);
  }
  return null;
}

function katanaHasDrawioModelRange(start, end) {
  return [start >= 0, end >= start].every(Boolean);
}

function katanaCreateDrawioViewer(container) {
  const state = { viewer: null };
  GraphViewer.createViewerForElement(container, (viewer) => {
    state.viewer = viewer;
  });
  return katanaRequiredDrawioViewer(state, container);
}

function katanaRequiredDrawioViewer(state, container) {
  if (state.viewer?.graph) {
    return state.viewer;
  }
  throw new Error(katanaDrawioViewerError(container));
}

function katanaDrawioViewerError(container) {
  const message = container.innerText || "Draw.io viewer did not produce SVG";
  return `Draw.io viewer failed: ${message}`;
}

function katanaViewerSvgMarkup(viewer) {
  const svg = viewer.graph.getSvg();
  katanaConfigureDrawioSvg(svg);
  katanaPostprocessDrawioSvg(svg);
  return new XMLSerializer().serializeToString(svg);
}

function katanaConfigureDrawioSvg(svg) {
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-roledescription", "drawio");
}

katanaRenderDrawioSvg();

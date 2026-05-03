function katanaInstallDrawioMxUtils(context) {
  if (typeof mxUtils !== "object") {
    return;
  }
  mxUtils.createXmlDocument = katanaDrawioCreateXmlDocument;
  mxUtils.load = (url) => katanaDrawioResourceResponse(context.resources, url);
  mxUtils.get = (url, onload, onerror) =>
    katanaDrawioGetResource(context.resources, url, onload, onerror);
}

function katanaDrawioGetResource(resources, url, onload, onerror) {
  const response = katanaDrawioResourceResponse(resources, url);
  katanaDispatchDrawioResourceResponse(response, onload, onerror);
  return response;
}

function katanaDispatchDrawioResourceResponse(response, onload, onerror) {
  if (katanaDrawioIsOkResponse(response)) {
    onload?.(response);
    return;
  }
  onerror?.(response);
}

function katanaDrawioIsOkResponse(response) {
  return [response.getStatus() >= 200, response.getStatus() < 300].every(Boolean);
}

function katanaInstallDrawioImageBundles(context) {
  if (typeof mxGraph !== "function") {
    return;
  }
  const originalGetImageFromBundles = mxGraph.prototype.getImageFromBundles;
  mxGraph.prototype.getImageFromBundles = function getImageFromBundles(key, ...args) {
    return katanaDrawioImageFromBundles(
      context.resources,
      originalGetImageFromBundles,
      this,
      key,
      args,
    );
  };
}

function katanaDrawioImageFromBundles(resources, originalGetImageFromBundles, graph, key, args) {
  const resource = resources.get(katanaDrawioResourcePath(key));
  if (katanaIsDrawioSvgResource(resource)) {
    return katanaDrawioDataUri(resource);
  }
  return katanaDrawioBundleFallback(originalGetImageFromBundles, graph, key, args);
}

function katanaIsDrawioSvgResource(resource) {
  return resource?.mime_type === "image/svg+xml";
}

function katanaDrawioBundleFallback(originalGetImageFromBundles, graph, key, args) {
  if (typeof originalGetImageFromBundles === "function") {
    return originalGetImageFromBundles.apply(graph, [key, ...args]);
  }
  return key;
}

function katanaInstallDrawioSvgImages(context) {
  if (typeof mxSvgCanvas2D !== "function") {
    return;
  }
  const originalImage = mxSvgCanvas2D.prototype.image;
  mxSvgCanvas2D.prototype.image = function image(x, y, width, height, src, aspect, flipH, flipV) {
    originalImage.call(
      this,
      x,
      y,
      width,
      height,
      katanaDrawioResolvedImageSrc(context.resources, src),
      aspect,
      flipH,
      flipV,
    );
  };
}

function katanaDrawioResolvedImageSrc(resources, src) {
  const resource = resources.get(katanaDrawioResourcePath(src));
  return [katanaDrawioImageResourceSrc(resource), katanaDrawioExternalImageSrc(src), src].filter(
    Boolean,
  )[0];
}

function katanaDrawioImageResourceSrc(resource) {
  return [resource].filter(Boolean).map(katanaDrawioDataUri).concat([""])[0];
}

function katanaDrawioExternalImageSrc(src) {
  return [src]
    .filter(katanaIsExternalDrawioImageSrc)
    .map(() => KATANA_DRAWIO_TRANSPARENT_IMAGE_SRC)
    .concat([""])[0];
}

function katanaIsExternalDrawioImageSrc(src) {
  return ["http://", "https://"].some((it) => String(src).startsWith(it));
}

function katanaDrawioDataUri(resource) {
  return `data:${resource.mime_type};base64,${katanaDrawioResourceBase64(resource)}`;
}

const KATANA_DRAWIO_TRANSPARENT_IMAGE_SRC =
  "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";

function katanaInstallDrawioOwnerSvgElement() {
  if (!katanaNeedsDrawioOwnerSvgElement()) {
    return;
  }
  Object.defineProperty(KatanaNode.prototype, "ownerSVGElement", {
    get() {
      return katanaOwnerSvgElement(this);
    },
  });
}

function katanaNeedsDrawioOwnerSvgElement() {
  return [
    typeof KatanaNode === "function",
    !Object.getOwnPropertyDescriptor(KatanaNode.prototype, "ownerSVGElement"),
  ].every(Boolean);
}

function katanaOwnerSvgElement(node) {
  const state = { node, result: null };
  while (katanaShouldContinueOwnerSvgSearch(state)) {
    katanaSearchOwnerSvgStep(state);
  }
  return state.result;
}

function katanaShouldContinueOwnerSvgSearch(state) {
  return [state.node !== null, state.result === null].every(Boolean);
}

function katanaSearchOwnerSvgStep(state) {
  if (String(state.node.nodeName).toLowerCase() === "svg") {
    state.result = state.node;
    return;
  }
  state.node = state.node.parentNode;
}

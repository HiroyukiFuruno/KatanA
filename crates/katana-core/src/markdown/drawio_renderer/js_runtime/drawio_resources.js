function katanaDrawioRuntimeContext() {
  return {
    request: globalThis.__katanaDrawioRequest ?? {},
    resources: new Map(),
    stencilXmlFiles: new Map(),
  };
}

function katanaRegisterDrawioResources(context) {
  (context.request.resources ?? []).forEach((resource) => {
    katanaRegisterDrawioResource(context, resource);
  });
}

function katanaRegisterDrawioResource(context, resource) {
  const path = katanaDrawioResourcePath(resource.path);
  context.resources.set(path, resource);
  katanaRunDrawioShapeResource(resource);
  katanaStoreDrawioStencilResource(context, path, resource);
}

function katanaRunDrawioShapeResource(resource) {
  if (resource.mime_type !== "application/javascript") {
    return;
  }
  katanaExecuteDrawioShapeResource(resource.path, katanaDrawioResourceContent(resource));
}

function katanaExecuteDrawioShapeResource(path, content) {
  try {
    Function(content).call(globalThis);
  } catch (error) {
    katanaTrackDrawioResourceError(path, error);
  }
}

function katanaTrackDrawioResourceError(path, error) {
  globalThis.__katanaDrawioResourceErrors.push(`${path}: ${String(error)}`);
}

function katanaStoreDrawioStencilResource(context, path, resource) {
  if (!katanaIsDrawioStencilResource(path, resource)) {
    return;
  }
  context.stencilXmlFiles.set(path, katanaDrawioResourceContent(resource));
}

function katanaIsDrawioStencilResource(path, resource) {
  return [
    resource.mime_type === "text/xml",
    ["stencils/", "stencils\\"].some((prefix) => path.indexOf(prefix) === 0),
  ].every(Boolean);
}

function katanaDrawioResourceResponse(resources, url) {
  const path = katanaDrawioResourcePath(url);
  const resource = resources.get(path);
  katanaTrackDrawioMissingResource(resource, path);
  return katanaDrawioResponseForResource(resource);
}

function katanaTrackDrawioMissingResource(resource, path) {
  if (!resource) {
    globalThis.__katanaDrawioMissingResources.push(path);
  }
}

function katanaDrawioResponseForResource(resource) {
  return {
    getStatus() {
      return katanaDrawioResourceStatus(resource);
    },
    getText() {
      return katanaDrawioResourceContent(resource);
    },
    getXml() {
      return new DOMParser().parseFromString(katanaDrawioResourceContent(resource), "text/xml");
    },
    getDocumentElement() {
      return this.getXml().documentElement;
    },
    isReady() {
      return true;
    },
  };
}

function katanaDrawioResourceStatus(resource) {
  if (resource) {
    return 200;
  }
  return 404;
}

function katanaDrawioResourceContent(resource) {
  if (resource) {
    return katanaDecodeDrawioResourceContent(resource);
  }
  return "";
}

function katanaDecodeDrawioResourceContent(resource) {
  if (resource.encoding === "base64") {
    return atob(resource.content);
  }
  return resource.content;
}

function katanaDrawioResourceBase64(resource) {
  if (resource.encoding === "base64") {
    return resource.content;
  }
  return btoa(resource.content);
}

function katanaDrawioResourcePath(url) {
  let path = String(url ?? "");
  path = path.replace(/^https?:\/\/[^/]+\//, "");
  path = path.replace(/^katana:\/\/drawio\//, "");
  path = path.replace(/[?#].*$/, "");
  path = path.replace(/^\/+/, "");
  return path;
}

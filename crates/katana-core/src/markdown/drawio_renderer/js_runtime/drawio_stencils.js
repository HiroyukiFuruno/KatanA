function katanaInstallDrawioStencilLoader(context) {
  if (typeof mxStencilRegistry !== "object") {
    return;
  }
  const originalGetStencil = mxStencilRegistry.getStencil;
  mxStencilRegistry.getStencil = function getStencil(name) {
    return katanaGetDrawioStencil(this, originalGetStencil, context.stencilXmlFiles, name);
  };
}

function katanaGetDrawioStencil(registry, originalGetStencil, stencilXmlFiles, name) {
  const stencil = originalGetStencil.call(registry, name);
  if (stencil) {
    return stencil;
  }
  return katanaFindDrawioStencil(stencilXmlFiles, name);
}

function katanaFindDrawioStencil(stencilXmlFiles, name) {
  const stencil = Array.from(stencilXmlFiles.values())
    .map((content) => katanaStencilFromXmlContent(content, name))
    .find(Boolean);
  return katanaNullableDrawioStencil(stencil);
}

function katanaNullableDrawioStencil(stencil) {
  if (stencil) {
    return stencil;
  }
  return null;
}

function katanaStencilFromXmlContent(content, name) {
  const stencilName = katanaDrawioStencilName(content, name);
  const match = content.match(katanaDrawioStencilPattern(stencilName));
  if (!match) {
    return null;
  }
  return katanaCreateDrawioStencil(name, match[0]);
}

function katanaDrawioStencilName(content, name) {
  const exactName = String(name);
  const groupedName = katanaDrawioGroupedStencilName(content, exactName);
  if (groupedName) {
    return groupedName;
  }
  return exactName;
}

function katanaDrawioGroupedStencilName(content, exactName) {
  const group = katanaDrawioStencilGroup(content);
  if (!katanaDrawioNameBelongsToGroup(exactName, group)) {
    return "";
  }
  return katanaFindDrawioGroupedStencilName(content, exactName.slice(group.length + 1));
}

function katanaDrawioStencilGroup(content) {
  const match = content.match(/<shapes[^>]+name="([^"]+)"/);
  return match?.[1] ?? "";
}

function katanaDrawioNameBelongsToGroup(name, group) {
  return Boolean(group) && name.toLowerCase().startsWith(`${group.toLowerCase()}.`);
}

function katanaFindDrawioGroupedStencilName(content, normalizedName) {
  return katanaDrawioShapeNames(content).find(
    (shapeName) => katanaNormalizeDrawioStencilName(shapeName) === normalizedName,
  );
}

function katanaDrawioShapeNames(content) {
  return Array.from(content.matchAll(/<(shape|stencil)[^>]+name="([^"]+)"/gm)).map(
    (match) => match[2],
  );
}

function katanaNormalizeDrawioStencilName(name) {
  return String(name)
    .trim()
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .replace(/[\s-]+/g, "_")
    .toLowerCase();
}

function katanaDrawioStencilPattern(name) {
  const escapedName = String(name).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp(`<(shape|stencil)[^>]+name="${escapedName}"[^>]*>([\\s\\S]*?)<\\/\\1>`, "m");
}

function katanaCreateDrawioStencil(name, xml) {
  try {
    const xmlDoc = new DOMParser().parseFromString(xml, "text/xml");
    const stencil = new mxStencil(xmlDoc.documentElement);
    mxStencilRegistry.addStencil(name, stencil);
    return stencil;
  } catch {
    return null;
  }
}

function katanaNormalizeDrawioGeometry(svg) {
  if (!katanaDrawioRequestSource().includes("mxgraph.aws")) {
    return;
  }
  Array.from(svg.querySelectorAll("g"))
    .filter(katanaDrawioIsCellContentGroup)
    .filter(katanaDrawioNeedsHalfPixelTranslate)
    .forEach(katanaApplyDrawioHalfPixelTranslate);
}

function katanaDrawioIsCellContentGroup(group) {
  return group.parentNode?.getAttribute?.("data-cell-id");
}

function katanaDrawioNeedsHalfPixelTranslate(group) {
  return [!group.getAttribute("transform"), katanaDrawioGroupHasDirectShape(group)].every(Boolean);
}

function katanaDrawioGroupHasDirectShape(group) {
  return Array.from(group.children).some((child) =>
    KATANA_DRAWIO_HALF_PIXEL_SHAPE_TAGS.has(child.localName),
  );
}

function katanaApplyDrawioHalfPixelTranslate(group) {
  group.setAttribute("transform", "translate(0.5,0.5)");
}

const KATANA_DRAWIO_HALF_PIXEL_SHAPE_TAGS = new Set([
  "circle",
  "ellipse",
  "line",
  "path",
  "polygon",
  "polyline",
  "rect",
]);

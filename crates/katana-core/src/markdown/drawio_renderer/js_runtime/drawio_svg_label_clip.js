let KATANA_DRAWIO_LABEL_CLIP_INDEX = 0;

function katanaClipWrappedDrawioTextLabel(text, box) {
  [katanaDrawioSvgAncestor(text)].filter(Boolean).forEach((svg) => {
    const id = katanaDrawioNextLabelClipId();
    katanaDrawioSvgDefs(svg).appendChild(katanaCreateDrawioLabelClipPath(id, box));
    text.setAttribute("clip-path", `url(#${id})`);
  });
}

function katanaDrawioNextLabelClipId() {
  KATANA_DRAWIO_LABEL_CLIP_INDEX += 1;
  return `katana-drawio-label-clip-${KATANA_DRAWIO_LABEL_CLIP_INDEX}`;
}

function katanaDrawioSvgAncestor(element) {
  return katanaDrawioAncestorElements(element)
    .filter((it) => it.localName === "svg")
    .concat([null])[0];
}

function katanaDrawioAncestorElements(element) {
  const parent1 = katanaDrawioParentNode(element);
  const parent2 = katanaDrawioParentNode(parent1);
  const parent3 = katanaDrawioParentNode(parent2);
  const parent4 = katanaDrawioParentNode(parent3);
  const parent5 = katanaDrawioParentNode(parent4);
  const parent6 = katanaDrawioParentNode(parent5);
  return [element, parent1, parent2, parent3, parent4, parent5, parent6].filter(Boolean);
}

function katanaDrawioSvgDefs(svg) {
  return [svg.querySelector("defs")].filter(Boolean).concat([katanaCreateDrawioSvgDefs(svg)])[0];
}

function katanaCreateDrawioSvgDefs(svg) {
  const defs = document.createElementNS("http://www.w3.org/2000/svg", "defs");
  svg.insertBefore(defs, svg.firstChild);
  return defs;
}

function katanaCreateDrawioLabelClipPath(id, box) {
  const clipPath = document.createElementNS("http://www.w3.org/2000/svg", "clipPath");
  clipPath.setAttribute("id", id);
  clipPath.appendChild(katanaCreateDrawioLabelClipRect(box));
  return clipPath;
}

function katanaCreateDrawioLabelClipRect(box) {
  const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
  rect.setAttribute("x", box.x);
  rect.setAttribute("y", box.y);
  rect.setAttribute("width", box.width);
  rect.setAttribute("height", box.height);
  return rect;
}

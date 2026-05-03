function katanaDrawioElementCellShape(element) {
  return katanaDrawioSourceCellShape(katanaDrawioElementCellId(element));
}

function katanaDrawioElementCellId(element) {
  return katanaDrawioElementAncestors(element)
    .map(katanaDrawioDirectElementCellId)
    .filter(Boolean)
    .concat([""])[0];
}

function katanaDrawioElementAncestors(element) {
  const parent1 = katanaDrawioParentNode(element);
  const parent2 = katanaDrawioParentNode(parent1);
  const parent3 = katanaDrawioParentNode(parent2);
  const parent4 = katanaDrawioParentNode(parent3);
  const parent5 = katanaDrawioParentNode(parent4);
  const parent6 = katanaDrawioParentNode(parent5);
  const parent7 = katanaDrawioParentNode(parent6);
  const parent8 = katanaDrawioParentNode(parent7);
  return [element, parent1, parent2, parent3, parent4, parent5, parent6, parent7, parent8].filter(
    Boolean,
  );
}

function katanaDrawioParentNode(node) {
  return [node]
    .filter(Boolean)
    .map((it) => it.parentNode)
    .filter(Boolean)
    .concat([null])[0];
}

function katanaDrawioDirectElementCellId(element) {
  return [element.getAttribute?.("data-cell-id")].filter(Boolean).concat([""])[0];
}

function katanaDrawioSourceCellShape(id) {
  return [KATANA_DRAWIO_SOURCE_CELL_STYLE_CACHE.get(id)]
    .filter(Boolean)
    .map((style) => style.get("shape"))
    .filter(Boolean)
    .concat([""])[0];
}

function katanaDrawioSourceCellStyleMap() {
  return new Map(
    Array.from(katanaDrawioSourceStyleRequestSource().matchAll(/<mxCell\b([^>]*)>/g))
      .map(katanaDrawioSourceCellStyleEntry)
      .filter((entry) => entry[0]),
  );
}

function katanaDrawioSourceCellStyleEntry(match) {
  const attributes = katanaDrawioSourceStyleXmlAttributes(match[1]);
  return [katanaDrawioSourceCellId(attributes), katanaDrawioSourceCellStyle(attributes)];
}

function katanaDrawioSourceCellId(attributes) {
  return [attributes.get("id")].filter(Boolean).concat([""])[0];
}

function katanaDrawioSourceCellStyle(attributes) {
  return katanaDrawioSourceStyleMap([attributes.get("style")].filter(Boolean).concat([""])[0]);
}

function katanaDrawioSourceStyleRequestSource() {
  return String(globalThis.__katanaDrawioRequest?.source ?? "");
}

function katanaDrawioSourceStyleXmlAttributes(source) {
  return new Map(
    Array.from(String(source).matchAll(/([a-zA-Z0-9:_-]+)="([^"]*)"/g)).map((match) => [
      match[1],
      decodeHtmlEntities(match[2]),
    ]),
  );
}

function katanaDrawioSourceStyleMap(style) {
  return new Map(
    String(style)
      .split(";")
      .map(katanaDrawioSourceStyleEntry)
      .filter((entry) => entry[0]),
  );
}

function katanaDrawioSourceStyleEntry(value) {
  return String(value).split("=").concat(["", ""]).slice(0, 2);
}

const KATANA_DRAWIO_SOURCE_CELL_STYLE_CACHE = katanaDrawioSourceCellStyleMap();

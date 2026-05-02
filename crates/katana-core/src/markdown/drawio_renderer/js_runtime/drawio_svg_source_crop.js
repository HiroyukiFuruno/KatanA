function katanaDrawioSourceContentBox(svg) {
  return katanaDrawioSourceCropBox(svg, katanaDrawioSourceGeometryEntries());
}

function katanaDrawioSourceGeometryEntries() {
  return [
    ...katanaDrawioSourceCellGeometryEntries(),
    ...katanaDrawioSourceUserObjectGeometryEntries(),
  ]
    .filter(katanaHasDrawioSourceGeometryEntry)
    .filter(katanaIsTopLevelDrawioSourceGeometryEntry);
}

function katanaDrawioSourceCellGeometryEntries() {
  return Array.from(
    katanaDrawioRequestSource().matchAll(/<mxCell\b([^>]*)>\s*<mxGeometry\b([^>]*)/g),
  ).map(katanaDrawioSourceGeometryEntry);
}

function katanaDrawioSourceUserObjectGeometryEntries() {
  return Array.from(
    katanaDrawioRequestSource().matchAll(
      /<UserObject\b([^>]*)>\s*<mxCell\b([^>]*)>\s*<mxGeometry\b([^>]*)/g,
    ),
  ).map(katanaDrawioSourceUserObjectGeometryEntry);
}

function katanaDrawioSourceGeometryEntry(match) {
  const cellAttributes = katanaDrawioXmlAttributes(match[1]);
  const geometryAttributes = katanaDrawioXmlAttributes(match[2]);
  return katanaDrawioSourceGeometry(cellAttributes, geometryAttributes);
}

function katanaDrawioSourceUserObjectGeometryEntry(match) {
  const userObjectAttributes = katanaDrawioXmlAttributes(match[1]);
  const cellAttributes = katanaDrawioXmlAttributes(match[2]);
  const geometryAttributes = katanaDrawioXmlAttributes(match[3]);
  return katanaDrawioSourceGeometry(userObjectAttributes, geometryAttributes, cellAttributes);
}

function katanaDrawioSourceGeometry(cellAttributes, geometryAttributes, fallbackAttributes) {
  return {
    id: katanaDrawioCellAttribute(cellAttributes, "id"),
    parent: katanaDrawioSourceParentAttribute(cellAttributes, fallbackAttributes),
    x: katanaDrawioNumberAttribute(geometryAttributes, "x"),
    y: katanaDrawioNumberAttribute(geometryAttributes, "y"),
    width: katanaDrawioNumberAttribute(geometryAttributes, "width"),
    height: katanaDrawioNumberAttribute(geometryAttributes, "height"),
  };
}

function katanaDrawioSourceParentAttribute(cellAttributes, fallbackAttributes) {
  return [
    katanaDrawioOptionalCellAttribute(cellAttributes, "parent"),
    katanaDrawioOptionalCellAttribute(fallbackAttributes, "parent"),
  ]
    .filter(Boolean)
    .concat([""])[0];
}

function katanaDrawioOptionalCellAttribute(attributes, name) {
  return attributes ? katanaDrawioCellAttribute(attributes, name) : "";
}

function katanaHasDrawioSourceGeometryEntry(entry) {
  return [
    entry.id,
    Number.isFinite(entry.x),
    Number.isFinite(entry.y),
    entry.width > 0,
    entry.height > 0,
  ].every(Boolean);
}

function katanaIsTopLevelDrawioSourceGeometryEntry(entry) {
  return ["", "1"].includes(entry.parent);
}

function katanaDrawioNumberAttribute(attributes, name) {
  const value = katanaDrawioCellAttribute(attributes, name);
  return value === "" ? Number.NaN : Number(value);
}

function katanaDrawioSourceCropBox(svg, entries) {
  const sourceBox = katanaDrawioUnionBox(entries);
  const offset = katanaDrawioSourceCropOffset(svg, entries);
  return [sourceBox, offset].every(Boolean)
    ? katanaDrawioShiftedSourceCropBox(sourceBox, offset)
    : null;
}

function katanaDrawioSourceCropOffset(svg, entries) {
  return katanaDrawioMedianOffset(katanaDrawioSourceCropOffsets(svg, entries));
}

function katanaDrawioSourceCropOffsets(svg, entries) {
  return entries.map((entry) => katanaDrawioSourceCellOffset(svg, entry)).filter(Boolean);
}

function katanaDrawioSourceCellOffset(svg, entry) {
  const box = katanaDrawioMeasuredSourceCellBox(svg, entry);
  return box ? { x: box.x - entry.x, y: box.y - entry.y } : null;
}

function katanaDrawioMeasuredSourceCellBox(svg, entry) {
  return [katanaDrawioCellGroup(svg, entry.id)]
    .filter(Boolean)
    .map(katanaDrawioCellShapeBox)
    .filter(Boolean)
    .filter((box) => katanaDrawioSimilarSourceBox(box, entry))
    .concat([null])[0];
}

function katanaDrawioSimilarSourceBox(box, entry) {
  return [
    katanaDrawioSimilarSourceSize(box.width, entry.width),
    katanaDrawioSimilarSourceSize(box.height, entry.height),
  ].every(Boolean);
}

function katanaDrawioSimilarSourceSize(actual, expected) {
  return Math.abs(actual - expected) <= Math.max(4, expected * 0.02);
}

function katanaDrawioMedianOffset(offsets) {
  return offsets.length === 0
    ? null
    : {
        x: katanaDrawioMedianValue(offsets.map((offset) => offset.x)),
        y: katanaDrawioMedianValue(offsets.map((offset) => offset.y)),
      };
}

function katanaDrawioMedianValue(values) {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.floor(sorted.length / 2)];
}

function katanaDrawioShiftedSourceCropBox(box, offset) {
  return {
    x: Math.floor(box.x + offset.x),
    y: Math.floor(box.y + offset.y),
    width: Math.ceil(box.width),
    height: Math.ceil(box.height),
  };
}

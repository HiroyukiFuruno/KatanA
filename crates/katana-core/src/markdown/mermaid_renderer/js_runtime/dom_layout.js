const KATANA_HIDDEN_LAYOUT_TAGS = new Set([
  "style",
  "script",
  "defs",
  "marker",
  "lineargradient",
  "radialgradient",
  "stop",
  "filter",
  "fedropshadow",
  "clippath",
  "mask",
  "pattern",
]);

function katanaOwnText(node) {
  return String(node?._textContent ?? "");
}

function katanaTextContent(node) {
  return node ? `${katanaOwnText(node)}${katanaTextContentChildren(node)}` : "";
}

function katanaTextContentChildren(node) {
  return (node.children ?? []).map((child) => katanaTextContent(child)).join("");
}

function katanaLayoutTextContent(node) {
  if (katanaIsHiddenLayoutNode(node)) {
    return "";
  }
  return `${katanaOwnText(node)}${katanaLayoutTextChildren(node)}`;
}

function katanaLayoutTextChildren(node) {
  return (node.children ?? []).map((child) => katanaLayoutTextContent(child)).join("");
}

function katanaIsHiddenLayoutNode(node) {
  return [!node, KATANA_HIDDEN_LAYOUT_TAGS.has(node?.localName)].includes(true);
}

function katanaNodeValue(value) {
  if (value instanceof KatanaNode) {
    return value;
  }
  return document.createTextNode(katanaStringValue(value));
}

function katanaStringValue(value) {
  return String(value ?? "");
}

function katanaNumberAttr(node, name) {
  return katanaNullableNumber(node?.getAttribute?.(name));
}

function katanaNullableNumber(rawValue) {
  return katanaHasNumberValue(rawValue) ? katanaFiniteNumber(Number(rawValue)) : null;
}

function katanaHasNumberValue(rawValue) {
  return ![null, undefined, ""].includes(rawValue);
}

function katanaFiniteNumber(value) {
  return Number.isFinite(value) ? value : null;
}

function katanaBox(x, y, width, height) {
  return { x, y, width, height, w: width, h: height };
}

function katanaUnionBox(boxes) {
  if (boxes.length === 0) {
    return katanaBox(0, 0, 0, 0);
  }
  const minX = Math.min(...boxes.map((box) => box.x));
  const minY = Math.min(...boxes.map((box) => box.y));
  const maxX = Math.max(...boxes.map((box) => box.x + box.width));
  const maxY = Math.max(...boxes.map((box) => box.y + box.height));
  return katanaBox(minX, minY, maxX - minX, maxY - minY);
}

function katanaVisibleChildBoxes(node) {
  return (node.children ?? [])
    .filter((child) => !KATANA_HIDDEN_LAYOUT_TAGS.has(child.localName))
    .map((child) => {
      const box = child.getBBox();
      const offset = katanaNodeTranslate(child);
      return katanaBox(box.x + offset[0], box.y + offset[1], box.width, box.height);
    });
}

function katanaMeasuredBox(node) {
  if (KATANA_HIDDEN_LAYOUT_TAGS.has(node.localName)) {
    return katanaBox(0, 0, 0, 0);
  }
  return katanaVisibleMeasuredBox(node);
}

function katanaVisibleMeasuredBox(node) {
  return (
    katanaExplicitMeasuredBox(node) ??
    katanaDirectShapeBox(node) ??
    katanaChildTextMeasuredBox(node)
  );
}

function katanaExplicitMeasuredBox(node) {
  const attrs = katanaBoxAttributes(node);
  if ([attrs.width > 0, attrs.height > 0].includes(true)) {
    return katanaBox(attrs.x, attrs.y, attrs.width, attrs.height);
  }
  return null;
}

function katanaBoxAttributes(node) {
  return {
    x: katanaNumberAttrOrDefault(node, "x", 0),
    y: katanaNumberAttrOrDefault(node, "y", 0),
    width: katanaNumberAttrOrDefault(node, "width", 0),
    height: katanaNumberAttrOrDefault(node, "height", 0),
  };
}

function katanaNumberAttrOrDefault(node, name, fallback) {
  const value = katanaNumberAttr(node, name);
  if (value === null) {
    return fallback;
  }
  return value;
}

function katanaDirectShapeBox(node) {
  return katanaLineBox(node) ?? katanaCircleBox(node);
}

function katanaChildTextMeasuredBox(node) {
  const childBox = katanaUnionBox(katanaVisibleChildBoxes(node));
  const text = katanaMeasuredNodeText(node, childBox);
  return katanaBox(
    0,
    0,
    katanaMeasuredTextWidth(text, childBox),
    katanaMeasuredTextHeight(text, childBox),
  );
}

function katanaMeasuredNodeText(node, childBox) {
  return katanaOwnText(node) || katanaFallbackLayoutText(node, childBox);
}

function katanaFallbackLayoutText(node, childBox) {
  return childBox.width === 0 ? katanaLayoutTextContent(node) : "";
}

function katanaMeasuredTextWidth(text, childBox) {
  return Math.max(16, katanaTextWidthFallback(text), childBox.width);
}

function katanaMeasuredTextHeight(text, childBox) {
  return Math.max(24, katanaTextHeightFallback(text), childBox.height);
}

function katanaTextWidthFallback(text) {
  return text.length === 0 ? 0 : Math.max(16, katanaTextWidth(text));
}

function katanaTextHeightFallback(text) {
  return text.length === 0 ? 0 : 24;
}

function katanaLineBox(node) {
  if (node.localName !== "line") {
    return null;
  }
  return katanaLineShapeBox(node);
}

function katanaLineShapeBox(node) {
  const x1 = katanaNumberAttrOrDefault(node, "x1", 0);
  const y1 = katanaNumberAttrOrDefault(node, "y1", 0);
  const x2 = katanaNumberAttrOrDefault(node, "x2", 0);
  const y2 = katanaNumberAttrOrDefault(node, "y2", 0);
  return katanaBox(Math.min(x1, x2), Math.min(y1, y2), Math.abs(x2 - x1), Math.abs(y2 - y1));
}

function katanaCircleBox(node) {
  if (!katanaIsRoundSvgShape(node)) {
    return null;
  }
  return katanaRoundShapeBox(node);
}

function katanaRoundShapeBox(node) {
  const cx = katanaNumberAttrOrDefault(node, "cx", 0);
  const cy = katanaNumberAttrOrDefault(node, "cy", 0);
  const rx = katanaRoundRadius(node, "rx");
  const ry = katanaRoundRadius(node, "ry");
  return katanaBox(cx - rx, cy - ry, rx * 2, ry * 2);
}

function katanaRoundRadius(node, name) {
  return katanaNumberAttr(node, name) ?? katanaNumberAttrOrDefault(node, "r", 0);
}

function katanaIsRoundSvgShape(node) {
  return ["circle", "ellipse"].includes(node.localName);
}

function katanaNodeTranslate(node) {
  const match = katanaTransformValue(node).match(/translate\(([-\d.]+)[,\s]+([-\d.]+)\)/);
  if (match) {
    return [Number(match[1]), Number(match[2])];
  }
  return [0, 0];
}

function katanaTransformValue(node) {
  return String(node?.getAttribute?.("transform") ?? "");
}

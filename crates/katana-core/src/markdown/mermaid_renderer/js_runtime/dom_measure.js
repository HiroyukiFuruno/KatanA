function katanaMeasuredBoxAccurate(node) {
  if (KATANA_HIDDEN_LAYOUT_TAGS.has(node.localName)) {
    return katanaBox(0, 0, 0, 0);
  }
  return katanaVisibleMeasuredBoxAccurate(node);
}

function katanaVisibleMeasuredBoxAccurate(node) {
  return (
    katanaTextElementMeasuredBox(node) ??
    katanaTextFragmentMeasuredBox(node) ??
    katanaExplicitMeasuredBox(node) ??
    katanaDirectMeasuredBox(node) ??
    katanaChildTextBox(node)
  );
}

function katanaTextElementMeasuredBox(node) {
  if (node.localName === "text") {
    return katanaTextElementBox(node);
  }
  return null;
}

function katanaTextFragmentMeasuredBox(node) {
  if (["tspan", "#text"].includes(node.localName)) {
    return katanaTextFragmentBox(node);
  }
  return null;
}

function katanaDirectMeasuredBox(node) {
  return katanaLineBox(node) ?? katanaCircleBox(node) ?? katanaPathBox(node);
}

function katanaChildTextBox(node) {
  const childBox = katanaUnionBox(katanaVisibleChildBoxes(node));
  const text = katanaMeasuredNodeText(node, childBox);
  return katanaUnionBox([childBox, katanaOptionalTextBox(node, text)].filter(katanaHasArea));
}

function katanaOptionalTextBox(node, text) {
  return text ? katanaAnchoredTextNodeBox(node, text, 0, 0) : katanaBox(0, 0, 0, 0);
}

function katanaHasArea(box) {
  return [box.width > 0, box.height > 0].includes(true);
}

function katanaPathBox(node) {
  return katanaSvgPathBox(node) ?? katanaSvgPointsBox(node);
}

function katanaSvgPathBox(node) {
  if (node.localName === "path") {
    return katanaOptionalNumberListBox(node.getAttribute("d"));
  }
  return null;
}

function katanaSvgPointsBox(node) {
  if (["polygon", "polyline"].includes(node.localName)) {
    return katanaOptionalNumberListBox(node.getAttribute("points"));
  }
  return null;
}

function katanaOptionalNumberListBox(value) {
  return value ? katanaNumberListBox(value) : null;
}

function katanaNumberListBox(value) {
  const numbers = Array.from(String(value).matchAll(/-?\d+(?:\.\d+)?(?:e-?\d+)?/gi)).map((match) =>
    Number(match[0]),
  );
  if (numbers.length < 2) {
    return null;
  }
  const xs = numbers.filter((_value, index) => index % 2 === 0);
  const ys = numbers.filter((_value, index) => index % 2 === 1);
  return katanaBox(
    Math.min(...xs),
    Math.min(...ys),
    Math.max(...xs) - Math.min(...xs),
    Math.max(...ys) - Math.min(...ys),
  );
}

function katanaTextElementBox(node) {
  const lines = katanaTextLines(node);
  const text = katanaTextElementText(node, lines);
  if (!text) {
    return katanaBox(0, 0, 0, 0);
  }
  return katanaUnionBox(
    katanaTextLineValues(node, text, lines).map((line, index) =>
      katanaTextLineBox(node, line, index),
    ),
  );
}

function katanaTextElementText(node, lines) {
  return lines.length === 0 ? katanaTextContent(node) : lines.map((line) => line.text).join("");
}

function katanaTextLineValues(node, text, lines) {
  if (lines.length === 0) {
    return [{ text, x: katanaNumberAttr(node, "x"), y: katanaNumberAttr(node, "y") }];
  }
  return lines;
}

function katanaTextLineBox(node, line, index) {
  const x = katanaLineValueOrDefault(line.x, katanaNumberAttrOrDefault(node, "x", 0));
  const y = katanaLineValueOrDefault(line.y, katanaTextLineFallbackY(node, index));
  return katanaAnchoredTextNodeBox(node, line.text, x, y);
}

function katanaLineValueOrDefault(value, fallback) {
  return value ?? fallback;
}

function katanaTextLineFallbackY(node, index) {
  return (katanaNumberAttr(node, "y") ?? 0) + index * katanaLineHeight(node);
}

function katanaTextFragmentBox(node) {
  const text = katanaFragmentText(node);
  if (text) {
    return katanaAnchoredTextNodeBox(
      katanaTextFragmentParent(node),
      text,
      katanaNumberAttrOrDefault(node, "x", 0),
      katanaNumberAttrOrDefault(node, "y", 0),
    );
  }
  return katanaBox(0, 0, 0, 0);
}

function katanaTextFragmentParent(node) {
  return node.parentNode ?? node;
}

function katanaFragmentText(node) {
  return [katanaOwnText(node), katanaTextContent(node)].find(Boolean) ?? "";
}

KatanaNode.prototype.getBBox = function getBBox() {
  return katanaMeasuredBoxAccurate(this);
};

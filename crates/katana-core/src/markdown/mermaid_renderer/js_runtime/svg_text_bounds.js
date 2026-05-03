function katanaSvgTextBox(svg, startIndex, tag, offset) {
  const endIndex = svg.indexOf("</text>", startIndex);
  if (endIndex < 0) {
    return null;
  }
  return katanaSvgTextBoxFromRange(svg, startIndex, endIndex, tag, offset);
}

function katanaSvgTextBoxFromRange(svg, startIndex, endIndex, tag, offset) {
  const lines = katanaSvgTextLines(svg, startIndex, endIndex);
  if (lines.length === 0) {
    return null;
  }
  return katanaSvgTextBoxFromLines(lines, tag, offset);
}

function katanaSvgTextBoxFromLines(lines, tag, offset) {
  const x = katanaAttrNumberOrDefault(tag, "x", 0) + offset[0];
  const y = katanaAttrNumberOrDefault(tag, "y", 0) + offset[1];
  const width = Math.max(16, ...lines.map((line) => line.length * 8));
  return katanaAnchoredTextBox(tag, x, y, width);
}

function katanaSvgTextLines(svg, startIndex, endIndex) {
  const body = svg.slice(startIndex, endIndex);
  const lines = Array.from(body.matchAll(/<tspan\b[^>]*>([^<]*)<\/tspan>/g))
    .map((match) => katanaNormalizedSvgText(match[1]))
    .filter(Boolean);
  return lines.length > 0 ? lines : [katanaNormalizedSvgText(body)].filter(Boolean);
}

function katanaNormalizedSvgText(value) {
  return value
    .replace(/<[^>]+>/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function katanaAnchoredTextBox(tag, x, y, width) {
  const anchor = katanaTextAnchorAttr(tag);
  return (KATANA_ANCHORED_TEXT_BOXES[anchor] ?? KATANA_ANCHORED_TEXT_BOXES.start)(x, y, width);
}

function katanaTextAnchorAttr(tag) {
  return String(katanaAttrText(tag, "text-anchor") ?? "start");
}

const KATANA_ANCHORED_TEXT_BOXES = {
  end: (x, y, width) => [x - width, y - 18, x, y + 12],
  middle: (x, y, width) => [x - width / 2, y - 18, x + width / 2, y + 12],
  start: (x, y, width) => [x, y - 18, x + width, y + 12],
};

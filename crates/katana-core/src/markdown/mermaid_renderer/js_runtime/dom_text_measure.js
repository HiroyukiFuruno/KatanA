function katanaAnchoredTextNodeBox(node, text, x, y) {
  const width = Math.max(1, katanaTextNodeWidth(node, text));
  const height = katanaLineHeight(node);
  return katanaBox(
    katanaAnchoredTextLeft(katanaTextAnchor(node), x, width),
    y - height * 0.8,
    width,
    height,
  );
}

function katanaAnchoredTextLeft(anchor, x, width) {
  return (KATANA_TEXT_ANCHORS[anchor] ?? KATANA_TEXT_ANCHORS.start)(x, width);
}

const KATANA_TEXT_ANCHORS = {
  end: (x, width) => x - width,
  middle: (x, width) => x - width / 2,
  start: (x) => x,
};

function katanaTextAnchor(node) {
  return String(katanaTextAnchorValues(node).find(Boolean) ?? "start");
}

function katanaTextAnchorValues(node) {
  return [
    node.getAttribute?.("text-anchor"),
    node.style?.getPropertyValue?.("text-anchor"),
    node.parentNode?.getAttribute?.("text-anchor"),
  ];
}

function katanaLineHeight(node) {
  const fontSize = Number(String(katanaLineHeightFontSize(node)).replace("px", ""));
  return katanaBrowserTextBoxHeight(katanaFiniteFontSize(fontSize));
}

function katanaBrowserTextBoxHeight(fontSize) {
  const measured = KATANA_TEXT_BOX_HEIGHTS[String(fontSize)];
  if (measured) {
    return measured;
  }
  return Math.max(12, Math.ceil(fontSize * 1.15));
}

const KATANA_TEXT_BOX_HEIGHTS = {
  10: 11,
  11: 12.25,
  12: 14,
  14: 16,
  16: 19,
  18: 21,
  20: 23,
};

function katanaLineHeightFontSize(node) {
  return katanaLineHeightFontSizeValues(node).find(Boolean) ?? "16";
}

function katanaLineHeightFontSizeValues(node) {
  return [
    node.style?.getPropertyValue?.("font-size"),
    katanaCssComputedStyleValue(node, "font-size"),
    node.getAttribute?.("font-size"),
    node.parentNode?.style?.getPropertyValue?.("font-size"),
    katanaCssComputedStyleValue(node.parentNode, "font-size"),
    node.parentNode?.getAttribute?.("font-size"),
  ];
}

function katanaFiniteFontSize(fontSize) {
  if (Number.isFinite(fontSize)) {
    return fontSize;
  }
  return 16;
}

function katanaTextNodeWidth(node, text) {
  return katanaTextWidth(text) * katanaTextWidthScale(node);
}

function katanaTextWidthScale(node) {
  return (
    katanaFiniteFontSize(Number(String(katanaLineHeightFontSize(node)).replace("px", ""))) / 16
  );
}

function katanaTextWidth(text) {
  const chars = Array.from(katanaMeasuredTextValue(text));
  const characterWidth = chars
    .map((char) => katanaCharacterWidth(char))
    .reduce((width, charWidth) => width + charWidth, 0);
  return characterWidth + katanaTextKerningWidth(chars);
}

function katanaMeasuredTextValue(text) {
  return String(text).replace(/&(?:amp;)?nbsp;|&#160;/g, "\u00A0");
}

function katanaTextKerningWidth(chars) {
  return chars
    .slice(0, -1)
    .map((char, index) => katanaKerningPairWidth(char, chars[index + 1]))
    .reduce((width, pairWidth) => width + pairWidth, 0);
}

function katanaKerningPairWidth(left, right) {
  return KATANA_KERNING_PAIR_WIDTH_READERS[Number(katanaHasWideKerningChar(left, right))](
    left,
    right,
  );
}

function katanaHasWideKerningChar(left, right) {
  return [left, right].some(katanaIsWideTextChar);
}

function katanaIsWideTextChar(char) {
  return char.charCodeAt(0) > 255;
}

function katanaCharacterWidth(char) {
  return KATANA_CHARACTER_WIDTHS[Number(char.charCodeAt(0) > 255)](char);
}

function katanaAsciiCharacterWidth(char) {
  if (char.charCodeAt(0) === 160) {
    return KATANA_ASCII_TEXT_WIDTHS[" "];
  }
  return [KATANA_ASCII_TEXT_WIDTHS[char]].filter(Boolean).concat([8])[0];
}

function katanaWideCharacterWidth(char) {
  return KATANA_WIDE_CHARACTER_WIDTHS[Number(globalThis.__katanaMermaidDiagramType === "kanban")](
    char,
  );
}

const KATANA_WIDE_CHARACTER_WIDTHS = [
  katanaDefaultWideCharacterWidth,
  // WHY: Official Kanban measures wide labels slightly narrower; matching total width wraps one character too early inside cards.
  () => 12.5,
];

function katanaDefaultWideCharacterWidth(char) {
  if (katanaIsWidePunctuation(char)) {
    return 20;
  }
  return KATANA_DEFAULT_WIDE_CHARACTER_WIDTHS[Number(katanaIsCjkIdeograph(char))]();
}

function katanaIsCjkIdeograph(char) {
  const codePoint = katanaCharacterCodePoint(char);
  return KATANA_CJK_IDEOGRAPH_RULES.every((rule) => rule(codePoint));
}

function katanaCharacterCodePoint(char) {
  return char.codePointAt(0) ?? 0;
}

function katanaIsCjkIdeographStart(codePoint) {
  return codePoint >= 0x4e00;
}

function katanaIsCjkIdeographEnd(codePoint) {
  return codePoint <= 0x9fff;
}

function katanaIsWidePunctuation(char) {
  return ["。", "、"].includes(char);
}

const KATANA_DEFAULT_WIDE_CHARACTER_WIDTHS = [() => 15.8, () => 16.3];
const KATANA_CJK_IDEOGRAPH_RULES = [katanaIsCjkIdeographStart, katanaIsCjkIdeographEnd];

const KATANA_CHARACTER_WIDTHS = [katanaAsciiCharacterWidth, katanaWideCharacterWidth];

const KATANA_KERNING_PAIR_WIDTH_READERS = [
  (left, right) => KATANA_ASCII_KERNING_PAIRS[`${left}${right}`] ?? 0,
  () => 0,
];

KatanaNode.prototype.getComputedTextLength = function getComputedTextLength() {
  const lines = katanaComputedTextLines(this);
  return Math.max(1, ...lines.map((line) => katanaTextNodeWidth(this, line)));
};

function katanaComputedTextLines(node) {
  if (node.localName === "text") {
    return katanaTextLines(node).map((line) => line.text);
  }
  return [katanaTextContent(node)];
}

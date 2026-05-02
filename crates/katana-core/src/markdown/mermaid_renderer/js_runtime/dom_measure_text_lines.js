function katanaTextLines(node) {
  return katanaTextTspans(node).reduce(katanaAppendTextLine, { lines: [], current: null }).lines;
}

function katanaTextTspans(node) {
  return katanaDescendantElements(node, "tspan")
    .filter((child) => katanaTextContent(child).length > 0)
    .filter((child) => !katanaHasTextTspanAncestor(child, node));
}

function katanaHasTextTspanAncestor(child, root) {
  return katanaTextTspanAncestors(child, root).some(katanaIsFilledTextTspan);
}

function katanaTextTspanAncestors(child, root) {
  return katanaTextTspanAncestorList(child.parentNode, root);
}

function katanaTextTspanAncestorList(node, root) {
  return KATANA_TEXT_TSPAN_ANCESTOR_READERS[Number(katanaHasTextAncestorNode(node, root))](
    node,
    root,
  );
}

function katanaHasTextAncestorNode(node, root) {
  return Boolean(node) && node !== root;
}

function katanaIsFilledTextTspan(node) {
  return [node.localName === "tspan", katanaTextContent(node).length > 0].every(Boolean);
}

function katanaAppendTextLine(state, tspan, index) {
  const next = katanaTextLineState(state, tspan, index);
  next.current.text += katanaTextContent(tspan);
  return next;
}

function katanaTextLineState(state, tspan, index) {
  return KATANA_TEXT_LINE_STATE_READERS[Number(katanaStartsTextLine(tspan, index))](state, tspan);
}

function katanaStartsTextLine(tspan, index) {
  return [
    index === 0,
    tspan.hasAttribute("x"),
    Math.abs(katanaSvgTextLengthAttr(tspan, "dy") ?? 0) >= 1,
  ].includes(true);
}

function katanaStartTextLine(state, tspan) {
  const line = { text: "", x: katanaNumberAttr(tspan, "x"), y: katanaTspanLineY(tspan) };
  return { lines: [...state.lines, line], current: line };
}

function katanaTspanLineY(tspan) {
  const values = [katanaSvgTextLengthAttr(tspan, "y"), katanaSvgTextLengthAttr(tspan, "dy")];
  return KATANA_TSPAN_LINE_Y_READERS[Number(values.some(katanaIsSvgTextLengthValue))](values);
}

function katanaIsSvgTextLengthValue(value) {
  return value !== null;
}

function katanaSvgTextLengthAttr(node, name) {
  return [String(node.getAttribute?.(name) ?? "")]
    .filter(Boolean)
    .map((raw) => katanaSvgTextLengthValue(node, raw))
    .concat([null])[0];
}

function katanaSvgTextLengthValue(node, raw) {
  return [raw.match(/^(-?\d+(?:\.\d+)?)([a-z%]*)$/i)]
    .filter(Boolean)
    .map((match) => katanaSvgTextLengthMatchValue(node, match))
    .concat([null])[0];
}

function katanaSvgTextLengthMatchValue(node, match) {
  return [Number(match[1])]
    .filter(Number.isFinite)
    .map((value) => katanaSvgTextLengthPixels(node, value, match[2]))
    .concat([null])[0];
}

function katanaSvgTextLengthPixels(node, value, unit) {
  return (KATANA_SVG_TEXT_LENGTH_UNITS[unit] ?? KATANA_SVG_TEXT_LENGTH_UNITS.px)(node, value);
}

function katanaDescendantElements(node, tagName) {
  return Array.from(node.children).flatMap((child) => katanaDescendantElement(child, tagName));
}

function katanaDescendantElement(child, tagName) {
  return [katanaMatchingDescendantElement(child, tagName)]
    .filter(Boolean)
    .concat(Array.from(child.children).flatMap((node) => katanaDescendantElement(node, tagName)));
}

function katanaMatchingDescendantElement(child, tagName) {
  return [child].filter((node) => node.localName === tagName)[0] ?? null;
}

const KATANA_TEXT_TSPAN_ANCESTOR_READERS = [
  () => [],
  (node, root) => [node].concat(katanaTextTspanAncestorList(node.parentNode, root)),
];

const KATANA_TEXT_LINE_STATE_READERS = [
  (state) => state,
  (state, tspan) => katanaStartTextLine(state, tspan),
];

function katanaSumTspanLineY(values) {
  return katanaOptionalSvgTextLength(values[0]) + katanaOptionalSvgTextLength(values[1]);
}

function katanaOptionalSvgTextLength(value) {
  return [value].filter(katanaIsSvgTextLengthValue).concat([0])[0];
}

const KATANA_TSPAN_LINE_Y_READERS = [() => null, (values) => katanaSumTspanLineY(values)];

const KATANA_SVG_TEXT_LENGTH_UNITS = {
  em: (node, value) =>
    value * katanaFiniteFontSize(Number(String(katanaLineHeightFontSize(node)).replace("px", ""))),
  px: (_node, value) => value,
};

function katanaNormalizeRichDrawioTextLabels(svg) {
  katanaDrawioRichTextLabelEntries().forEach((entry) => {
    katanaNormalizeRichDrawioTextLabel(svg, entry);
  });
}

function katanaDrawioRichTextLabelEntries() {
  return Array.from(katanaDrawioRequestSource().matchAll(/<mxCell\b([^>]*)>/g))
    .map(katanaDrawioRichTextLabelEntry)
    .filter(katanaHasRichDrawioTextLabel);
}

function katanaDrawioRichTextLabelEntry(match) {
  const attributes = katanaDrawioXmlAttributes(match[1]);
  const label = katanaDrawioCellAttribute(attributes, "value");
  return {
    id: katanaDrawioCellAttribute(attributes, "id"),
    lines: katanaDrawioRichTextLines(label),
  };
}

function katanaHasRichDrawioTextLabel(entry) {
  return [
    entry.id,
    entry.lines.length > 0,
    entry.lines.some((line) => line.bold || line.italic),
  ].every(Boolean);
}

function katanaDrawioRichTextLines(label) {
  return String(label)
    .split(/<\s*br\s*\/?>/i)
    .map(katanaDrawioRichTextLine)
    .filter((line) => line.text);
}

function katanaDrawioRichTextLine(source) {
  return {
    text: katanaDrawioHtmlLabelText(source),
    bold: /<\s*b\b/i.test(source),
    italic: /<\s*i\b/i.test(source),
  };
}

function katanaNormalizeRichDrawioTextLabel(svg, entry) {
  [katanaDrawioCellGroup(svg, entry.id)]
    .filter(Boolean)
    .filter((group) => !katanaDrawioGroupHasForeignObject(group))
    .map(katanaDrawioRichTextElement)
    .filter(Boolean)
    .forEach((text) => {
      katanaApplyRichDrawioTextLines(text, entry.lines);
    });
}

function katanaDrawioRichTextElement(group) {
  return Array.from(group.querySelectorAll("text"))[0] ?? null;
}

function katanaApplyRichDrawioTextLines(text, lines) {
  const tspans = Array.from(text.querySelectorAll("tspan"));
  if (tspans.length === 0) {
    katanaApplyRichDrawioTextStyle(text, lines[0]);
    return;
  }
  tspans.forEach((tspan, index) => {
    katanaApplyRichDrawioTextStyle(tspan, lines[index]);
  });
}

function katanaApplyRichDrawioTextStyle(node, line) {
  katanaRichDrawioTextAttributes(line).forEach((attribute) => {
    node.setAttribute(attribute.name, attribute.value);
  });
}

function katanaRichDrawioTextAttributes(line) {
  return [line]
    .filter(Boolean)
    .flatMap((it) => [
      [it.bold, { name: "font-weight", value: "bold" }],
      [it.italic, { name: "font-style", value: "italic" }],
    ])
    .filter((entry) => entry[0])
    .map((entry) => entry[1]);
}

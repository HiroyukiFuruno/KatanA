function katanaNormalizeWrappedDrawioTextLabels(svg) {
  katanaDrawioSourceCellLabels().forEach((entry) => {
    katanaNormalizeWrappedDrawioTextLabel(svg, entry);
  });
}

function katanaNormalizeWrappedDrawioTextLabel(svg, entry) {
  [katanaDrawioCellGroup(svg, entry.id)]
    .filter(Boolean)
    .filter((group) => katanaShouldNormalizeWrappedDrawioLabel(group, entry))
    .forEach((group) => {
      katanaReplaceWrappedDrawioTextLabel(group, entry);
    });
}

function katanaShouldNormalizeWrappedDrawioLabel(group, entry) {
  if (katanaDrawioGroupHasForeignObject(group)) return false;
  return [katanaDrawioCellShapeBox(group)]
    .filter(Boolean)
    .map((box) => katanaWrappedDrawioLabelNeeded(entry, box))
    .concat([false])[0];
}

function katanaDrawioGroupHasForeignObject(group) {
  return group.querySelectorAll("foreignObject").length > 0;
}

function katanaWrappedDrawioLabelNeeded(entry, box) {
  return [
    katanaDrawioStyleValue(entry.style, "whiteSpace") === "wrap",
    katanaDrawioTextNeedsWrap(entry.label, box, katanaDrawioFontSize(entry.style)),
  ].every(Boolean);
}

function katanaDrawioTextNeedsWrap(label, box, fontSize) {
  const lineLimit = katanaDrawioMaxLineChars(box, fontSize);
  return katanaDrawioTextLines(label).some((line) => line.length > lineLimit);
}

function katanaReplaceWrappedDrawioTextLabel(group, entry) {
  [katanaDrawioCellShapeBox(group)].filter(Boolean).forEach((box) => {
    const label = katanaCreateWrappedDrawioTextLabel(entry, box);
    katanaDrawioTextNodes(group).forEach(katanaRemoveDrawioNode);
    group.appendChild(label);
    katanaClipWrappedDrawioTextLabel(label, box);
  });
}

function katanaDrawioTextNodes(group) {
  return Array.from(group.querySelectorAll("text"));
}

function katanaCreateWrappedDrawioTextLabel(entry, box) {
  const style = entry.style;
  const fontSize = katanaDrawioFontSize(entry.style);
  const lines = katanaDrawioWrappedTextLines(entry.label, box, fontSize);
  const text = katanaWrappedDrawioTextElement(box, style, lines, fontSize);
  lines.forEach((line, index) => {
    text.appendChild(katanaWrappedDrawioTextLine(text, line, index, fontSize));
  });
  return text;
}

function katanaWrappedDrawioTextElement(box, style, lines, fontSize) {
  const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
  const alignment = katanaDrawioTextAlignment(style);
  text.setAttribute("x", katanaDrawioAlignedTextX(box, alignment));
  text.setAttribute("y", katanaDrawioAlignedTextY(box, style, lines, fontSize));
  text.setAttribute("fill", katanaDrawioTextColor(style));
  text.setAttribute("font-family", "Helvetica");
  text.setAttribute("font-size", `${fontSize}px`);
  text.setAttribute("text-anchor", alignment.anchor);
  return text;
}

function katanaWrappedDrawioTextLine(text, line, index, fontSize) {
  const tspan = document.createElementNS("http://www.w3.org/2000/svg", "tspan");
  tspan.setAttribute("x", text.getAttribute("x"));
  tspan.setAttribute("dy", katanaWrappedDrawioLineDy(index, fontSize));
  tspan.textContent = line;
  return tspan;
}

function katanaDrawioWrappedTextLines(label, box, fontSize) {
  const lineLimit = katanaDrawioMaxLineChars(box, fontSize);
  return katanaDrawioTextLines(label).flatMap((line) => katanaDrawioWrapTextLine(line, lineLimit));
}

function katanaDrawioWrapTextLine(line, lineLimit) {
  return String(line)
    .split(/\s+/)
    .filter(Boolean)
    .reduce(katanaAppendDrawioWrappedWord(lineLimit), []);
}

function katanaAppendDrawioWrappedWord(lineLimit) {
  return (lines, word) => katanaAppendDrawioWrappedLine(lines, word, lineLimit);
}

function katanaAppendDrawioWrappedLine(lines, word, lineLimit) {
  const candidate = [katanaDrawioLastTextLine(lines), word].filter(Boolean).join(" ");
  return candidate.length <= lineLimit
    ? katanaDrawioReplaceLastTextLine(lines, candidate)
    : lines.concat([word]);
}

function katanaDrawioLastTextLine(lines) {
  return lines.at(-1) ?? "";
}

function katanaDrawioReplaceLastTextLine(lines, line) {
  return lines.slice(0, -1).concat([line]);
}

function katanaDrawioMaxLineChars(box, fontSize) {
  return Math.max(1, Math.floor((box.width - 8) / katanaDrawioAverageCharWidth(fontSize)));
}

function katanaDrawioAverageCharWidth(fontSize) {
  return fontSize * 0.55;
}

function katanaDrawioAlignedTextX(box, alignment) {
  return box.x + alignment.offset + box.width * alignment.ratio;
}

function katanaDrawioAlignedTextY(box, style, lines, fontSize) {
  return katanaDrawioTextVerticalAlignment(style).y(box, lines, fontSize);
}

function katanaDrawioStyleMap(style) {
  return new Map(
    String(style)
      .split(";")
      .map(katanaDrawioStyleEntry)
      .filter((entry) => entry[0]),
  );
}

function katanaDrawioStyleEntry(value) {
  const parts = String(value).split("=").concat(["", ""]);
  return [parts[0], parts[1]];
}

function katanaDrawioStyleValue(style, name) {
  return katanaDrawioStyleMap(style).get(name) ?? "";
}

function katanaDrawioFontSize(style) {
  const value = Number(katanaDrawioStyleValue(style, "fontSize"));
  return [value]
    .filter(Number.isFinite)
    .filter((it) => it > 0)
    .concat([12])[0];
}

function katanaDrawioTextColor(style) {
  return katanaDrawioStyleValueFromMap(style, "fontColor", "#000000");
}

function katanaDrawioStyleValueFromMap(style, name, fallback) {
  return katanaDrawioStyleMap(style).get(name) ?? fallback;
}

function katanaDrawioTextAlignment(style) {
  return (
    KATANA_DRAWIO_TEXT_ALIGNMENT.get(katanaDrawioStyleValueFromMap(style, "align", "center")) ??
    KATANA_DRAWIO_TEXT_ALIGNMENT.get("center")
  );
}

function katanaDrawioTextVerticalAlignment(style) {
  return (
    KATANA_DRAWIO_TEXT_VERTICAL_ALIGNMENT.get(
      katanaDrawioStyleValueFromMap(style, "verticalAlign", "middle"),
    ) ?? KATANA_DRAWIO_TEXT_VERTICAL_ALIGNMENT.get("middle")
  );
}

function katanaWrappedDrawioLineDy(index, fontSize) {
  return ["0", String(Math.ceil(fontSize * 1.2))][Number(index > 0)];
}

const KATANA_DRAWIO_TEXT_ALIGNMENT = new Map([
  ["left", { anchor: "start", offset: 4, ratio: 0 }],
  ["center", { anchor: "middle", offset: 0, ratio: 0.5 }],
  ["right", { anchor: "end", offset: -4, ratio: 1 }],
]);

const KATANA_DRAWIO_TEXT_VERTICAL_ALIGNMENT = new Map([
  ["top", { y: (box, _lines, fontSize) => box.y + fontSize }],
  [
    "middle",
    {
      y: (box, lines, fontSize) =>
        box.y + box.height / 2 + fontSize / 2 - ((lines.length - 1) * fontSize * 1.2) / 2,
    },
  ],
  ["bottom", { y: (box, lines, fontSize) => box.y + box.height - lines.length * fontSize }],
]);

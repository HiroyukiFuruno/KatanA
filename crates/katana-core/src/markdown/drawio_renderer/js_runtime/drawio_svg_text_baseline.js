const KATANA_DRAWIO_TEXT_BASELINE_OFFSET = 0;

function katanaNormalizeDrawioTextBaselines(svg) {
  Array.from(svg.querySelectorAll("text")).forEach(katanaNormalizeDrawioTextBaseline);
}

function katanaNormalizeDrawioTextBaseline(text) {
  katanaNormalizeDrawioEncodedLineBreaks(text);
  const y = Number(text.getAttribute("y"));
  if (Number.isFinite(y)) {
    text.setAttribute("y", String(y - KATANA_DRAWIO_TEXT_BASELINE_OFFSET));
  }
}

function katanaNormalizeDrawioEncodedLineBreaks(text) {
  const lines = decodeHtmlEntities(text.textContent).split(/\n+/).filter(Boolean);
  if (lines.length < 2) {
    return;
  }
  text.replaceChildren(
    ...lines.map((line, index) => katanaDrawioLineBreakTspan(text, line, index)),
  );
}

function katanaDrawioLineBreakTspan(text, line, index) {
  const tspan = document.createElementNS("http://www.w3.org/2000/svg", "tspan");
  tspan.setAttribute("x", text.getAttribute("x"));
  tspan.setAttribute("dy", katanaDrawioTextLineDy(index, katanaDrawioTextFontSize(text)));
  tspan.textContent = line;
  return tspan;
}

function katanaDrawioTextFontSize(text) {
  return Number.parseFloat(text.getAttribute("font-size")) || 12;
}

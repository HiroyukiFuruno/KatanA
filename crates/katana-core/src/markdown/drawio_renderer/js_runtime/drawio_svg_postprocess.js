function katanaPostprocessDrawioSvg(svg) {
  katanaRemoveOversizedDrawioLabelBackgrounds(svg);
  katanaFillMissingDrawioTextLabels(svg);
  katanaNormalizeWrappedDrawioTextLabels(svg);
  katanaInstallDrawioHtmlTextLabels(svg);
  katanaNormalizeDrawioHtmlTables(svg);
  katanaNormalizeRichDrawioTextLabels(svg);
  katanaNormalizeDrawioTextBaselines(svg);
  katanaNormalizeDrawioForeignObjects(svg);
  katanaNormalizeDrawioGeometry(svg);
  if (!katanaDrawioRequestSource().includes("mxgraph.aws")) {
    katanaCropDrawioSvgToContent(svg);
  }
  katanaApplyDrawioShadows(svg);
  katanaNormalizeDrawioSvgColors(svg);
  katanaInstallDrawioPageBackground(svg);
}

function katanaInstallDrawioPageBackground(svg) {
  const box = katanaDrawioSvgBox(svg);
  const color = katanaDrawioPageBackgroundColor(svg);
  if (!color) {
    return;
  }
  const rect = katanaCreateDrawioBackgroundRect(box, color);
  katanaSetDrawioPageBackgroundStyle(svg, color);
  svg.insertBefore(rect, svg.firstChild);
}

function katanaDrawioPageBackgroundColor(svg) {
  return [
    katanaDrawioStyleBackground(svg.getAttribute("style")),
    katanaDrawioRequestBackground(),
  ].filter(katanaIsVisibleDrawioBackground)[0];
}

function katanaDrawioRequestBackground() {
  return globalThis.__katanaDrawioRequest?.background ?? "";
}

function katanaDrawioStyleBackground(style) {
  const match = String(style).match(/background(?:-color)?:\s*([^;]+)/);
  return [match].filter(Boolean).map(katanaDrawioMatchValue).concat([""])[0];
}

function katanaDrawioMatchValue(match) {
  return match[1].trim();
}

function katanaIsVisibleDrawioBackground(color) {
  return !["", "transparent"].includes(String(color).toLowerCase());
}

function katanaSetDrawioPageBackgroundStyle(svg, color) {
  svg.setAttribute("style", `background: ${color}; background-color: ${color};`);
}

function katanaCreateDrawioBackgroundRect(box, color) {
  const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
  rect.setAttribute("data-katana-drawio-background", "true");
  rect.setAttribute("x", box.x);
  rect.setAttribute("y", box.y);
  rect.setAttribute("width", box.width);
  rect.setAttribute("height", box.height);
  rect.setAttribute("fill", color);
  return rect;
}

function katanaDrawioSvgBox(svg) {
  return katanaParsedDrawioViewBox(svg.getAttribute("viewBox")) ?? katanaDrawioSizeBox(svg);
}

function katanaParsedDrawioViewBox(value) {
  const parts = String(value).trim().split(/\s+/).map(Number);
  return [parts].filter(katanaIsDrawioBoxParts).map(katanaDrawioBoxFromParts).concat([null])[0];
}

function katanaIsDrawioBoxParts(parts) {
  return [parts.length === 4, parts.every(Number.isFinite)].every(Boolean);
}

function katanaDrawioBoxFromParts(parts) {
  return { x: parts[0], y: parts[1], width: parts[2], height: parts[3] };
}

function katanaDrawioSizeBox(svg) {
  return {
    x: 0,
    y: 0,
    width: katanaDrawioCssPixels(svg.getAttribute("width")),
    height: katanaDrawioCssPixels(svg.getAttribute("height")),
  };
}

function katanaDrawioCssPixels(value) {
  const pixels = Number(String(value).replace(/px$/, ""));
  return [pixels].filter(Number.isFinite).concat([0])[0];
}

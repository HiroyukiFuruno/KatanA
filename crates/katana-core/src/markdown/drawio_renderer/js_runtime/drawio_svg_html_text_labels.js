function katanaInstallDrawioHtmlTextLabels(svg) {
  katanaDrawioHtmlTextLabelEntries().forEach((entry) => {
    katanaInstallDrawioHtmlTextLabel(svg, entry);
  });
}

function katanaDrawioHtmlTextLabelEntries() {
  return Array.from(katanaDrawioRequestSource().matchAll(/<mxCell\b([^>]*)>/g))
    .map(katanaDrawioHtmlTextLabelEntry)
    .filter(katanaHasDrawioHtmlTextLabel);
}

function katanaDrawioHtmlTextLabelEntry(match) {
  const attributes = katanaDrawioXmlAttributes(match[1]);
  const html = katanaDrawioCellAttribute(attributes, "value");
  return {
    id: katanaDrawioCellAttribute(attributes, "id"),
    html: katanaDrawioHtmlTextLabelHtml(html),
    label: katanaDrawioHtmlLabelText(html),
    style: katanaDrawioCellAttribute(attributes, "style"),
  };
}

function katanaDrawioHtmlTextLabelHtml(html) {
  return katanaDrawioIsDarkMode()
    ? String(html).replaceAll('color="#000000"', 'color="#ededed"')
    : html;
}

function katanaHasDrawioHtmlTextLabel(entry) {
  return [
    entry.id,
    entry.html,
    entry.label,
    katanaDrawioStyleValue(entry.style, "html") === "1",
  ].every(Boolean);
}

function katanaInstallDrawioHtmlTextLabel(svg, entry) {
  [katanaDrawioCellGroup(svg, entry.id)]
    .filter(Boolean)
    .filter((group) => !katanaDrawioGroupHasForeignObject(group))
    .map((group) => katanaDrawioHtmlTextLabelContext(group, entry))
    .filter(katanaHasDrawioHtmlTextLabelContext)
    .forEach(katanaApplyDrawioHtmlTextLabel);
}

function katanaDrawioHtmlTextLabelContext(group, entry) {
  return {
    group,
    entry,
    box: katanaDrawioCellShapeBox(group),
    fallback: katanaDrawioTextNodes(group),
  };
}

function katanaHasDrawioHtmlTextLabelContext(context) {
  return [context.box, context.fallback.length > 0].every(Boolean);
}

function katanaApplyDrawioHtmlTextLabel(context) {
  const label = katanaCreateDrawioHtmlTextLabel(context.entry, context.box, context.fallback);
  context.group.appendChild(label);
}

function katanaCreateDrawioHtmlTextLabel(entry, box, fallback) {
  const group = katanaCreateDrawioSvgElement("g");
  const scaled = katanaCreateDrawioSvgElement("g");
  const switchNode = katanaCreateDrawioSvgElement("switch");
  scaled.setAttribute("transform", "scale(0.9999999999999999)");
  switchNode.appendChild(katanaCreateDrawioHtmlTextForeignObject(entry, box));
  fallback.forEach((node) => {
    switchNode.appendChild(node);
  });
  scaled.appendChild(switchNode);
  group.appendChild(scaled);
  return group;
}

function katanaCreateDrawioHtmlTextForeignObject(entry, box) {
  const foreignObject = katanaCreateDrawioSvgElement("foreignObject");
  foreignObject.setAttribute(
    "style",
    `overflow: visible; text-align: ${katanaDrawioTextAlign(entry.style)};`,
  );
  foreignObject.setAttribute("pointer-events", "none");
  foreignObject.setAttribute("width", "101%");
  foreignObject.setAttribute("height", "101%");
  foreignObject.setAttribute(
    "requiredFeatures",
    "http://www.w3.org/TR/SVG11/feature#Extensibility",
  );
  foreignObject.appendChild(katanaCreateDrawioHtmlTextOuterDiv(entry, box));
  return foreignObject;
}

function katanaCreateDrawioHtmlTextOuterDiv(entry, box) {
  const div = katanaCreateDrawioHtmlElement("div");
  div.setAttribute("xmlns", "http://www.w3.org/1999/xhtml");
  div.setAttribute("style", katanaDrawioHtmlTextOuterStyle(entry.style, box));
  div.appendChild(katanaCreateDrawioHtmlTextBoxDiv(entry));
  return div;
}

function katanaCreateDrawioHtmlTextBoxDiv(entry) {
  const div = katanaCreateDrawioHtmlElement("div");
  div.setAttribute("style", katanaDrawioHtmlTextBoxStyle(entry.style));
  div.appendChild(katanaCreateDrawioHtmlTextContentDiv(entry));
  return div;
}

function katanaCreateDrawioHtmlTextContentDiv(entry) {
  const div = katanaCreateDrawioHtmlElement("div");
  div.setAttribute("style", katanaDrawioHtmlTextContentStyle(entry.style));
  div.innerHTML = entry.html;
  return div;
}

function katanaCreateDrawioSvgElement(tagName) {
  return document.createElementNS("http://www.w3.org/2000/svg", tagName);
}

function katanaCreateDrawioHtmlElement(tagName) {
  return document.createElementNS("http://www.w3.org/1999/xhtml", tagName);
}

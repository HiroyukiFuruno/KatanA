function katanaNormalizeDrawioHtmlTables(svg) {
  katanaDrawioHtmlTableEntries().forEach((entry) => {
    katanaNormalizeDrawioHtmlTable(svg, entry);
  });
}

function katanaDrawioHtmlTableEntries() {
  return Array.from(katanaDrawioRequestSource().matchAll(/<mxCell\b([^>]*)>/g))
    .map(katanaDrawioHtmlTableEntry)
    .filter(katanaHasDrawioHtmlTableEntry);
}

function katanaDrawioHtmlTableEntry(match) {
  const attributes = katanaDrawioXmlAttributes(match[1]);
  return {
    id: katanaDrawioCellAttribute(attributes, "id"),
    html: katanaDrawioCellAttribute(attributes, "value"),
    style: katanaDrawioCellAttribute(attributes, "style"),
  };
}

function katanaHasDrawioHtmlTableEntry(entry) {
  return [
    entry.id,
    katanaDrawioStyleValue(entry.style, "html") === "1",
    katanaDrawioStyleValue(entry.style, "overflow") === "fill",
    entry.html.trim().toLowerCase().startsWith("<table"),
  ].every(Boolean);
}

function katanaNormalizeDrawioHtmlTable(svg, entry) {
  [katanaDrawioHtmlTableContext(svg, entry)]
    .filter(katanaHasDrawioHtmlTableContext)
    .forEach(katanaApplyDrawioHtmlTable);
}

function katanaDrawioHtmlTableContext(svg, entry) {
  const group = katanaDrawioCellGroup(svg, entry.id);
  return {
    entry,
    group,
    box: katanaDrawioHtmlTableShapeBox(group),
    foreignObject: katanaDrawioHtmlTableForeignObject(group),
  };
}

function katanaDrawioHtmlTableShapeBox(group) {
  return [group].filter(Boolean).map(katanaDrawioCellShapeBox).concat([null])[0];
}

function katanaDrawioHtmlTableForeignObject(group) {
  return [group]
    .filter(Boolean)
    .map((it) => it.querySelector("foreignObject"))
    .concat([null])[0];
}

function katanaHasDrawioHtmlTableContext(context) {
  return [context.group, context.box, context.foreignObject].every(Boolean);
}

function katanaApplyDrawioHtmlTable(context) {
  const foreignObject = context.foreignObject;
  foreignObject.setAttribute("style", "overflow: visible; text-align: left;");
  foreignObject.setAttribute("pointer-events", "none");
  foreignObject.setAttribute("width", "101%");
  foreignObject.setAttribute("height", "101%");
  foreignObject.setAttribute(
    "requiredFeatures",
    "http://www.w3.org/TR/SVG11/feature#Extensibility",
  );
  katanaReplaceDrawioHtmlTableChildren(foreignObject, context.entry, context.box);
}

function katanaReplaceDrawioHtmlTableChildren(foreignObject, entry, box) {
  Array.from(foreignObject.childNodes).forEach((child) => {
    child.remove();
  });
  foreignObject.appendChild(katanaCreateDrawioHtmlTableOuterDiv(entry, box));
}

function katanaCreateDrawioHtmlTableOuterDiv(entry, box) {
  const div = katanaCreateDrawioHtmlElement("div");
  div.setAttribute("xmlns", "http://www.w3.org/1999/xhtml");
  div.setAttribute("style", katanaDrawioHtmlTableOuterStyle(box));
  div.appendChild(katanaCreateDrawioHtmlTableBoxDiv(entry, box));
  return div;
}

function katanaCreateDrawioHtmlTableBoxDiv(entry, box) {
  const div = katanaCreateDrawioHtmlElement("div");
  div.setAttribute("style", katanaDrawioHtmlTableBoxStyle(entry.style, box));
  div.appendChild(katanaCreateDrawioHtmlTableContentDiv(entry, box));
  return div;
}

function katanaCreateDrawioHtmlTableContentDiv(entry, box) {
  const div = katanaCreateDrawioHtmlElement("div");
  div.setAttribute("style", katanaDrawioHtmlTableContentStyle(entry.style, box));
  div.innerHTML = katanaDrawioHtmlTableMarkup(entry.html);
  return div;
}

function katanaDrawioHtmlTableMarkup(html) {
  return katanaDrawioIsDarkMode() ? katanaDrawioDarkHtmlTableMarkup(html) : html;
}

function katanaDrawioDarkHtmlTableMarkup(html) {
  return KATANA_DRAWIO_HTML_TABLE_DARK_COLORS.reduce(
    (text, entry) => katanaDrawioReplaceHtmlTableColor(text, entry),
    html,
  ).replace(/color:\s*#ffffff/gi, "color: #121212");
}

function katanaDrawioReplaceHtmlTableColor(text, entry) {
  return text.replace(new RegExp(katanaDrawioEscapeRegExp(entry[0]), "gi"), entry[1]);
}

function katanaDrawioEscapeRegExp(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function katanaDrawioHtmlTableOuterStyle(box) {
  return [
    "display: flex",
    "align-items: unsafe flex-start",
    "justify-content: unsafe flex-start",
    `width: ${Math.round(box.width + 2)}px`,
    `height: ${Math.round(box.height)}px`,
    `padding-top: ${Math.round(box.y)}px`,
    `margin-left: ${Math.round(box.x)}px`,
  ].join("; ");
}

function katanaDrawioHtmlTableBoxStyle(style, box) {
  return [
    "box-sizing: border-box",
    "font-size: 0",
    `text-align: ${katanaDrawioTextAlign(style)}`,
    `width: ${Math.round(box.width)}px`,
    `height: ${Math.round(box.height)}px`,
    "overflow: hidden",
    `color: ${katanaDrawioTextColor(style)}`,
    "",
  ].join("; ");
}

function katanaDrawioHtmlTableContentStyle(style, _box) {
  return [
    "display: inline-block",
    `font-size: ${katanaDrawioFontSize(style)}px`,
    `font-family: ${katanaDrawioHtmlFontFamily(style)}`,
    `color: ${katanaDrawioHtmlTextColor(style)}`,
    "line-height: 1.2",
    "pointer-events: all",
    "width: 100%",
    "height: 100%",
    "white-space: nowrap",
    "",
  ].join("; ");
}

const KATANA_DRAWIO_HTML_TABLE_DARK_COLORS = [
  ["#98bf21", "#4a6b00"],
  ["#a7c942", "#405e00"],
  ["#eaf2d3", "#1d2409"],
];

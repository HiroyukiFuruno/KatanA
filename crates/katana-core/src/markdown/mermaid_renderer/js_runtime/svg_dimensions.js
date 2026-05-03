function katanaReadViewBox(svg) {
  const match = svg.match(/viewBox="([^"]+)"/);
  if (!match) {
    return null;
  }
  return katanaViewBoxValues(match[1]);
}

function katanaViewBoxValues(value) {
  const values = value.split(/\s+/).map((it) => Number(it));
  if (!katanaValidViewBoxValues(values)) {
    return null;
  }
  return values;
}

function katanaValidViewBoxValues(values) {
  return [values.length === 4, values.every((value) => Number.isFinite(value))].every(Boolean);
}

function katanaNormalizedSvgDimensions(svg) {
  const viewBox = katanaReadViewBox(svg);
  if (!viewBox) {
    return null;
  }
  return katanaPickSvgDimensions(viewBox, katanaContentBox(svg), svg);
}

function katanaPickSvgDimensions(viewBox, contentBox, svg) {
  if (!katanaUseContentBox(viewBox, contentBox, svg)) {
    return viewBox;
  }
  return contentBox;
}

function katanaUseContentBox(viewBox, contentBox, svg) {
  if (!contentBox) {
    return false;
  }
  return katanaUseExistingContentBox(viewBox, contentBox, svg);
}

function katanaUseExistingContentBox(viewBox, contentBox, svg) {
  return [
    katanaRoleContentBoxDecision(svg, "class", contentBox[3] < viewBox[3] - 8),
    katanaRoleContentBoxDecision(svg, "gitGraph", contentBox[3] < viewBox[3] - 8),
    katanaRoleContentBoxDecision(svg, "ishikawa", contentBox[2] < viewBox[2] - 8),
    katanaUseNonClassContentBox(viewBox, contentBox, svg),
  ].find((decision) => decision !== null);
}

function katanaRoleContentBoxDecision(svg, role, decision) {
  return [decision].filter(() => svg.includes(`aria-roledescription="${role}"`)).concat([null])[0];
}

function katanaUseNonClassContentBox(viewBox, contentBox, svg) {
  if (katanaKeepsOriginalViewBox(svg)) {
    return false;
  }
  return katanaIsOversizedViewBox(viewBox, contentBox);
}

function katanaKeepsOriginalViewBox(svg) {
  return [
    "venn",
    "gantt",
    "pie",
    "er",
    "treemap",
    "mindmap",
    "block",
    "kanban",
    "xychart",
    "wardley-beta",
  ].some((role) => svg.includes(`aria-roledescription="${role}"`));
}

function katanaIsOversizedViewBox(viewBox, contentBox) {
  return [
    viewBox[2] > 1600,
    viewBox[3] > 1200,
    katanaContentOverflowsViewBox(viewBox, contentBox),
    contentBox[2] > viewBox[2] * 1.2,
    contentBox[3] > viewBox[3] * 1.2,
  ].some(Boolean);
}

function katanaContentOverflowsViewBox(viewBox, contentBox) {
  return [
    contentBox[0] < viewBox[0] - 8,
    contentBox[1] < viewBox[1] - 8,
    contentBox[0] + contentBox[2] > viewBox[0] + viewBox[2] + 8,
    contentBox[1] + contentBox[3] > viewBox[1] + viewBox[3] + 8,
  ].some(Boolean);
}

function katanaSetSvgDimension(svg, name, value) {
  const pattern = new RegExp(`\\s${name}="[^"]*"`);
  return svg.replace(/<svg\b[^>]*>/, (tag) => katanaSvgTagWithDimension(tag, pattern, name, value));
}

function katanaSvgTagWithDimension(tag, pattern, name, value) {
  if (pattern.test(tag)) {
    return tag.replace(pattern, ` ${name}="${value}"`);
  }
  return tag.replace("<svg", `<svg ${name}="${value}"`);
}

function katanaSetSvgViewBox(svg, value) {
  return svg.replace(/viewBox="[^"]*"/, `viewBox="${value}"`);
}

function katanaSetSvgMaxWidth(svg, width) {
  if (/style="[^"]*max-width:/.test(svg)) {
    return svg.replace(/max-width:\s*[^;"]+;?/g, `max-width: ${width}px;`);
  }
  return svg;
}

function katanaEnsureSvgAttr(tag, name, value) {
  const pattern = new RegExp(`\\s${name}="`);
  if (pattern.test(tag)) {
    return tag;
  }
  return tag.replace(/>$/, ` ${name}="${value}">`);
}

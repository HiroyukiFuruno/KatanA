function katanaNormalizeDrawioSvgColors(svg) {
  [svg, ...svg.querySelectorAll("*")].forEach(katanaNormalizeDrawioElementColors);
}

function katanaNormalizeDrawioElementColors(element) {
  katanaResolveDrawioLightDarkStyleColors(element);
  ["fill", "stroke", "stop-color"].forEach((name) => {
    katanaNormalizeDrawioColorAttribute(element, name);
  });
  katanaNormalizeDrawioStyleTextColor(element);
}

function katanaNormalizeDrawioColorAttribute(element, name) {
  const color = element.getAttribute(name);
  if (color) {
    element.setAttribute(name, katanaDrawioResolvedAttributeColor(element, name, color));
  }
}

function katanaDrawioResolvedAttributeColor(element, name, color) {
  return katanaDrawioShouldResolveTextColor(element, name)
    ? katanaDrawioResolvedTextColor(color)
    : katanaDrawioResolvedColor(element, name, color);
}

function katanaDrawioShouldResolveTextColor(element, name) {
  return [element.localName === "text", name === "fill"].every(Boolean);
}

function katanaDrawioResolvedColor(element, name, color) {
  return (
    katanaDrawioPaintServerPassthrough(color) ||
    katanaDrawioResolvedNonPaintColor(element, name, color)
  );
}

function katanaDrawioPaintServerPassthrough(color) {
  return katanaDrawioIsPaintServer(color) ? color : "";
}

function katanaDrawioResolvedNonPaintColor(element, name, color) {
  const value = katanaDrawioColorKey(color);
  return (
    katanaDrawioShapeMappedColor(element, name, value) ||
    katanaDrawioContextMappedColor(element, name, value) ||
    katanaDrawioLightDarkExactColor(value) ||
    katanaDrawioThemeMappedColor(element, name, value)
  );
}

function katanaDrawioLightDarkExactColor(value) {
  const colors = KATANA_DRAWIO_LIGHT_DARK_COLORS.get(value);
  return colors?.[Number(katanaDrawioIsDarkMode())] ?? "";
}

function katanaDrawioThemeMappedColor(element, name, value) {
  return (
    katanaDrawioColorMapForTheme().get(value) ||
    katanaDrawioFallbackThemeColor(element, name, value) ||
    value
  );
}

function katanaDrawioContextMappedColor(element, name, value) {
  return [
    katanaDrawioContextOriginalColor(element, name, value),
    katanaDrawioContextColorMapForTheme().get(katanaDrawioContextColorKey(element, name, value)),
  ]
    .filter(Boolean)
    .concat([""])[0];
}

function katanaDrawioShapeMappedColor(element, name, value) {
  return [katanaDrawioShapeColorMap(element).get(`${name}|${value}`)]
    .filter(Boolean)
    .concat([""])[0];
}

function katanaDrawioShapeColorMap(element) {
  return KATANA_DRAWIO_SHAPE_COLOR_MAPS[Number(katanaDrawioIsAws3dElement(element))];
}

function katanaDrawioIsAws3dElement(element) {
  return [
    katanaDrawioIsDarkMode(),
    katanaDrawioElementCellShape(element).startsWith("mxgraph.aws3d."),
  ].every(Boolean);
}

function katanaDrawioColorMapForTheme() {
  return KATANA_DRAWIO_COLOR_BY_THEME[Number(katanaDrawioIsDarkMode())];
}

function katanaDrawioContextColorMapForTheme() {
  return KATANA_DRAWIO_CONTEXT_COLOR_BY_THEME[Number(katanaDrawioIsDarkMode())];
}

function katanaDrawioColorKey(color) {
  return String(color).trim().toLowerCase();
}

function katanaDrawioContextColorKey(element, name, value) {
  return [element.tagName, name, value].map(katanaDrawioColorKey).join("|");
}

function katanaDrawioIsDarkMode() {
  return globalThis.__katanaDrawioRequest?.dark_mode === true;
}

const KATANA_DRAWIO_LIGHT_DARK_COLORS = new Map([
  ["light-dark(#000000, #ffffff)", ["#000000", "#ffffff"]],
  ["light-dark(rgb(255, 255, 255), rgb(18, 18, 18))", ["rgb(255, 255, 255)", "rgb(18, 18, 18)"]],
  ["light-dark(rgb(218, 232, 252), rgb(29, 41, 59))", ["rgb(218, 232, 252)", "rgb(29, 41, 59)"]],
  [
    "light-dark(rgb(108, 142, 191), rgb(92, 121, 163))",
    ["rgb(108, 142, 191)", "rgb(92, 121, 163)"],
  ],
  ["light-dark(rgb(213, 232, 212), rgb(31, 47, 30))", ["rgb(213, 232, 212)", "rgb(31, 47, 30)"]],
  ["light-dark(rgb(130, 179, 102), rgb(68, 110, 44))", ["rgb(130, 179, 102)", "rgb(68, 110, 44)"]],
  ["light-dark(rgb(255, 242, 204), rgb(40, 29, 0))", ["rgb(255, 242, 204)", "rgb(40, 29, 0)"]],
  ["light-dark(rgb(214, 182, 86), rgb(109, 81, 0))", ["rgb(214, 182, 86)", "rgb(109, 81, 0)"]],
  ["light-dark(rgb(248, 206, 204), rgb(81, 45, 43))", ["rgb(248, 206, 204)", "rgb(81, 45, 43)"]],
  ["light-dark(rgb(184, 84, 80), rgb(215, 129, 126))", ["rgb(184, 84, 80)", "rgb(215, 129, 126)"]],
  ["light-dark(rgb(225, 213, 231), rgb(57, 47, 63))", ["rgb(225, 213, 231)", "rgb(57, 47, 63)"]],
  [
    "light-dark(rgb(150, 115, 166), rgb(149, 119, 163))",
    ["rgb(150, 115, 166)", "rgb(149, 119, 163)"],
  ],
  ["light-dark(rgb(255, 230, 204), rgb(54, 33, 10))", ["rgb(255, 230, 204)", "rgb(54, 33, 10)"]],
  ["light-dark(rgb(215, 155, 0), rgb(153, 101, 0))", ["rgb(215, 155, 0)", "rgb(153, 101, 0)"]],
]);

const KATANA_DRAWIO_DARK_COLOR_BY_LIGHT_COLOR = new Map([
  ["#000000", "#ffffff"],
  ["rgb(0, 0, 0)", "rgb(255, 255, 255)"],
  ["#232f3e", "#bdc7d4"],
  ["rgb(35, 47, 62)", "rgb(189, 199, 212)"],
  ["#23445d", "#a0bcd2"],
  ["rgb(35, 68, 93)", "rgb(160, 188, 210)"],
  ["#545b64", "#9aa0a8"],
  ["rgb(84, 91, 100)", "rgb(154, 160, 168)"],
  ["#5a6c86", "#8494aa"],
  ["rgb(90, 108, 134)", "rgb(132, 148, 170)"],
  ["#eaeded", "#1f2222"],
  ["rgb(234, 237, 237)", "rgb(31, 34, 34)"],
  ["#f59d56", "#9a4e11"],
  ["rgb(245, 157, 86)", "rgb(154, 78, 17)"],
  ["#116d5b", "#63b3a3"],
  ["rgb(17, 109, 91)", "rgb(99, 179, 163)"],
  ["#4ab29a", "#237d68"],
  ["rgb(74, 178, 154)", "rgb(35, 125, 104)"],
  ["#277116", "#72b264"],
  ["rgb(39, 113, 22)", "rgb(114, 178, 100)"],
  ["#60a337", "#4d872a"],
  ["rgb(96, 163, 55)", "rgb(77, 135, 42)"],
  ["#5a30b5", "#c8a4ff"],
  ["rgb(90, 48, 181)", "rgb(200, 164, 255)"],
  ["#945df2", "#a677f7"],
  ["rgb(148, 93, 242)", "rgb(166, 119, 247)"],
  ["#3334b9", "#afb0ff"],
  ["rgb(51, 52, 185)", "rgb(175, 176, 255)"],
  ["#4d72f3", "#6989f8"],
  ["rgb(77, 114, 243)", "rgb(105, 137, 248)"],
  ["#bc1356", "#ff97d1"],
  ["rgb(188, 19, 86)", "rgb(255, 151, 209)"],
  ["#f34482", "#ff6ba1"],
  ["rgb(243, 68, 130)", "rgb(255, 107, 161)"],
  ["#c7131f", "#ff9aa5"],
  ["rgb(199, 19, 31)", "rgb(255, 154, 165)"],
  ["#f54749", "#ff7072"],
  ["rgb(245, 71, 73)", "rgb(255, 112, 114)"],
  ["#be0917", "#ffa1ad"],
  ["rgb(190, 9, 23)", "rgb(255, 161, 173)"],
  ["#d05c17", "#e07c41"],
  ["rgb(208, 92, 23)", "rgb(224, 124, 65)"],
  ["#007cbd", "#3ea8e0"],
  ["rgb(0, 124, 189)", "rgb(62, 168, 224)"],
  ["#b3b3b3", "#535353"],
  ["rgb(179, 179, 179)", "rgb(83, 83, 83)"],
  ["#f4f4f4", "#1b1b1b"],
  ["rgb(244, 244, 244)", "rgb(27, 27, 27)"],
  ["#dae8fc", "#1d293b"],
  ["rgb(218, 232, 252)", "rgb(29, 41, 59)"],
  ["#6c8ebf", "#5c79a3"],
  ["rgb(108, 142, 191)", "rgb(92, 121, 163)"],
  ["#d5e8d4", "#1f2f1e"],
  ["rgb(213, 232, 212)", "rgb(31, 47, 30)"],
  ["#82b366", "#446e2c"],
  ["rgb(130, 179, 102)", "rgb(68, 110, 44)"],
  ["#fff2cc", "#281d00"],
  ["rgb(255, 242, 204)", "rgb(40, 29, 0)"],
  ["#d6b656", "#6d5100"],
  ["rgb(214, 182, 86)", "rgb(109, 81, 0)"],
  ["#f8cecc", "#512d2b"],
  ["rgb(248, 206, 204)", "rgb(81, 45, 43)"],
  ["#b85450", "#d7817e"],
  ["rgb(184, 84, 80)", "rgb(215, 129, 126)"],
  ["#e1d5e7", "#392f3f"],
  ["rgb(225, 213, 231)", "rgb(57, 47, 63)"],
  ["#9673a6", "#9577a3"],
  ["rgb(150, 115, 166)", "rgb(149, 119, 163)"],
  ["#ffe6cc", "#36210a"],
  ["rgb(255, 230, 204)", "rgb(54, 33, 10)"],
  ["#d79b00", "#996500"],
  ["rgb(215, 155, 0)", "rgb(153, 101, 0)"],
]);

const KATANA_DRAWIO_COLOR_BY_THEME = [new Map(), KATANA_DRAWIO_DARK_COLOR_BY_LIGHT_COLOR];

const KATANA_DRAWIO_DARK_CONTEXT_COLOR = new Map([["rect|fill|#ffffff", "#121212"]]);

const KATANA_DRAWIO_CONTEXT_COLOR_BY_THEME = [new Map(), KATANA_DRAWIO_DARK_CONTEXT_COLOR];

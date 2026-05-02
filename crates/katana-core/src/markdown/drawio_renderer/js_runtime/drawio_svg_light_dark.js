const KATANA_DRAWIO_LIGHT_DARK_PATTERN =
  /light-dark\((rgb\([^)]+\)|#[\da-f]{6}),\s*(rgb\([^)]+\)|#[\da-f]{6}|var\(--ge-dark-color,\s*#[\da-f]{6}\))\)/gi;

const KATANA_DRAWIO_STYLE_SVG_COLOR_ATTRIBUTES = ["fill", "stroke", "stop-color"];

function katanaResolveDrawioLightDarkStyleColors(element) {
  const style = element.getAttribute("style");
  if (!style?.includes("light-dark(")) {
    return;
  }

  const resolvedStyle = katanaResolvedDrawioLightDarkStyle(style);
  element.setAttribute("style", resolvedStyle);
  katanaApplyDrawioStyleColorAttributes(element, resolvedStyle);
}

function katanaResolvedDrawioLightDarkStyle(style) {
  return style.replace(KATANA_DRAWIO_LIGHT_DARK_PATTERN, (_match, light, dark) =>
    katanaDrawioLightDarkStyleChoice(light, dark),
  );
}

function katanaDrawioLightDarkStyleChoice(light, dark) {
  return katanaDrawioIsDarkMode()
    ? katanaDrawioLightDarkDarkStyleColor(dark)
    : String(light).trim();
}

function katanaDrawioLightDarkDarkStyleColor(dark) {
  const value = String(dark).trim();
  const fallback = value.match(/var\(--ge-dark-color,\s*(#[\da-f]{6})\)/i);
  return fallback?.[1] ?? value;
}

function katanaApplyDrawioStyleColorAttributes(element, style) {
  KATANA_DRAWIO_STYLE_SVG_COLOR_ATTRIBUTES.map((name) =>
    katanaDrawioStyleColorAttribute(style, name),
  )
    .filter(katanaHasDrawioStyleColorAttributeValue)
    .forEach((attribute) => {
      element.setAttribute(attribute.name, attribute.value);
    });
}

function katanaDrawioStyleColorAttribute(style, name) {
  return { name, value: katanaDrawioStylePropertyValue(style, name) };
}

function katanaHasDrawioStyleColorAttributeValue(attribute) {
  return attribute.value !== "";
}

function katanaDrawioStylePropertyValue(style, name) {
  return style
    .split(";")
    .map((declaration) => declaration.trim())
    .filter((declaration) => declaration.toLowerCase().startsWith(`${name}:`))
    .map(katanaDrawioStyleDeclarationValue)
    .filter(Boolean)
    .concat([""])[0];
}

function katanaDrawioStyleDeclarationValue(declaration) {
  return declaration.slice(declaration.indexOf(":") + 1).trim();
}

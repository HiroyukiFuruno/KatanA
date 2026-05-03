globalThis.getComputedStyle = (node) => katanaComputedStyle(node);

function katanaComputedStyle(node) {
  const box = katanaComputedStyleBox(node);
  return {
    width: `${box.width}px`,
    height: `${box.height}px`,
    display: "block",
    fontSize: katanaComputedFontSize(node),
    fontFamily: katanaComputedFontFamily(node),
    getPropertyValue(name) {
      return katanaComputedPropertyValue(node, this, name);
    },
  };
}

function katanaComputedFontSize(node) {
  return katanaComputedStyleValue(node, "font-size") || "16px";
}

function katanaComputedFontFamily(node) {
  return (
    katanaComputedStyleValue(node, "font-family") || "trebuchet ms, verdana, arial, sans-serif"
  );
}

function katanaComputedPropertyValue(node, style, name) {
  return katanaComputedStyleValue(node, name) || style[name] || "";
}

function katanaComputedStyleBox(node) {
  if (node) {
    return katanaElementClientBox(node);
  }
  return katanaBox(0, 0, 0, 0);
}

function katanaComputedStyleValue(node, name) {
  return (
    [katanaRawComputedStyleValue(node, name), katanaCssComputedStyleValue(node, name)]
      .filter(Boolean)
      .concat([katanaDefaultComputedStyleValue(name)])[0] ?? ""
  );
}

function katanaRawComputedStyleValue(node, name) {
  return node?.style?.getPropertyValue?.(name) ?? "";
}

function katanaDefaultComputedStyleValue(name) {
  if (/^(padding|margin|border).*/.test(name)) {
    return "0px";
  }
  return "";
}

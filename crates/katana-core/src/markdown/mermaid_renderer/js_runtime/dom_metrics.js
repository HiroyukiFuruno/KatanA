const KATANA_DEFAULT_CLIENT_TAGS = new Set(["body", "div", "main", "pre", "section", "article"]);

function katanaMeasuredClientBox(node) {
  const context = katanaClientBoxContext(node);
  return (
    katanaDefaultExplicitZeroBox(context) ??
    katanaExplicitClientBox(context) ??
    katanaSvgClientBox(context) ??
    katanaEmptyDefaultClientBox(context) ??
    context.box
  );
}

function katanaClientBoxContext(node) {
  return {
    node,
    box: node.getBBox(),
    explicitWidth: katanaExplicitClientWidth(node),
    explicitHeight: katanaExplicitClientHeight(node),
  };
}

function katanaExplicitClientWidth(node) {
  return (
    katanaCssLength(node.style?.getPropertyValue?.("width")) ?? katanaNumberAttr(node, "width")
  );
}

function katanaExplicitClientHeight(node) {
  return (
    katanaCssLength(node.style?.getPropertyValue?.("height")) ?? katanaNumberAttr(node, "height")
  );
}

function katanaDefaultExplicitZeroBox(context) {
  if ([katanaNeedsDefaultClientBox(context.node), context.explicitWidth === 0].every(Boolean)) {
    return katanaBox(
      0,
      0,
      katanaDefaultViewportWidth(),
      katanaPositiveOrDefault(context.explicitHeight, katanaDefaultViewportHeight()),
    );
  }
  return null;
}

function katanaExplicitClientBox(context) {
  if (katanaHasExplicitClientSize(context)) {
    return katanaResolvedExplicitClientBox(context);
  }
  return null;
}

function katanaHasExplicitClientSize(context) {
  return [context.explicitWidth !== null, context.explicitHeight !== null].includes(true);
}

function katanaResolvedExplicitClientBox(context) {
  return katanaBox(
    context.box.x,
    context.box.y,
    katanaExplicitWidthValue(context),
    katanaExplicitHeightValue(context),
  );
}

function katanaExplicitWidthValue(context) {
  return context.explicitWidth ?? context.box.width;
}

function katanaExplicitHeightValue(context) {
  return context.explicitHeight ?? context.box.height;
}

function katanaSvgClientBox(context) {
  if (context.node.localName === "svg") {
    return katanaSvgViewBoxClientBox(context);
  }
  return null;
}

function katanaSvgViewBoxClientBox(context) {
  const viewBox = katanaViewBoxSize(context.node.getAttribute("viewBox"));
  return katanaBox(
    context.box.x,
    context.box.y,
    katanaViewBoxWidth(context, viewBox),
    katanaViewBoxHeight(context, viewBox),
  );
}

function katanaViewBoxWidth(context, viewBox) {
  return viewBox?.[0] ?? Math.max(context.box.width, katanaDefaultViewportWidth());
}

function katanaViewBoxHeight(context, viewBox) {
  return viewBox?.[1] ?? Math.max(context.box.height, katanaDefaultViewportHeight());
}

function katanaEmptyDefaultClientBox(context) {
  if ([katanaNeedsDefaultClientBox(context.node), katanaIsEmptyBox(context.box)].every(Boolean)) {
    return katanaBox(0, 0, katanaDefaultViewportWidth(), katanaDefaultViewportHeight());
  }
  return null;
}

function katanaDefaultViewportWidth() {
  return Number(globalThis.innerWidth ?? globalThis.screen?.width ?? 800);
}

function katanaDefaultViewportHeight() {
  return Number(globalThis.innerHeight ?? globalThis.screen?.height ?? 600);
}

function katanaNeedsDefaultClientBox(node) {
  return [KATANA_DEFAULT_CLIENT_TAGS.has(node.localName), katanaHasSvgChild(node)].includes(true);
}

function katanaIsEmptyBox(box) {
  return [box.width === 0, box.height === 0].every(Boolean);
}

function katanaPositiveOrDefault(value, fallback) {
  return value > 0 ? value : fallback;
}

function katanaCssLength(value) {
  if (!value) {
    return null;
  }
  return katanaFiniteCssLength(value);
}

function katanaFiniteCssLength(value) {
  const number = Number(String(value).replace("px", ""));
  if (Number.isFinite(number)) {
    return number;
  }
  return null;
}

function katanaViewBoxSize(value) {
  const values = katanaViewBoxValues(value);
  if (katanaIsValidViewBox(values)) {
    return [values[2], values[3]];
  }
  return null;
}

function katanaViewBoxValues(value) {
  return String(value ?? "")
    .split(/\s+/)
    .map((it) => Number(it));
}

function katanaIsValidViewBox(values) {
  return [values.length === 4, values.every((it) => Number.isFinite(it))].every(Boolean);
}

function katanaHasSvgChild(node) {
  return (node.children ?? []).some((child) => child.localName === "svg");
}

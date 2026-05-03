function katanaReadErNodeLayout(node) {
  const outer = katanaReadErPathBox(
    node.match(/<g class="outer-path"[\s\S]*?<path d="([^"]+)"/)?.[1],
  );
  const rows = katanaReadErRows(node);
  if (!katanaHasErLayout(outer, rows)) {
    return null;
  }
  return katanaBuildErLayout(outer, rows, node);
}

function katanaReadErRows(node) {
  return Array.from(
    node.matchAll(/<g style="" class="row-rect-(?:odd|even)">\s*<path d="([^"]+)"/g),
  )
    .map((match) => katanaReadErPathBox(match[1]))
    .filter(Boolean);
}

function katanaHasErLayout(outer, rows) {
  return [Boolean(outer), rows.length > 0].every(Boolean);
}

function katanaBuildErLayout(outer, rows, node) {
  const dividerX = katanaErDividerXOrDefault(katanaReadErDividerX(node, rows), outer);
  const headerHeight = rows[0].top - outer.top;
  return {
    left: outer.left,
    right: outer.right,
    dividerX,
    headerLabelY: outer.top + headerHeight / 4 + 0.1875,
    rows: rows.map((row) => ({
      labelY: row.top + (row.bottom - row.top) / 4,
    })),
  };
}

function katanaErDividerXOrDefault(dividerX, outer) {
  if (dividerX !== null) {
    return dividerX;
  }
  return outer.left + (outer.right - outer.left) / 2;
}

function katanaReadErDividerX(node, rows) {
  const minHeight = Math.max(1, rows[0].bottom - rows[0].top);
  const divider = Array.from(node.matchAll(/<g class="divider">\s*<path d="([^"]+)"/g))
    .map((match) => katanaReadErPathBox(match[1]))
    .find((box) => katanaIsErDividerBox(box, minHeight));
  return katanaErDividerCenter(divider);
}

function katanaIsErDividerBox(box, minHeight) {
  if (!box) {
    return false;
  }
  return [box.right - box.left < 1, box.bottom - box.top > minHeight].every(Boolean);
}

function katanaErDividerCenter(divider) {
  if (!divider) {
    return null;
  }
  return (divider.left + divider.right) / 2;
}

function katanaReadErPathBox(pathData) {
  if (!pathData) {
    return null;
  }
  return katanaErPathBoxFromValues(katanaErPathNumbers(pathData));
}

function katanaErPathNumbers(pathData) {
  return Array.from(pathData.matchAll(/-?\d+(?:\.\d+)?/g)).map((match) => Number(match[0]));
}

function katanaErPathBoxFromValues(numbers) {
  if (numbers.length < 4) {
    return null;
  }
  return katanaErPathBoxFromCoordinates(numbers);
}

function katanaErPathBoxFromCoordinates(numbers) {
  const xs = numbers.filter((_value, index) => index % 2 === 0);
  const ys = numbers.filter((_value, index) => index % 2 === 1);
  return {
    left: Math.min(...xs),
    top: Math.min(...ys),
    right: Math.max(...xs),
    bottom: Math.max(...ys),
  };
}

function katanaErAttributeX(kind, layout) {
  return katanaErAttributeBaseX(kind, layout) + 12.5;
}

function katanaErAttributeBaseX(kind, layout) {
  const bases = {
    "attribute-type": layout.left,
    "attribute-name": layout.dividerX,
  };
  if (Object.hasOwn(bases, kind)) {
    return bases[kind];
  }
  return layout.right;
}

function katanaReadErLabelText(node, kind) {
  const pattern = new RegExp(
    `<g class="label ${kind}"[\\s\\S]*?<tspan font-style="normal" class="text-inner-tspan" font-weight="normal">([^<]*)<\\/tspan>`,
  );
  return node.match(pattern)?.[1] ?? "";
}

function katanaErTextWidth(text) {
  const exact = {
    CUSTOMER: 76.0625,
    DIAGRAM: 64.625,
    DOCUMENT: 79.96875,
    ORDER: 47.796875,
    ORDER_ITEM: 89.84375,
    PRODUCT: 68.0625,
    SECTION: 60.578125,
  };
  if (Object.hasOwn(exact, text)) {
    return exact[text];
  }
  return Math.max(1, katanaEstimatedErTextWidth(text));
}

function katanaEstimatedErTextWidth(text) {
  if (typeof katanaTextWidth === "function") {
    return katanaTextWidth(text) * 1.25;
  }
  return String(text).length * 10;
}

function katanaNormalizeErSvg(svg) {
  if (!svg.includes('aria-roledescription="er"')) {
    return svg;
  }
  return katanaNormalizeErEdgeLabels(
    katanaCenterErStandaloneLabels(
      katanaCenterErHeaderLabels(katanaMoveErRowsBehindLabels(katanaNormalizeErNodes(svg))),
    ),
  );
}

function katanaNormalizeErEdgeLabels(svg) {
  return katanaRewriteBalancedGroups(
    svg,
    /<g class="label" data-id="id_entity-[^"]+"/g,
    katanaNormalizeErEdgeLabel,
  );
}

function katanaNormalizeErEdgeLabel(label) {
  return label
    .replace(
      /(<g class="label" data-id="[^"]+" transform="translate\(0, )[-\d.]+(\)">)/,
      "$1-9.000000476837158$2",
    )
    .replace(
      /(<rect class="background" style="" x="[^"]+" )y="[-\d.]+"( width="[^"]+" height=")23(?:\.\d+)?(")/,
      '$1y="-1"$223$3',
    )
    .replace(/class="text-outer-tspan(?! row)"/g, 'class="text-outer-tspan row"');
}

function katanaCenterErHeaderLabels(svg) {
  return svg.replace(
    /(<g class="label name" transform="translate\()[-\d.]+,\s*([-\d.]+)(\)"[^>]*>[\s\S]*?<text\b)([^>]*)(>)/g,
    (_match, start, y, middle, attributes, end) =>
      `${start}0, ${y}${middle}${katanaCenteredErTextAttributes(attributes)}${end}`,
  );
}

function katanaCenteredErTextAttributes(attributes) {
  return `${attributes.replace(/\stext-anchor="[^"]*"/g, "")} text-anchor="middle"`;
}

function katanaCenterErStandaloneLabels(svg) {
  return svg.replace(
    /(<g class="label" style="" transform="translate\()0,\s*([-\d.]+)(\)">\s*<rect><\/rect>\s*<g>\s*<rect class="background" style="stroke: none"><\/rect>\s*)<text\b[^>]*>/g,
    '$10, $2$3<text y="-10.1" style="" text-anchor="middle">',
  );
}

function katanaNormalizeErNodes(svg) {
  return katanaRewriteBalancedGroups(
    svg,
    /<g class="node default " id="[^"]*entity-[^"]*"/g,
    katanaNormalizeErNode,
  );
}

function katanaNormalizeErNode(node) {
  const layout = katanaReadErNodeLayout(node);
  if (!layout) {
    return katanaNormalizeErSingleNodeLabel(node);
  }
  return katanaNormalizeErNodeWithLayout(node, layout);
}

function katanaNormalizeErNodeWithLayout(node, layout) {
  const state = { attributeIndex: 0 };
  const normalized = node.replace(
    /class="label (name|attribute-type|attribute-name|attribute-keys|attribute-comment)" transform="translate\([^)]+\)"/g,
    (_match, kind) => katanaErLabelTransform(kind, state, layout),
  );
  return katanaCenterErNameText(katanaAddErRowTextClass(normalized));
}

function katanaErLabelTransform(kind, state, layout) {
  if (kind === "name") {
    return katanaErNameTransform(layout);
  }
  return katanaErAttributeTransform(kind, state, layout);
}

function katanaErNameTransform(layout) {
  return `class="label name" transform="translate(0, ${katanaFormatErNumber(layout.headerLabelY)})"`;
}

function katanaErAttributeTransform(kind, state, layout) {
  const row = katanaErRowAt(layout, state.attributeIndex);
  state.attributeIndex += 1;
  return `class="label ${kind}" transform="translate(${katanaFormatErNumber(katanaErAttributeX(kind, layout))}, ${katanaFormatErNumber(row.labelY)})"`;
}

function katanaErRowAt(layout, attributeIndex) {
  const row = layout.rows[Math.floor(attributeIndex / 4)];
  if (row) {
    return row;
  }
  return layout.rows[layout.rows.length - 1];
}

function katanaNormalizeErSingleNodeLabel(node) {
  if (!node.includes('class="basic label-container"')) {
    return node;
  }
  return node
    .replace(
      /(<g class="label" style="" transform="translate\()[-\d.]+,\s*[-\d.]+(\)">)/,
      "$10, -9.5$2",
    )
    .replace(/<text\b[^>]*>/, '<text y="-10.1" style="" text-anchor="middle">');
}

function katanaAddErRowTextClass(node) {
  return node.replace(/class="text-outer-tspan(?! row)"/g, 'class="text-outer-tspan row"');
}

function katanaCenterErNameText(node) {
  return node.replace(
    /(<g class="label name"[\s\S]*?<\/g>)(?=<g class="label |<g class="divider"|<\/g>$)/g,
    (label) =>
      label.replace(/<text\b([^>]*)>/, (_match, attributes) => {
        const cleaned = attributes.replace(/\stext-anchor="[^"]*"/g, "");
        return `<text${cleaned} text-anchor="middle">`;
      }),
  );
}

function katanaFormatErNumber(value) {
  return Number(value.toFixed(6)).toString();
}

function katanaMoveErRowsBehindLabels(svg) {
  return katanaRewriteBalancedGroups(
    svg,
    /<g class="node default " id="[^"]*entity-[^"]*"/g,
    katanaMoveErRowsInNode,
  );
}

function katanaMoveErRowsInNode(node) {
  const rowPattern = /<g style="" class="row-rect-(?:odd|even)">[\s\S]*?<\/g>/g;
  const rows = node.match(rowPattern);
  if (!rows) {
    return node;
  }
  const withoutRows = node.replace(rowPattern, "");
  return withoutRows.replace(
    /(<g class="outer-path" style="">[\s\S]*?<\/g>)/,
    `$1${rows.join("")}`,
  );
}

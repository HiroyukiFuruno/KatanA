function katanaNormalizeNativeSvgFallbacks(svg) {
  return svg.replace(/<foreignObject\b[\s\S]*?<\/foreignObject>/g, "");
}

function katanaNormalizeFlowchartEdgeLabelSvg(svg, request) {
  return katanaApplySvgWhen(svg, katanaHasFlowchartRole, (current) =>
    katanaEdgeLabelSvgForRequest(current, request),
  );
}

function katanaNormalizeStateEdgeLabelSvg(svg, request) {
  return katanaApplySvgWhen(svg, katanaHasStateDiagramRole, (current) =>
    katanaNormalizeStateViewBox(
      katanaNormalizeStateNodeLabels(katanaEdgeLabelSvgForRequest(current, request)),
    ),
  );
}

function katanaNormalizeStateViewBox(svg) {
  return katanaStateViewBoxContext(svg).map(katanaApplyStateViewBox).concat([svg])[0];
}

function katanaHasFlowchartRole(svg) {
  return svg.includes('aria-roledescription="flowchart-v2"');
}

function katanaHasStateDiagramRole(svg) {
  return svg.includes('aria-roledescription="stateDiagram"');
}

function katanaApplySvgWhen(svg, predicate, mapper) {
  return [svg].filter(predicate).map(mapper).concat([svg])[0];
}

function katanaEdgeLabelSvgForRequest(svg, request) {
  return KATANA_EDGE_LABEL_BACKGROUND_READERS[Number(katanaShouldHideEdgeLabelBackground(request))](
    svg,
  );
}

function katanaStateViewBoxContext(svg) {
  return [{ svg, viewBox: katanaReadViewBox(svg) }].filter(katanaHasStateViewBoxContext);
}

function katanaHasStateViewBoxContext(context) {
  return [
    context.viewBox,
    context.viewBox?.[1] < 0,
    Math.abs(context.viewBox?.[1] ?? 0) <= 12,
  ].every(Boolean);
}

function katanaApplyStateViewBox(context) {
  const viewBox = context.viewBox;
  return katanaSetSvgViewBox(
    katanaSetSvgMaxWidth(context.svg, viewBox[2]),
    `${katanaFormatVisualNumber(viewBox[0])} 0 ${katanaFormatVisualNumber(viewBox[2])} ${katanaFormatVisualNumber(viewBox[3] + viewBox[1])}`,
  );
}

function katanaFormatVisualNumber(value) {
  return Number(value.toFixed(6)).toString();
}

function katanaShouldHideEdgeLabelBackground(request) {
  return request.theme !== "dark";
}

function katanaNormalizeStateNodeLabels(svg) {
  return katanaRewriteBalancedGroups(
    svg,
    /<g class="node\s+statediagram-state\s+"/g,
    katanaNormalizeStateNodeLabel,
  );
}

function katanaNormalizeStateNodeLabel(node) {
  return node
    .replace(
      /(<g class="label" style="" transform="translate\()[-\d.]+,\s*([-\d.]+)(\)">)/g,
      "$10, $2$3",
    )
    .replace(/<text y="-10\.1" style="">/g, '<text y="-10.1" style="" text-anchor="middle">');
}

function katanaNormalizeRequirementEdgeLabelSvg(svg) {
  if (!svg.includes('aria-roledescription="requirement"')) {
    return svg;
  }
  return svg
    .replace(/(<g class="label" data-id="[^"]+" transform="translate\(0, )5\.76(\)")/g, "$1-10.5$2")
    .replace(
      /<rect class="background" style="" x="-54" y="-17\.36" width="108" height="23\.2"><\/rect>/g,
      '<rect class="background" style="" x="-48" y="-1" width="96" height="23"></rect>',
    );
}

function katanaHideBackgroundRects(svg) {
  return svg.replace(/<rect\b([^>]*\bclass="background"[^>]*)>/g, (_match, attributes) => {
    const cleaned = ["style", "fill", "opacity", "stroke"].reduce(
      (current, name) => katanaRemoveSvgAttr(current, name),
      attributes,
    );
    return `<rect${cleaned} style="fill: transparent !important; opacity: 0 !important; stroke: none !important;">`;
  });
}

function katanaNormalizeJourneySvg(svg, request) {
  if (!svg.includes('aria-roledescription="journey"')) {
    return svg;
  }
  return svg
    .replace(/<text\b([^>]*?)\sfill=""([^>]*)>/g, `<text$1 fill="${request.text}"$2>`)
    .replace(/(<line\b[^>]*\bstroke=")black(")/g, `$1${request.text}$2`);
}

function katanaNormalizePieSvg(svg) {
  if (!svg.includes('aria-roledescription="pie"')) {
    return svg;
  }
  const fills = Array.from(svg.matchAll(/<path\b[^>]*\bfill="([^"]+)"[^>]*class="pieCircle"/g)).map(
    (match) => match[1],
  );
  let legendIndex = 0;
  const withLegend = svg.replace(
    /(<g class="legend"[\s\S]*?<rect\b)([^>]*)(>)/g,
    (_match, start, attributes, end) => {
      const fill = fills[legendIndex] ?? "lightgrey";
      legendIndex += 1;
      return `${start}${katanaRemoveSvgAttr(attributes, "style")} style="fill: ${fill}; stroke: ${fill};"${end}`;
    },
  );
  return katanaNormalizePieViewBox(withLegend);
}

function katanaNormalizePieViewBox(svg) {
  return katanaPieViewBoxContext(svg).map(katanaApplyPieViewBox).concat([svg])[0];
}

function katanaPieViewBoxContext(svg) {
  return [{ svg, viewBox: katanaReadViewBox(svg), contentBox: katanaContentBox(svg) }]
    .filter(katanaHasPieViewBoxContext)
    .filter(katanaNeedsPieViewBoxWidth);
}

function katanaHasPieViewBoxContext(context) {
  return [context.viewBox, context.contentBox].every(Boolean);
}

function katanaNeedsPieViewBoxWidth(context) {
  return katanaPieViewBoxWidth(context.viewBox, context.contentBox) > context.viewBox[2];
}

function katanaApplyPieViewBox(context) {
  const width = katanaPieViewBoxWidth(context.viewBox, context.contentBox);
  return katanaSetNormalizedSvgSize(context.svg, [
    context.viewBox[0],
    context.viewBox[1],
    width,
    context.viewBox[3],
  ]);
}

const KATANA_EDGE_LABEL_BACKGROUND_READERS = [
  (svg) => svg,
  (svg) => katanaHideBackgroundRects(svg),
];

function katanaPieViewBoxWidth(viewBox, contentBox) {
  const right = contentBox[0] + contentBox[2];
  return Math.max(viewBox[2], right + 40);
}

function katanaNormalizeVennSvg(svg) {
  if (!svg.includes('aria-roledescription="venn"')) {
    return svg;
  }
  const withSets = [
    ["venn-set-0", "rgb(122, 122, 122)", "rgb(122, 122, 122)", "0.1"],
    ["venn-set-1", "rgb(164, 0, 0)", "rgb(164, 0, 0)", "0.1"],
    ["venn-set-2", "rgb(204, 42, 145)", "rgb(204, 42, 145)", "0.1"],
  ].reduce((current, rule) => katanaNormalizeVennCircle(current, ...rule), svg);
  return [
    ["A_B", "skyblue", null],
    ["B_C", "orange", null],
    ["A_C", "lightgreen", null],
    ["A_B_C", "white", "red"],
  ].reduce((current, rule) => katanaNormalizeVennArea(current, ...rule), withSets);
}

function katanaNormalizeVennCircle(svg, className, fill, stroke, opacity) {
  const area = new RegExp(
    `(<g class="venn-area venn-circle ${className}"[\\s\\S]*?<path\\b)([^>]*)(>)`,
  );
  return svg.replace(area, (_match, start, attributes, end) => {
    const cleaned = ["style", "fill", "stroke", "fill-opacity", "stroke-opacity"].reduce(
      (current, name) => katanaRemoveSvgAttr(current, name),
      attributes,
    );
    return `${start}${cleaned} style="fill: ${fill}; stroke: ${stroke}; fill-opacity: ${opacity}; stroke-opacity: 0.95; stroke-width: 2.5;"${end}`;
  });
}

function katanaNormalizeVennArea(svg, sets, fill, textFill) {
  const area = new RegExp(
    `(<g class="venn-area venn-intersection" data-venn-sets="${sets}"[\\s\\S]*?<path\\b)([^>]*)(>)`,
  );
  const colored = svg.replace(area, (_match, start, attributes, end) => {
    const cleaned = katanaRemoveSvgAttr(attributes, "style");
    return `${start}${cleaned} style="fill: ${fill}; fill-opacity: 1;"${end}`;
  });
  return textFill ? katanaNormalizeVennText(colored, sets, textFill) : colored;
}

function katanaNormalizeVennText(svg, sets, fill) {
  const area = new RegExp(
    `(<g class="venn-area venn-intersection" data-venn-sets="${sets}"[\\s\\S]*?<text\\b)([^>]*)(>)`,
  );
  return svg.replace(area, (_match, start, attributes, end) => {
    const cleaned = katanaRemoveSvgAttr(attributes, "style");
    return `${start}${cleaned} style="fill: ${fill}; font-size: 24px;"${end}`;
  });
}

function katanaRemoveSvgAttr(attributes, name) {
  return attributes.replace(new RegExp(`\\s${name}="[^"]*"`, "g"), "");
}

function katanaNormalizeEmptyTextFill(svg, request) {
  return svg.replace(/<text\b([^>]*?)\sfill=""([^>]*)>/g, `<text$1 fill="${request.text}"$2>`);
}

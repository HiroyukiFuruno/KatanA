function katanaNormalizeReviewFeedbackSvg(svg, request) {
  let normalized = svg;
  normalized = katanaNormalizeErReviewSvg(normalized);
  normalized = katanaNormalizeJourneyReviewSvg(normalized, request);
  normalized = katanaNormalizeGanttReviewSvg(normalized, request);
  normalized = katanaNormalizePieReviewSvg(normalized);
  normalized = katanaNormalizeQuadrantReviewSvg(normalized, request);
  normalized = katanaNormalizeTimelineReviewSvg(normalized);
  normalized = katanaNormalizeIshikawaReviewSvg(normalized);
  normalized = katanaNormalizeVennReviewSvg(normalized, request);
  normalized = katanaNormalizeTreemapReviewSvg(normalized);
  return normalized;
}

function katanaNormalizeErReviewSvg(svg) {
  if (!svg.includes('aria-roledescription="er"')) {
    return svg;
  }
  return svg
    .replace(
      /(data-id="id_entity-[^"]+" transform="translate\(0, )5\.76(\)")/g,
      "$1-9.000000476837158$2",
    )
    .replace(
      /<rect class="background" style="" x="([^"]+)" y="-17\.36" width="([^"]+)" height="23\.2"><\/rect>/g,
      '<rect class="background" style="" x="$1" y="-1" width="$2" height="23"></rect>',
    )
    .replace(
      /(<g class="node default " id="[^"]*entity-DIAGRAM-[^"]*" data-look="classic" transform="translate\()85, 481\.70000000000005(\)">)/g,
      "$178.265625, 480$2",
    )
    .replace(
      /<rect class="basic label-container" style="" x="-50" y="-39\.6" width="100" height="79\.2">/g,
      '<rect class="basic label-container" style="" x="-52.3125" y="-39.5" width="104.625" height="79">',
    )
    .replace(
      /(<g class="node default " id="[^"]*entity-DIAGRAM-[^"]*"[\s\S]*?<rect class="background" style="stroke: none"><\/rect>)<text[\s\S]*?<tspan font-style="normal" class="text-inner-tspan" font-weight="normal">DIAGRAM<\/tspan><\/tspan><\/text>/,
      '$1<text y="-10.1" style="" text-anchor="middle"><tspan class="text-outer-tspan row" x="0" y="-0.1em" dy="1.1em"><tspan font-style="normal" class="text-inner-tspan" font-weight="normal">DIAGRAM</tspan></tspan></text>',
    );
}

function katanaNormalizeJourneyReviewSvg(svg, request) {
  if (!svg.includes('aria-roledescription="journey"')) {
    return svg;
  }
  return svg
    .replace(/font-size: 14; font-family/g, "font-size: 14px; font-family")
    .replace("</style>", `text.journey-section{fill:${request.text}!important;}</style>`);
}

function katanaNormalizeGanttReviewSvg(svg, request) {
  if (!svg.includes('aria-roledescription="gantt"')) {
    return svg;
  }
  return katanaNormalizeGanttReviewTheme(katanaNormalizeGanttReviewSections(svg), request);
}

function katanaNormalizeGanttReviewSections(svg) {
  return svg
    .replace(/<rect\b([^>]*\bclass="section section0"[^>]*)>/g, (_match, attributes) =>
      katanaReviewTag(
        "rect",
        attributes,
        'fill="hsl(52.9411764706, 28.813559322%, 58.431372549%)" opacity="0.2"',
      ),
    )
    .replace(/<rect\b([^>]*\bclass="section section1"[^>]*)>/g, (_match, attributes) =>
      katanaReviewTag("rect", attributes, 'fill="transparent" opacity="0.2"'),
    )
    .replace("</style>", `.grid .tick{stroke:lightgrey!important;opacity:0.8!important;}</style>`);
}

function katanaNormalizeGanttReviewTheme(svg, request) {
  if (request.theme !== "dark") {
    return svg;
  }
  return katanaInsertSvgBackground(svg, "#1e1e1e");
}

function katanaNormalizePieReviewSvg(svg) {
  if (!svg.includes('aria-roledescription="pie"')) {
    return svg;
  }
  return svg
    .replace(/width="512"/, 'width="641"')
    .replace(/viewBox="0 0 512 450"/, 'viewBox="0 0 640.5 450"')
    .replace(/max-width:\s*512px/, "max-width: 640.5px");
}

function katanaNormalizeQuadrantReviewSvg(svg, request) {
  if (!svg.includes('aria-roledescription="quadrantChart"')) {
    return svg;
  }
  return svg
    .replace(
      /fill="hsl\(0, 0%, NaN%\)" stroke="hsl\(0, 0%, NaN%\)"/g,
      `fill="${request.text}" stroke="${request.text}"`,
    )
    .replace(/style="stroke-width: 2"/g, 'style="stroke: rgb(136, 136, 136); stroke-width: 2;"');
}

function katanaNormalizeTimelineReviewSvg(svg) {
  if (!svg.includes('aria-roledescription="timeline"')) {
    return svg;
  }
  return svg.replace(
    /<tspan x="0" dy="1em">Performance<\/tspan><tspan x="0" dy="1\.1em">check<\/tspan>/g,
    '<tspan x="0" dy="1em">Performance   check</tspan>',
  );
}

function katanaReviewTag(tagName, attributes, forcedAttributes) {
  const cleaned = ["fill", "opacity", "stroke"].reduce(
    (current, name) => katanaRemoveSvgAttr(current, name),
    attributes,
  );
  return `<${tagName}${cleaned} ${forcedAttributes}>`;
}

function katanaReviewPathAttrs(attributes, color, opacity) {
  const cleaned = ["style", "fill", "stroke", "fill-opacity", "stroke-opacity"].reduce(
    (current, name) => katanaRemoveSvgAttr(current, name),
    attributes,
  );
  return `${cleaned} fill="${color}" stroke="${color}" fill-opacity="${opacity}" stroke-opacity="0.95" style="stroke-width: 2.5;"`;
}

function katanaSetSvgAttr(attributes, name, value) {
  return `${katanaRemoveSvgAttr(attributes, name)} ${name}="${value}"`;
}

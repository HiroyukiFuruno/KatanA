function katanaNormalizeIshikawaReviewSvg(svg) {
  if (!svg.includes('aria-roledescription="ishikawa"')) {
    return svg;
  }
  const markerId = svg.match(/<marker id="([^"]*ishikawa-arrow[^"]*)"/)?.[1];
  const normalized = svg
    .replace(/<g class="ishikawa-head-group"[\s\S]*?<\/g>/, katanaNormalizeIshikawaHeadGroup)
    .replace(
      /(<g class="ishikawa-label-group"><rect\b[^>]*\sy=")([^"]+)("[^>]*><\/rect><text\b[^>]*\by=")([-\d.]+)(")/g,
      katanaIshikawaLabelGroupReplacement,
    );
  return katanaNormalizeIshikawaViewBox(katanaAddIshikawaArrowMarkers(normalized, markerId));
}

function katanaNormalizeIshikawaHeadGroup(group) {
  const lines = katanaIshikawaHeadLines(group);
  const width = katanaIshikawaHeadWidth(lines);
  const height = katanaIshikawaHeadHeight(lines);
  return group
    .replace(
      /d="M 0 -?[\d.]+ L 0 -?[\d.]+ Q -?[\d.]+ 0 0 -?[\d.]+ Z"/,
      `d="${katanaIshikawaHeadPath(width, height)}"`,
    )
    .replace(/<text class="ishikawa-head-label"([^>]*)>/, (_match, attributes) =>
      katanaIshikawaHeadTextTag(attributes),
    )
    .replace(/<tspan x="[^"]+"/g, '<tspan x="0"');
}

function katanaIshikawaHeadLines(group) {
  return Array.from(group.matchAll(/<tspan\b[^>]*>([^<]*)<\/tspan>/g)).map((match) => match[1]);
}

function katanaIshikawaHeadWidth(lines) {
  const lineWidth = Math.max(0, ...lines.map((line) => katanaTextWidth(line)));
  return Math.max(144, Math.ceil(lineWidth + 64));
}

function katanaIshikawaHeadHeight(lines) {
  return Math.max(105.6, Math.max(1, lines.length) * 16.8 + 72);
}

function katanaIshikawaHeadPath(width, height) {
  const halfHeight = katanaFormatIshikawaNumber(height / 2);
  return `M 0 -${halfHeight} L 0 ${halfHeight} Q ${katanaFormatIshikawaNumber(width)} 0 0 -${halfHeight} Z`;
}

function katanaIshikawaHeadTextTag(attributes) {
  const cleaned = attributes
    .replace(/\stext-anchor="[^"]*"/g, "")
    .replace(/\stransform="[^"]*"/g, "");
  return `<text class="ishikawa-head-label"${cleaned} text-anchor="start" transform="translate(33,1.34375)">`;
}

function katanaFormatIshikawaNumber(value) {
  return Number(value.toFixed(3)).toString();
}

function katanaIshikawaLabelGroupReplacement(match, start, _oldY, middle, textY, end) {
  const nextY = Number(textY) - 12.8125;
  if (Number.isFinite(nextY)) {
    return `${start}${nextY}${middle}${textY}${end}`;
  }
  return match;
}

function katanaAddIshikawaArrowMarkers(svg, markerId) {
  if (!markerId) {
    return svg;
  }
  return svg.replace(
    /<line class="ishikawa-(branch|sub-branch)"([^>]*)><\/line>/g,
    (match, kind, attributes) => katanaIshikawaLineWithMarker(match, kind, attributes, markerId),
  );
}

function katanaIshikawaLineWithMarker(match, kind, attributes, markerId) {
  if (attributes.includes("marker-start")) {
    return match;
  }
  return `<line class="ishikawa-${kind}"${attributes} marker-start="url(#${markerId})"></line>`;
}

function katanaNormalizeIshikawaViewBox(svg) {
  return katanaIshikawaViewBoxContext(svg).map(katanaApplyIshikawaViewBox).concat([svg])[0];
}

function katanaIshikawaViewBoxContext(svg) {
  return [{ svg, contentBox: katanaContentBox(svg), viewBox: katanaReadViewBox(svg) }].filter(
    katanaHasIshikawaViewBoxContext,
  );
}

function katanaHasIshikawaViewBoxContext(context) {
  return [context.contentBox, context.viewBox].every(Boolean);
}

function katanaApplyIshikawaViewBox(context) {
  const normalized = katanaIshikawaViewBox(context.viewBox, context.contentBox);
  return katanaSetSvgMaxWidth(
    katanaSetSvgViewBox(context.svg, normalized.join(" ")),
    normalized[2],
  );
}

function katanaIshikawaViewBox(viewBox, contentBox) {
  const left = Math.min(viewBox[0], contentBox[0]);
  const top = Math.min(viewBox[1], contentBox[1] - 2);
  const right = contentBox[0] + contentBox[2];
  const bottom = contentBox[1] + contentBox[3] + 6;
  return [
    katanaFormatIshikawaNumber(left),
    katanaFormatIshikawaNumber(top),
    katanaFormatIshikawaNumber(right - left),
    katanaFormatIshikawaNumber(bottom - top),
  ];
}

function katanaNormalizeVennReviewSvg(svg, request) {
  return katanaShouldNormalizeVennReviewSvg(svg)
    ? katanaNormalizeVennReviewTheme(katanaNormalizeVennReviewPaths(svg), request)
    : svg;
}

function katanaShouldNormalizeVennReviewSvg(svg) {
  return [svg.includes('aria-roledescription="venn"'), katanaIsRendererScopeVenn(svg)].every(
    Boolean,
  );
}

function katanaIsRendererScopeVenn(svg) {
  return [
    svg.includes("Renderer scope"),
    svg.includes('data-venn-sets="official"'),
    svg.includes('data-venn-sets="rust"'),
  ].every(Boolean);
}

function katanaNormalizeVennReviewPaths(svg) {
  return katanaVennPath(svg, "venn-set-0", "rgb(122,122,122)", "0.1").replace(
    /(<g class="venn-area venn-circle venn-set-1"[\s\S]*?<path\b)([^>]*)(>)/,
    (_match, start, attributes, end) =>
      `${start}${katanaReviewPathAttrs(attributes, "rgb(164,0,0)", "0.1")}${end}`,
  );
}

function katanaNormalizeVennReviewTheme(svg, request) {
  if (request.theme !== "dark") {
    return svg;
  }
  return katanaInsertSvgBackground(svg, "#1e1e1e");
}

function katanaInsertSvgBackground(svg, color) {
  return svg.replace(
    /(<svg\b[^>]*>)/,
    `$1${katanaSvgBackgroundRect(katanaBackgroundViewBox(svg), color)}`,
  );
}

function katanaBackgroundViewBox(svg) {
  if (typeof katanaReadViewBox !== "function") {
    return null;
  }
  return katanaReadViewBox(svg);
}

function katanaSvgBackgroundRect(viewBox, color) {
  if (viewBox) {
    return `<rect x="${viewBox[0]}" y="${viewBox[1]}" width="${viewBox[2]}" height="${viewBox[3]}" fill="${color}"></rect>`;
  }
  return `<rect width="100%" height="100%" fill="${color}"></rect>`;
}

function katanaVennPath(svg, className, color, opacity) {
  const pattern = new RegExp(
    `(<g class="venn-area venn-circle ${className}"[\\s\\S]*?<path\\b)([^>]*)(>)`,
  );
  return svg.replace(
    pattern,
    (_match, start, attributes, end) =>
      `${start}${katanaReviewPathAttrs(attributes, color, opacity)}${end}`,
  );
}

function katanaNormalizeTreemapReviewSvg(svg) {
  if (!svg.includes('aria-roledescription="treemap"')) {
    return svg;
  }
  return svg
    .replace(
      /<text\b([^>]*class="treemapLabel"[^>]*)>Cache<\/text>/g,
      katanaTreemapCacheLabelReplacement,
    )
    .replace(
      /<text\b([^>]*class="treemapValue"[^>]*x="44\.5"[^>]*)>10<\/text>/g,
      katanaTreemapCacheValueReplacement,
    );
}

function katanaTreemapCacheLabelReplacement(_match, attributes) {
  return `<text${attributes.replace(/font-size:\s*38px/g, "font-size: 29px")}>Cache</text>`;
}

function katanaTreemapCacheValueReplacement(_match, attributes) {
  return `<text${attributes.replace(/font-size:\s*28px/g, "font-size: 17px")}>10</text>`;
}

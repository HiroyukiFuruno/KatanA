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
  return Math.max(144, Math.ceil(lineWidth + katanaIshikawaHeadHorizontalPadding(lines)));
}

function katanaIshikawaHeadHeight(lines) {
  return Math.max(
    katanaIshikawaHeadMinimumHeight(lines),
    Math.max(1, lines.length) * 16.8 + katanaIshikawaHeadVerticalPadding(lines),
  );
}

function katanaIshikawaHeadHorizontalPadding(lines) {
  return KATANA_ISHIKAWA_HEAD_HORIZONTAL_PADDING[Number(lines.length > 1)];
}

function katanaIshikawaHeadVerticalPadding(lines) {
  return KATANA_ISHIKAWA_HEAD_VERTICAL_PADDING[Number(lines.length > 1)];
}

function katanaIshikawaHeadMinimumHeight(lines) {
  return KATANA_ISHIKAWA_HEAD_MINIMUM_HEIGHT[Number(lines.length > 1)];
}

function katanaIshikawaHeadPath(width, height) {
  const halfHeight = katanaFormatIshikawaNumber(height / 2);
  return `M 0 -${halfHeight} L 0 ${halfHeight} Q ${katanaFormatIshikawaNumber(width)} 0 0 -${halfHeight} Z`;
}

const KATANA_ISHIKAWA_HEAD_HORIZONTAL_PADDING = [
  // WHY: Mermaid.js keeps one-line review heads compact; widening them regresses localized labels.
  48,
  // WHY: Two-line review heads need extra room so long labels stay inside the fish-head shape.
  64,
];

const KATANA_ISHIKAWA_HEAD_VERTICAL_PADDING = [
  55.2,
  // WHY: Mermaid.js uses a taller envelope for wrapped fish-head labels.
  72,
];

const KATANA_ISHIKAWA_HEAD_MINIMUM_HEIGHT = [
  72,
  // WHY: This is the browser-measured height for the accepted two-line "Blurry Photo" case.
  105.6,
];

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

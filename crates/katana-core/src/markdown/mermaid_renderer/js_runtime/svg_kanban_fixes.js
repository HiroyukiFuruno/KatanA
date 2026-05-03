function katanaNormalizeKanbanLayoutSvg(svg, request) {
  const layout = katanaKanbanLayoutState(svg);
  const withNodes = katanaRewriteBalancedGroups(svg, /<g class="node undefined "[^>]*>/g, (group) =>
    katanaNormalizeKanbanNodeGroup(group, layout),
  );
  const withLabels = katanaNormalizeKanbanClusterLabels(withNodes);
  const withViewBox = katanaNormalizeKanbanViewBox(
    katanaNormalizeKanbanSectionGroups(withLabels, layout),
    layout,
  );
  return katanaNormalizeKanbanReadableTextSvg(withViewBox, request);
}

function katanaNormalizeKanbanReadableTextSvg(svg, request) {
  if (request.theme === "dark") {
    return svg;
  }
  return svg.replace("</style>", `${katanaKanbanReadableTextStyle(svg)}</style>`);
}

function katanaKanbanReadableTextStyle(svg) {
  const root = katanaKanbanRootSelector(svg);
  const selectors = [
    `${root} .cluster-label text`,
    `${root} .cluster-label tspan`,
    `${root} .label text`,
    `${root} .label tspan`,
    `${root} .kanban-ticket-link text`,
    `${root} .kanban-ticket-link tspan`,
  ];
  return `${selectors.join(",")}{fill:#333333!important;color:#333333!important;}`;
}

function katanaKanbanRootSelector(svg) {
  const id = svg.match(/^<svg\b[^>]*\sid="([^"]+)"/)?.[1];
  return id ? `#${id}` : "svg";
}

function katanaKanbanLayoutState(svg) {
  const sections = new Map();
  for (const match of svg.matchAll(
    /<g class="cluster undefined [^"]*"[^>]*>[\s\S]*?<rect[^>]*\sx="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/g,
  )) {
    const left = Number(match[1]);
    const top = Number(match[2]);
    const width = Number(match[3]);
    sections.set(katanaKanbanSectionKey(left + width / 2), {
      left,
      right: left + width,
      top,
      bottom: top,
    });
  }
  return { sections };
}

function katanaNormalizeKanbanNodeGroup(group, layout) {
  const input = katanaKanbanNodeLayoutInput(group);
  return input ? katanaNormalizeKanbanNodeGroupInput(group, layout, input) : group;
}

function katanaKanbanNodeLayoutInput(group) {
  const position = katanaKanbanNodePosition(group);
  const oldHeight = katanaKanbanNodeRectHeight(group);
  return katanaIsKanbanNodeLayoutInput(position, oldHeight) ? { position, oldHeight } : null;
}

function katanaIsKanbanNodeLayoutInput(position, oldHeight) {
  return [Boolean(position), oldHeight > 0].every(Boolean);
}

function katanaNormalizeKanbanNodeGroupInput(group, layout, input) {
  const normalizedGroup = katanaNormalizeKanbanLabelGroups(group);
  const metrics = katanaKanbanNodeMetrics(normalizedGroup);
  const nextHeight = metrics.height;
  const section = katanaKanbanNodeSection(layout, input.position.x);
  const top = katanaKanbanNextNodeTop(section);
  katanaUpdateKanbanSectionBounds(section, top, nextHeight);
  return katanaKanbanNodeWithLayout(normalizedGroup, input.position, top, metrics);
}

const KATANA_KANBAN_CARD_GAP = 5;
const KATANA_KANBAN_SECTION_BOTTOM_GAP = 10;

function katanaKanbanNextNodeTop(section) {
  return section.nextTop ?? section.top + 25;
}

function katanaUpdateKanbanSectionBounds(section, top, height) {
  section.nextTop = top + height + KATANA_KANBAN_CARD_GAP;
  section.bottom = top + height + KATANA_KANBAN_SECTION_BOTTOM_GAP;
}

function katanaKanbanNodePosition(group) {
  const match = group.match(/transform="translate\(([-\d.]+), ([-\d.]+)\)"/);
  if (!match) {
    return null;
  }
  return { x: Number(match[1]), y: Number(match[2]) };
}

function katanaKanbanNodeRectHeight(group) {
  const match = group.match(/<rect class="basic label-container[^"]*"[^>]*\sheight="([^"]+)"/);
  return match ? Number(match[1]) : 0;
}

function katanaKanbanNodeMetrics(group) {
  const mainLabel = katanaKanbanMainLabelGroup(group);
  const lines = mainLabel?.lines ?? 1;
  const hasMetadata = katanaKanbanHasVisibleMetadata(group, mainLabel);
  return { height: katanaKanbanNodeHeight(lines, hasMetadata), mainLabel, hasMetadata };
}

function katanaKanbanNodeHeight(lines, hasMetadata) {
  return KATANA_KANBAN_BASE_CARD_HEIGHTS[Number(hasMetadata)] + Math.max(0, lines - 1) * 17.6;
}

function katanaKanbanHasVisibleMetadata(group, mainLabel) {
  return katanaKanbanLabelGroups(group).some(
    (label) => label.tag !== mainLabel?.tag && label.text.length > 0,
  );
}

const KATANA_KANBAN_BASE_CARD_HEIGHTS = [39, 48.5];

function katanaKanbanNodeSection(layout, x) {
  const key = katanaKanbanSectionKey(x);
  const section = layout.sections.get(key) ?? {
    left: x - 100,
    right: x + 100,
    top: -300,
    bottom: -300,
  };
  layout.sections.set(key, section);
  return section;
}

function katanaKanbanNodeWithLayout(group, position, top, metrics) {
  const height = metrics.height;
  const y = top + height / 2;
  const moved = group
    .replace(
      /transform="translate\(([-\d.]+), ([-\d.]+)\)"/,
      `transform="translate(${position.x}, ${katanaFormatSvgNumber(y)})"`,
    )
    .replace(
      /(<rect class="basic label-container[^"]*"[^>]*\sy=")([^"]+)("[^>]*\sheight=")([^"]+)(")/,
      `$1${katanaFormatSvgNumber(-height / 2)}$3${katanaFormatSvgNumber(height)}$5`,
    )
    .replace(
      /(<line\b[^>]*\by1=")([^"]+)("[^>]*\by2=")([^"]+)(")/g,
      `$1${katanaFormatSvgNumber(-height / 2 + 2)}$3${katanaFormatSvgNumber(height / 2 - 2)}$5`,
    );
  return katanaKanbanMoveMainLabel(katanaKanbanMoveMetadataLabels(moved, metrics), metrics);
}

function katanaNormalizeKanbanSectionGroups(svg, layout) {
  return svg.replace(
    /(<rect style="" rx="5" ry="5" x="([^"]+)" y="([^"]+)" width="([^"]+)" height=")([^"]+)(")/g,
    (_match, before, x, y, width, height, after) =>
      katanaKanbanSectionHeightReplacement(layout, before, x, y, width, height, after),
  );
}

function katanaKanbanSectionHeightReplacement(layout, before, x, y, width, height, after) {
  return `${before}${katanaFormatSvgNumber(katanaKanbanSectionHeight(layout, x, y, width, height))}${after}`;
}

function katanaKanbanSectionHeight(layout, x, y, width, height) {
  const section = layout.sections.get(katanaKanbanSectionKey(Number(x) + Number(width) / 2));
  return KATANA_KANBAN_SECTION_HEIGHTS[Number(Boolean(section))](section, y, height);
}

function katanaMeasuredKanbanSectionHeight(section, y) {
  return section.bottom - Number(y);
}

function katanaOriginalKanbanSectionHeight(_section, _y, height) {
  return Number(height);
}

const KATANA_KANBAN_SECTION_HEIGHTS = [
  katanaOriginalKanbanSectionHeight,
  katanaMeasuredKanbanSectionHeight,
];

function katanaKanbanSectionKey(x) {
  return String(Math.round(x * 100) / 100);
}

function katanaNormalizeKanbanViewBox(svg, layout) {
  const sections = Array.from(layout.sections.values());
  return sections.length === 0 ? svg : katanaSetSvgViewBox(svg, katanaKanbanViewBoxValue(sections));
}

function katanaKanbanViewBoxValue(sections) {
  return katanaKanbanViewBox(sections).map(katanaFormatSvgNumber).join(" ");
}

function katanaKanbanViewBox(sections) {
  const left = Math.min(...sections.map((section) => section.left)) - 10;
  const top = Math.min(...sections.map((section) => section.top)) - 10;
  const right = Math.max(...sections.map((section) => section.right)) + 10;
  const bottom = Math.max(...sections.map((section) => section.bottom)) + 10;
  return [left, top, right - left, bottom - top];
}

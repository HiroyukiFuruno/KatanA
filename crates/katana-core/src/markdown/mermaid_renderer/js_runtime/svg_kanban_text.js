const KATANA_KANBAN_LABEL_WIDTH = 185;

function katanaKanbanWrappedLabelLines(labelGroup) {
  return Math.max(
    katanaKanbanOuterLineCount(labelGroup),
    katanaKanbanMeasuredLineCount(labelGroup),
  );
}

function katanaKanbanOuterLineCount(labelGroup) {
  return (labelGroup.match(/<tspan class="text-outer-tspan"/g) ?? []).length;
}

function katanaKanbanMeasuredLineCount(labelGroup) {
  const text = katanaKanbanLabelText(labelGroup);
  return text ? Math.ceil(katanaTextWidth(text) / KATANA_KANBAN_LABEL_WIDTH) : 0;
}

function katanaKanbanLabelText(labelGroup) {
  return Array.from(
    labelGroup.matchAll(/<tspan\b[^>]*class="text-inner-tspan"[^>]*>([^<]*)<\/tspan>/g),
  )
    .map((match) => match[1])
    .join("");
}

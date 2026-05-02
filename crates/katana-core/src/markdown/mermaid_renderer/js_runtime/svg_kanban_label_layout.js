function katanaKanbanMoveMainLabel(group, metrics) {
  const label = metrics.mainLabel?.tag ?? "";
  return KATANA_KANBAN_MAIN_LABEL_MOVERS[Number(label.length > 0)](group, label, metrics);
}

function katanaKanbanKeepMainLabelGroup(group) {
  return group;
}

function katanaKanbanReplaceMainLabelGroup(group, label, metrics) {
  return group.replace(label, katanaKanbanLabelTransform(label, katanaKanbanMainLabelY(metrics)));
}

const KATANA_KANBAN_MAIN_LABEL_MOVERS = [
  katanaKanbanKeepMainLabelGroup,
  katanaKanbanReplaceMainLabelGroup,
];

function katanaKanbanMainLabelY(metrics) {
  return -metrics.height / 2 + KATANA_KANBAN_MAIN_LABEL_OFFSETS[Number(metrics.hasMetadata)];
}

const KATANA_KANBAN_MAIN_LABEL_OFFSETS = [10, 5.25];

function katanaKanbanMainLabelGroup(group) {
  return katanaKanbanLabelGroups(group)
    .filter((label) => label.text.length > 0)
    .sort(katanaKanbanMainLabelSort)[0];
}

function katanaKanbanLabelGroups(group) {
  const labels = [];
  const pattern = /<g class="label"[^>]*transform="translate\([^"]+\)">/g;
  let match = pattern.exec(group);
  while (match !== null) {
    const end = katanaFindBalancedGroupEnd(group, match.index);
    labels.push(katanaKanbanLabelGroup(match[0], group.slice(match.index, end)));
    pattern.lastIndex = end;
    match = pattern.exec(group);
  }
  return labels;
}

function katanaKanbanLabelGroup(tag, body) {
  return {
    body,
    lines: katanaKanbanLabelLines(body),
    tag,
    text: katanaKanbanLabelText(body),
    y: katanaKanbanLabelY(tag),
  };
}

function katanaKanbanLabelY(tag) {
  const match = tag.match(/translate\([-\d.]+, ([-\d.]+)\)/);
  return Number(match?.[1] ?? 0);
}

function katanaKanbanMainLabelSort(left, right) {
  return left.y - right.y || right.lines - left.lines;
}

function katanaKanbanMoveMetadataLabels(group, metrics) {
  return group.replace(
    /<g class="label"[^>]*transform="translate\(([-\d.]+), ([-\d.]+)\)">/g,
    (match) => katanaKanbanMetadataLabelTransform(match, metrics),
  );
}

function katanaKanbanMetadataLabelTransform(tag, metrics) {
  return tag === metrics.mainLabel?.tag
    ? tag
    : katanaKanbanLabelTransform(tag, katanaKanbanMetadataLabelY(metrics));
}

function katanaKanbanMetadataLabelY(metrics) {
  return metrics.height / 2 - KATANA_KANBAN_METADATA_LABEL_OFFSETS[Number(metrics.hasMetadata)];
}

const KATANA_KANBAN_METADATA_LABEL_OFFSETS = [10, 24.25];

function katanaKanbanLabelLines(labelGroup) {
  return katanaKanbanWrappedLabelLines(labelGroup);
}

function katanaKanbanLabelTransform(tag, y) {
  return tag.replace(
    /translate\(([-\d.]+), ([-\d.]+)\)/,
    (_match, x) => `translate(${x}, ${katanaFormatSvgNumber(y)})`,
  );
}

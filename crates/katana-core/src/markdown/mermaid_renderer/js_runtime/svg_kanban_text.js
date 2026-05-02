const KATANA_KANBAN_LABEL_WIDTH = 185;
const KATANA_KANBAN_I18N_LABEL_WIDTH = 160;
const KATANA_KANBAN_I18N_WIDE_CHARACTER_WIDTH = 14;
const KATANA_KANBAN_I18N_LINE_APPENDERS = [
  katanaKanbanAppendI18nCurrentLine,
  katanaKanbanAppendI18nNextLine,
];
const KATANA_KANBAN_I18N_CHARACTER_WIDTHS = [
  katanaAsciiCharacterWidth,
  () => KATANA_KANBAN_I18N_WIDE_CHARACTER_WIDTH,
];

function katanaKanbanWrappedLabelLines(labelGroup) {
  return Math.max(
    katanaKanbanOuterLineCount(labelGroup),
    katanaKanbanMeasuredLineCount(labelGroup),
  );
}

function katanaNormalizeKanbanLabelGroups(group) {
  let result = "";
  let cursor = 0;
  const pattern = /<g class="label"[^>]*transform="translate\([^"]+\)">/g;
  let match = pattern.exec(group);
  while (match !== null) {
    const end = katanaFindBalancedGroupEnd(group, match.index);
    result += group.slice(cursor, match.index);
    result += katanaNormalizeKanbanLabelTextLines(group.slice(match.index, end));
    cursor = end;
    pattern.lastIndex = end;
    match = pattern.exec(group);
  }
  return result + group.slice(cursor);
}

function katanaNormalizeKanbanLabelTextLines(labelGroup) {
  const text = katanaKanbanLabelText(labelGroup);
  if (!katanaKanbanNeedsI18nWrap(text)) {
    return labelGroup;
  }
  const lines = katanaKanbanWrapI18nLabel(text);
  return labelGroup.replace(/(<text\b[^>]*>)[\s\S]*?(<\/text>)/, (_match, before, after) => {
    return `${before}${katanaKanbanLabelLineTspans(lines)}${after}`;
  });
}

function katanaKanbanNeedsI18nWrap(text) {
  return Array.from(text).some((char) => (char.codePointAt(0) ?? 0) > 0x7f);
}

function katanaKanbanWrapI18nLabel(text) {
  return Array.from(text).reduce(katanaKanbanAppendI18nCharacter, [""]).filter(Boolean);
}

function katanaKanbanAppendI18nCharacter(lines, char) {
  const line = lines.at(-1) ?? "";
  const next = `${line}${char}`;
  return KATANA_KANBAN_I18N_LINE_APPENDERS[Number(katanaKanbanShouldStartNewI18nLine(line, next))](
    lines,
    char,
    next,
  );
}

function katanaKanbanShouldStartNewI18nLine(line, next) {
  return [line.length > 0, katanaKanbanI18nTextWidth(next) > KATANA_KANBAN_I18N_LABEL_WIDTH].every(
    Boolean,
  );
}

function katanaKanbanAppendI18nCurrentLine(lines, _char, next) {
  return [...lines.slice(0, -1), next];
}

function katanaKanbanAppendI18nNextLine(lines, char) {
  return [...lines, char];
}

function katanaKanbanI18nTextWidth(text) {
  return Array.from(text)
    .map((char) => katanaKanbanI18nCharacterWidth(char))
    .reduce((width, charWidth) => width + charWidth, 0);
}

function katanaKanbanI18nCharacterWidth(char) {
  return KATANA_KANBAN_I18N_CHARACTER_WIDTHS[Number(katanaKanbanIsWideI18nCharacter(char))](char);
}

function katanaKanbanIsWideI18nCharacter(char) {
  return (char.codePointAt(0) ?? 0) > 0x7f;
}

function katanaKanbanLabelLineTspans(lines) {
  return lines.map((line, index) => katanaKanbanLabelLineTspan(line, index)).join("");
}

function katanaKanbanLabelLineTspan(line, index) {
  const y = katanaKanbanLabelLineY(index);
  return `<tspan class="text-outer-tspan row" x="0" y="${y}em" dy="1.1em"><tspan font-style="normal" class="text-inner-tspan" font-weight="normal">${katanaEscapeSvgText(line)}</tspan></tspan>`;
}

function katanaKanbanLabelLineY(index) {
  return katanaFormatSvgNumber(index * 1.1 - 0.1);
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

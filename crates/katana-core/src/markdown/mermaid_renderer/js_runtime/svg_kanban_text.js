const KATANA_KANBAN_LABEL_WIDTH = 185;
const KATANA_KANBAN_I18N_LABEL_WIDTH = 174;
const KATANA_KANBAN_I18N_CJK_CHARACTER_WIDTH = 16.3;
const KATANA_KANBAN_I18N_PUNCTUATION_WIDTH = 20;
const KATANA_KANBAN_I18N_WIDE_CHARACTER_WIDTH = 15.6;
const KATANA_KANBAN_I18N_LINE_APPENDERS = [
  katanaKanbanAppendI18nCurrentLine,
  katanaKanbanAppendI18nNextLine,
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
  const resolver =
    KATANA_KANBAN_I18N_CHARACTER_WIDTH_RESOLVERS[Number(katanaKanbanIsWideI18nCharacter(char))];
  return resolver(char);
}

function katanaKanbanWideI18nCharacterWidth(char) {
  const override = KATANA_KANBAN_I18N_CHARACTER_WIDTH_OVERRIDES.get(char);
  if (override) {
    return override;
  }
  return katanaKanbanDefaultWideI18nCharacterWidth(char);
}

function katanaKanbanDefaultWideI18nCharacterWidth(char) {
  if (katanaKanbanIsWidePunctuation(char)) {
    return KATANA_KANBAN_I18N_PUNCTUATION_WIDTH;
  }
  return KATANA_KANBAN_I18N_WIDE_CHARACTER_WIDTHS[Number(katanaKanbanIsCjkIdeograph(char))]();
}

function katanaKanbanIsWideI18nCharacter(char) {
  return (char.codePointAt(0) ?? 0) > 0x7f;
}

function katanaKanbanIsCjkIdeograph(char) {
  const codePoint = katanaKanbanCodePoint(char);
  return KATANA_KANBAN_CJK_IDEOGRAPH_RULES.every((rule) => rule(codePoint));
}

function katanaKanbanCodePoint(char) {
  return char.codePointAt(0) ?? 0;
}

function katanaKanbanIsCjkIdeographStart(codePoint) {
  return codePoint >= 0x4e00;
}

function katanaKanbanIsCjkIdeographEnd(codePoint) {
  return codePoint <= 0x9fff;
}

function katanaKanbanIsWidePunctuation(char) {
  return ["。", "、"].includes(char);
}

const KATANA_KANBAN_I18N_WIDE_CHARACTER_WIDTHS = [
  () => KATANA_KANBAN_I18N_WIDE_CHARACTER_WIDTH,
  () => KATANA_KANBAN_I18N_CJK_CHARACTER_WIDTH,
];
const KATANA_KANBAN_I18N_CHARACTER_WIDTH_OVERRIDES = new Map([["\u3082", 16.1]]);
const KATANA_KANBAN_I18N_CHARACTER_WIDTH_RESOLVERS = [
  katanaAsciiCharacterWidth,
  katanaKanbanWideI18nCharacterWidth,
];
const KATANA_KANBAN_CJK_IDEOGRAPH_RULES = [
  katanaKanbanIsCjkIdeographStart,
  katanaKanbanIsCjkIdeographEnd,
];

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
  return katanaKanbanTextMeasuredLineCount(text);
}

function katanaKanbanTextMeasuredLineCount(text) {
  if (!text) {
    return 0;
  }
  return katanaKanbanNonEmptyTextMeasuredLineCount(text);
}

function katanaKanbanNonEmptyTextMeasuredLineCount(text) {
  if (katanaKanbanNeedsI18nWrap(text)) {
    return katanaKanbanWrapI18nLabel(text).length;
  }
  return Math.ceil(katanaTextWidth(text) / KATANA_KANBAN_LABEL_WIDTH);
}

function katanaKanbanLabelText(labelGroup) {
  return Array.from(
    labelGroup.matchAll(/<tspan\b[^>]*class="text-inner-tspan"[^>]*>([^<]*)<\/tspan>/g),
  )
    .map((match) => match[1])
    .join("");
}

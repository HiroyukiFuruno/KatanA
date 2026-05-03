function katanaRestoreMermaidI18nText(svg, replacements) {
  return replacements.reduce(
    (current, replacement) => katanaRestoreMermaidI18nReplacement(current, replacement),
    svg,
  );
}

function katanaRestoreMermaidI18nReplacement(svg, replacement) {
  const pattern = new RegExp(`(>[^<]*)${replacement.placeholder}([^<]*<)`, "g");
  return svg.replace(
    pattern,
    (_match, before, after) => `${before}${katanaEscapeSvgText(replacement.text)}${after}`,
  );
}

function katanaI18nContext(source) {
  const frontmatter = katanaI18nFrontmatter(source);
  const replacements = [];
  const idMap = new Map();
  return {
    source,
    prefix: frontmatter.prefix,
    body: frontmatter.body,
    replacements,
    label(text) {
      return katanaI18nLabel(replacements, text);
    },
    id(text) {
      return katanaI18nId(idMap, replacements, text);
    },
    result(sourceText) {
      return {
        source: sourceText,
        replacements,
      };
    },
  };
}

function katanaI18nLabel(replacements, text) {
  return katanaI18nValueReaders()[Number(katanaNeedsI18nPlaceholder(text))](replacements, text);
}

function katanaI18nId(idMap, replacements, text) {
  return katanaI18nIdReaders()[Number(katanaNeedsI18nPlaceholder(text))](idMap, replacements, text);
}

function katanaI18nValueReaders() {
  return [katanaI18nOriginalText, katanaPushI18nReplacement];
}

function katanaI18nIdReaders() {
  return [katanaI18nOriginalId, katanaCachedI18nId];
}

function katanaI18nOriginalText(_replacements, text) {
  return text;
}

function katanaI18nOriginalId(_idMap, _replacements, text) {
  return text;
}

function katanaCachedI18nId(idMap, replacements, text) {
  return (
    idMap.get(text) ?? katanaStoreI18nId(idMap, text, katanaPushI18nReplacement(replacements, text))
  );
}

function katanaStoreI18nId(idMap, text, value) {
  idMap.set(text, value);
  return value;
}

function katanaI18nFrontmatter(source) {
  return katanaI18nFrontmatterReaders()[Number(source.trimStart().startsWith("---"))](source);
}

function katanaI18nFrontmatterReaders() {
  return [katanaI18nNoFrontmatter, katanaI18nParsedFrontmatter];
}

function katanaI18nNoFrontmatter(source) {
  return { prefix: "", body: source };
}

function katanaI18nParsedFrontmatter(source) {
  const split = katanaI18nFrontmatterSplit(source);
  return katanaI18nFrontmatterSplitReaders()[Number(split >= 0)](source, split);
}

function katanaI18nFrontmatterSplitReaders() {
  return [katanaI18nNoFrontmatter, katanaI18nSplitFrontmatter];
}

function katanaI18nFrontmatterSplit(source) {
  const offset = source.indexOf("---");
  const rest = source.slice(offset + 3);
  const end = rest.indexOf("\n---");
  return katanaI18nFrontmatterEndReaders()[Number(end >= 0)](source, offset, end);
}

function katanaI18nFrontmatterEndReaders() {
  return [katanaI18nMissingFrontmatterEnd, katanaI18nFrontmatterNextLine];
}

function katanaI18nMissingFrontmatterEnd() {
  return -1;
}

function katanaI18nFrontmatterNextLine(source, offset, end) {
  const prefixEnd = offset + 3 + end + "\n---".length;
  const nextLine = source.indexOf("\n", prefixEnd);
  return [nextLine + 1, source.length][Number(nextLine < 0)];
}

function katanaI18nSplitFrontmatter(source, split) {
  return {
    prefix: source.slice(0, split),
    body: source.slice(split),
  };
}

function katanaI18nDiagramType(body) {
  return body.split(/\s+/, 1)[0] ?? "";
}

function katanaPushI18nReplacement(replacements, text) {
  const placeholder = `KI${String(replacements.length).padStart(3, "0")}`;
  replacements.push({ placeholder, text });
  return placeholder;
}

function katanaNeedsI18nPlaceholder(text) {
  return Array.from(text).some(katanaIsNonAsciiChar);
}

function katanaIsNonAsciiChar(text) {
  return (text.codePointAt(0) ?? 0) > 0x7f;
}

function katanaEscapeSvgText(text) {
  return String(text).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

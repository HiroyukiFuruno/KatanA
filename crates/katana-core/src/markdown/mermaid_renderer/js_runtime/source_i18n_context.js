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
      if (!katanaNeedsI18nPlaceholder(text)) {
        return text;
      }
      return katanaPushI18nReplacement(replacements, text);
    },
    id(text) {
      if (!katanaNeedsI18nPlaceholder(text)) {
        return text;
      }
      if (!idMap.has(text)) {
        idMap.set(text, katanaPushI18nReplacement(replacements, text));
      }
      return idMap.get(text);
    },
    result(sourceText) {
      return {
        source: sourceText,
        replacements,
      };
    },
  };
}

function katanaI18nFrontmatter(source) {
  if (!source.trimStart().startsWith("---")) {
    return { prefix: "", body: source };
  }
  const offset = source.indexOf("---");
  const rest = source.slice(offset + 3);
  const end = rest.indexOf("\n---");
  if (end < 0) {
    return { prefix: "", body: source };
  }
  const prefixEnd = offset + 3 + end + "\n---".length;
  const nextLine = source.indexOf("\n", prefixEnd);
  const split = nextLine < 0 ? source.length : nextLine + 1;
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
  return /[^\x00-\x7F]/.test(text);
}

function katanaEscapeSvgText(text) {
  return String(text).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

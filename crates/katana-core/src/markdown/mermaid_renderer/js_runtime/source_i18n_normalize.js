function katanaNormalizeMermaidSourceI18n(source) {
  const context = katanaI18nContext(source);
  const body = context.body.trimStart();
  const type = katanaI18nDiagramType(body);
  const normalizer = KATANA_I18N_SOURCE_NORMALIZERS[type];
  if (!normalizer) {
    return context.result(source);
  }
  return context.result(`${context.prefix}${normalizer(context, context.body)}`);
}

const KATANA_I18N_SOURCE_NORMALIZERS = {
  erDiagram: katanaNormalizeErSourceI18n,
  requirementDiagram: katanaNormalizeRequirementSourceI18n,
  quadrantChart: katanaNormalizeQuadrantSourceI18n,
  "xychart-beta": katanaNormalizeXyChartSourceI18n,
  "sankey-beta": katanaNormalizeSankeySourceI18n,
  "architecture-beta": katanaNormalizeBracketLabelSourceI18n,
  "ishikawa-beta": katanaNormalizeIshikawaSourceI18n,
  "wardley-beta": katanaNormalizeWardleySourceI18n,
};

function katanaNormalizeErSourceI18n(context, source) {
  return source
    .replace(
      /^(\s*)([^\s{]+)(\s+\S+\s+)([^\s:]+)(\s*:\s*)(.+?)\s*$/gm,
      (_match, before, from, relation, to, separator, label) => {
        return `${before}${context.id(from)}${relation}${context.id(to)}${separator}${context.label(label)}`;
      },
    )
    .replace(
      /^(\s*)([^\s{]+)(\s*\{)/gm,
      (_match, before, id, after) => `${before}${context.id(id)}${after}`,
    )
    .replace(
      /^(\s+\S+\s+)([^\s]+)(\s*)$/gm,
      (_match, before, field, after) => `${before}${context.label(field)}${after}`,
    );
}

function katanaNormalizeRequirementSourceI18n(context, source) {
  return source
    .replace(
      /^(\s*(?:requirement|element)\s+)([^\s{]+)(\s*\{)/gm,
      (_match, before, id, after) => `${before}${context.id(id)}${after}`,
    )
    .replace(/^(\s*(?:text|type)\s*:\s*)(.+?)\s*$/gm, (_match, before, text) => {
      return `${before}${context.label(text)}`;
    })
    .replace(
      /^(\s*)([^\s]+)(\s+-\s+[A-Za-z_]+\s+->\s+)([^\s]+)(\s*)$/gm,
      (_match, before, from, arrow, to, after) => {
        return `${before}${context.id(from)}${arrow}${context.id(to)}${after}`;
      },
    );
}

function katanaNormalizeQuadrantSourceI18n(context, source) {
  return source
    .replace(
      /^(\s*title\s+)(.+?)\s*$/gm,
      (_match, before, text) => `${before}${context.label(text)}`,
    )
    .replace(
      /^(\s*[xy]-axis\s+)(.+?)(\s+-->\s+)(.+?)\s*$/gm,
      (_match, before, left, arrow, right) =>
        `${before}${context.label(left)}${arrow}${context.label(right)}`,
    )
    .replace(
      /^(\s*quadrant-\d+\s+)(.+?)\s*$/gm,
      (_match, before, text) => `${before}${context.label(text)}`,
    )
    .replace(
      /^(\s*)([^:\n]+?)(\s*:\s*\[[^\]]+\]\s*)$/gm,
      (_match, before, text, after) => `${before}${context.label(text)}${after}`,
    );
}

function katanaNormalizeXyChartSourceI18n(context, source) {
  return source
    .replace(/^(\s*title\s+)"([^"]+)"\s*$/gm, (_match, before, text) => {
      return `${before}"${context.label(text)}"`;
    })
    .replace(/^(\s*x-axis\s+\[)([^\]]+)(\]\s*)$/gm, (_match, before, labels, after) => {
      return `${before}${katanaNormalizeCsvLabels(context, labels)}${after}`;
    })
    .replace(
      /^(\s*y-axis\s+)"([^"]+)"(\s+[-\d.]+\s+-->\s+[-\d.]+\s*)$/gm,
      (_match, before, text, after) => `${before}"${context.label(text)}"${after}`,
    );
}

function katanaNormalizeSankeySourceI18n(context, source) {
  return source.replace(/^([^,\n]+),([^,\n]+),(.+)$/gm, (_match, from, to, value) => {
    return `${context.id(from)},${context.id(to)},${value}`;
  });
}

function katanaNormalizeBracketLabelSourceI18n(context, source) {
  return source.replace(/\[([^\]\n]*[^\x00-\x7F][^\]\n]*)\]/g, (_match, text) => {
    return `[${context.label(text)}]`;
  });
}

function katanaNormalizeIshikawaSourceI18n(context, source) {
  return source.replace(/^(\s*)([^\s].*?)\s*$/gm, (_match, before, text) => {
    if (text === "ishikawa-beta") {
      return `${before}${text}`;
    }
    return `${before}${context.label(text)}`;
  });
}

function katanaNormalizeWardleySourceI18n(context, source) {
  return source
    .replace(
      /^(\s*title\s+)(.+?)\s*$/gm,
      (_match, before, text) => `${before}${context.label(text)}`,
    )
    .replace(
      /^(\s*(?:anchor|component)\s+)(.+?)(\s+\[[^\]]+\].*)$/gm,
      (_match, before, id, after) => `${before}${context.id(id)}${after}`,
    )
    .replace(
      /^(\s*)(.+?)(\s+->\s+)(.+?)\s*$/gm,
      (_match, before, from, arrow, to) => `${before}${context.id(from)}${arrow}${context.id(to)}`,
    )
    .replace(
      /^(\s*evolve\s+)(.+?)(\s+[-\d.]+\s*)$/gm,
      (_match, before, id, after) => `${before}${context.id(id)}${after}`,
    )
    .replace(/(\bnote\s+")([^"]+)(")/g, (_match, before, text, after) => {
      return `${before}${context.label(text)}${after}`;
    });
}

function katanaNormalizeCsvLabels(context, labels) {
  return labels
    .split(",")
    .map((label) => {
      const trimmed = label.trim();
      return label.replace(trimmed, context.label(trimmed));
    })
    .join(",");
}

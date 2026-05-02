(async () => katanaRenderMermaid())().catch((error) => {
  const detail = error?.stack ? error.stack : String(error);
  throw new Error(detail);
});

async function katanaRenderMermaid() {
  const request = globalThis.__katanaMermaidRequest;
  const mermaidValue = katanaMermaidGlobal();
  mermaidValue.initialize({
    startOnLoad: false,
    securityLevel: "loose",
    htmlLabels: false,
    theme: request.theme,
    flowchart: {
      htmlLabels: false,
      useMaxWidth: false,
    },
    sequence: {
      useMaxWidth: false,
    },
    themeVariables: katanaMermaidThemeVariables(request),
  });
  const preparedSource = katanaNormalizeMermaidSourceI18n(
    katanaPrepareMermaidSource(request.source),
  );
  globalThis.__katanaMermaidDiagramType = katanaMermaidDiagramType(preparedSource.source);
  const result = await mermaidValue.render(request.svgId, preparedSource.source);
  return katanaNormalizeMermaidSvg(
    katanaRestoreMermaidI18nText(result.svg, preparedSource.replacements),
    request,
  );
}

function katanaMermaidGlobal() {
  const mermaidValue = [globalThis.mermaid, globalThis.window.mermaid].find(Boolean);
  if (!mermaidValue) {
    throw new Error("Mermaid global was not registered");
  }
  return mermaidValue;
}

function katanaMermaidThemeVariables(request) {
  return {
    background: request.background,
    mainBkg: request.fill,
    primaryColor: request.fill,
    primaryTextColor: request.text,
    primaryBorderColor: request.stroke,
    secondaryColor: request.fill,
    secondaryTextColor: request.text,
    secondaryBorderColor: request.stroke,
    tertiaryColor: request.fill,
    tertiaryTextColor: request.text,
    tertiaryBorderColor: request.stroke,
    nodeTextColor: request.text,
    lineColor: request.arrow,
    textColor: request.text,
    edgeLabelBackground: request.fill,
    actorBkg: request.fill,
    actorTextColor: request.text,
    actorBorder: request.stroke,
    signalColor: request.arrow,
    signalTextColor: request.text,
    labelTextColor: request.text,
    noteBkgColor: request.fill,
    noteTextColor: request.text,
    noteBorderColor: request.stroke,
    clusterBkg: request.background,
    clusterBorder: request.stroke,
    titleColor: request.text,
  };
}

function katanaMermaidDiagramType(source) {
  const lines = String(source)
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
  return katanaMermaidDiagramTypeLine(lines)?.split(/\s+/)[0].toLowerCase() ?? "";
}

function katanaMermaidDiagramTypeLine(lines) {
  return katanaMermaidDiagramTypeLineReaders()[Number(lines[0] === "---")](lines);
}

function katanaMermaidDiagramTypeLineReaders() {
  return [katanaMermaidFirstDiagramTypeLine, katanaMermaidFrontmatterDiagramTypeLine];
}

function katanaMermaidFirstDiagramTypeLine(lines) {
  return lines[0];
}

function katanaMermaidFrontmatterDiagramTypeLine(lines) {
  // WHY: Frontmatter diagrams still need measurement fixes keyed by the actual diagram type.
  const endIndex = lines.slice(1).indexOf("---");
  return lines[katanaMermaidFrontmatterBodyStartIndex(endIndex)];
}

function katanaMermaidFrontmatterBodyStartIndex(endIndex) {
  return [0, endIndex + 2][Number(endIndex >= 0)];
}

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

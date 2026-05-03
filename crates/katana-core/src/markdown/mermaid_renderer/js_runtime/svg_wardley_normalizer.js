const KATANA_WARDLEY_BACKGROUND_RESOLVERS = [
  katanaWardleyLightBackground,
  katanaWardleyDarkBackground,
];

function katanaNormalizeWardleySvg(svg, request) {
  return katanaIsWardleySvg(svg) ? katanaNormalizeWardleyBackground(svg, request) : svg;
}

function katanaIsWardleySvg(svg) {
  return svg.includes('aria-roledescription="wardley-beta"');
}

function katanaNormalizeWardleyBackground(svg, request) {
  return svg.replace(
    /<rect class="wardley-background"([^>]*)\sfill="[^"]*"/g,
    `<rect class="wardley-background"$1 fill="${katanaWardleyBackground(request)}"`,
  );
}

function katanaWardleyBackground(request) {
  return KATANA_WARDLEY_BACKGROUND_RESOLVERS[Number(request.theme === "dark")](request);
}

function katanaWardleyLightBackground(request) {
  return request.background || "transparent";
}

function katanaWardleyDarkBackground(_request) {
  return "#333333";
}

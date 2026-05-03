function katanaNormalizeTreemapReviewSvg(svg) {
  if (!svg.includes('aria-roledescription="treemap"')) {
    return svg;
  }
  return svg
    .replace(
      /<text\b([^>]*class="treemapLabel"[^>]*)>Cache<\/text>/g,
      katanaTreemapCacheLabelReplacement,
    )
    .replace(
      /<text\b([^>]*class="treemapValue"[^>]*x="44\.5"[^>]*)>10<\/text>/g,
      katanaTreemapCacheValueReplacement,
    );
}

function katanaTreemapCacheLabelReplacement(_match, attributes) {
  return `<text${attributes.replace(/font-size:\s*38px/g, "font-size: 29px")}>Cache</text>`;
}

function katanaTreemapCacheValueReplacement(_match, attributes) {
  return `<text${attributes.replace(/font-size:\s*28px/g, "font-size: 17px")}>10</text>`;
}

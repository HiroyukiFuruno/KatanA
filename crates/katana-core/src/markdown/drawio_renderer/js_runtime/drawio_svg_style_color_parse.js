const KATANA_DRAWIO_DARK_SCALE_RULES = [
  { luminance: 0.82, saturation: 0.25, scale: 0.14 },
  { luminance: 0.82, saturation: Number.POSITIVE_INFINITY, scale: 0.35 },
  { luminance: 0.62, saturation: 0.2, scale: 0.24 },
  { luminance: 0.62, saturation: Number.POSITIVE_INFINITY, scale: 0.45 },
  { luminance: 0.42, saturation: Number.POSITIVE_INFINITY, scale: 0.62 },
  { luminance: Number.NEGATIVE_INFINITY, saturation: Number.POSITIVE_INFINITY, scale: 1 },
];
const KATANA_DRAWIO_HEX_COLOR_READERS = [katanaDrawioFullHexChannels, katanaDrawioShortHexChannels];

function katanaDrawioDarkScale(color) {
  const metrics = katanaDrawioColorMetrics(color);
  return KATANA_DRAWIO_DARK_SCALE_RULES.find((rule) =>
    katanaDrawioDarkScaleRuleMatches(metrics, rule),
  ).scale;
}

function katanaDrawioColorMetrics(color) {
  return {
    luminance: katanaDrawioRgbLuminance(color),
    saturation: katanaDrawioRgbSaturation(color),
  };
}

function katanaDrawioDarkScaleRuleMatches(metrics, rule) {
  return [metrics.luminance > rule.luminance, metrics.saturation < rule.saturation].every(Boolean);
}

function katanaDrawioScaledDarkColor(color, scale) {
  return color.map((channel) => Math.round(channel * scale));
}

function katanaDrawioColorLuminance(value) {
  return [katanaDrawioParsedColor(value)]
    .filter(Boolean)
    .map(katanaDrawioRgbLuminance)
    .concat([1])[0];
}

function katanaDrawioRgbLuminance(color) {
  return (0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2]) / 255;
}

function katanaDrawioRgbSaturation(color) {
  const channels = color.map((channel) => channel / 255);
  const max = Math.max(...channels);
  const min = Math.min(...channels);
  return max === 0 ? 0 : (max - min) / max;
}

function katanaDrawioParsedColor(value) {
  return katanaDrawioParsedHexColor(value) || katanaDrawioParsedRgbColor(value);
}

function katanaDrawioParsedHexColor(value) {
  return KATANA_DRAWIO_HEX_COLOR_READERS.map((reader) => reader(value))
    .filter(Boolean)
    .concat([null])[0];
}

function katanaDrawioFullHexChannels(value) {
  const match = /^#([0-9a-f]{6})$/iu.exec(value);
  return match ? katanaDrawioHexChannels(match[1]) : null;
}

function katanaDrawioShortHexChannels(value) {
  const match = /^#([0-9a-f]{3})$/iu.exec(value);
  return match ? katanaDrawioHexChannels(katanaDrawioExpandedHex(match[1])) : null;
}

function katanaDrawioExpandedHex(hex) {
  return hex
    .split("")
    .map((channel) => `${channel}${channel}`)
    .join("");
}

function katanaDrawioHexChannels(hex) {
  return [hex.slice(0, 2), hex.slice(2, 4), hex.slice(4, 6)].map((channel) =>
    Number.parseInt(channel, 16),
  );
}

function katanaDrawioParsedRgbColor(value) {
  const match = /^rgb\((\d+),\s*(\d+),\s*(\d+)\)$/iu.exec(value);
  return match ? match.slice(1, 4).map(Number) : null;
}

function katanaDrawioRgbHex(color) {
  return `#${color.map(katanaDrawioHexChannel).join("")}`;
}

function katanaDrawioHexChannel(channel) {
  return channel.toString(16).padStart(2, "0");
}

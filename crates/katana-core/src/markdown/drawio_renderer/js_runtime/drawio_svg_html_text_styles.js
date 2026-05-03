function katanaDrawioHtmlTextOuterStyle(style, box) {
  return [
    "display: flex",
    `align-items: unsafe ${katanaDrawioHtmlTextAlignItems(style)}`,
    `justify-content: unsafe ${katanaDrawioHtmlTextJustifyContent(style)}`,
    `width: ${katanaDrawioHtmlTextWidth(style, box)}px`,
    "height: 1px",
    `padding-top: ${katanaDrawioHtmlTextTop(style, box)}px`,
    `margin-left: ${katanaDrawioHtmlTextLeft(style, box)}px`,
  ].join("; ");
}

function katanaDrawioHtmlTextBoxStyle(style) {
  return [
    "box-sizing: border-box",
    "font-size: 0",
    `text-align: ${katanaDrawioTextAlign(style)}`,
    `color: ${katanaDrawioTextColor(style)}`,
    "",
  ].join("; ");
}

function katanaDrawioHtmlTextContentStyle(style) {
  return [
    "display: inline-block",
    `font-size: ${katanaDrawioFontSize(style)}px`,
    `font-family: ${katanaDrawioHtmlFontFamily(style)}`,
    `color: ${katanaDrawioHtmlTextColor(style)}`,
    "line-height: 1.2",
    "pointer-events: all",
    "white-space: normal",
    "word-wrap: normal",
    "",
  ].join("; ");
}

function katanaDrawioHtmlTextAlignItems(style) {
  return (
    KATANA_DRAWIO_HTML_TEXT_ALIGN_ITEMS.get(
      katanaDrawioStyleValueFromMap(style, "verticalAlign", "middle"),
    ) ?? "center"
  );
}

function katanaDrawioHtmlTextJustifyContent(style) {
  return (
    KATANA_DRAWIO_HTML_TEXT_JUSTIFY_CONTENT.get(
      katanaDrawioStyleValueFromMap(style, "align", "center"),
    ) ?? "center"
  );
}

function katanaDrawioTextAlign(style) {
  return katanaDrawioStyleValueFromMap(style, "align", "center");
}

function katanaDrawioHtmlFontFamily(style) {
  const family = katanaDrawioStyleValueFromMap(style, "fontFamily", "Helvetica");
  return family.includes(" ") ? `'${family}'` : family;
}

function katanaDrawioHtmlTextTop(style, box) {
  return Math.round(
    (
      KATANA_DRAWIO_HTML_TEXT_TOP_BY_ALIGNMENT.get(
        katanaDrawioStyleValueFromMap(style, "verticalAlign", "middle"),
      ) ?? katanaDrawioMiddleHtmlTextTop
    )(box),
  );
}

function katanaDrawioTopHtmlTextTop(box) {
  return box.y + KATANA_DRAWIO_HTML_TEXT_TOP_PADDING;
}

function katanaDrawioMiddleHtmlTextTop(box) {
  return box.y + box.height / 2;
}

function katanaDrawioBottomHtmlTextTop(box) {
  return box.y + box.height - KATANA_DRAWIO_HTML_TEXT_TOP_PADDING;
}

function katanaDrawioHtmlTextWidth(style, box) {
  return katanaDrawioIsRightHtmlTextLabel(style) ? 1 : Math.max(1, Math.round(box.width - 2));
}

function katanaDrawioHtmlTextLeft(style, box) {
  return Math.round(
    katanaDrawioIsRightHtmlTextLabel(style)
      ? box.x + box.width + 2 + katanaDrawioHtmlTextSpacingLeft(style)
      : box.x + KATANA_DRAWIO_HTML_TEXT_LEFT_PADDING,
  );
}

function katanaDrawioIsRightHtmlTextLabel(style) {
  return katanaDrawioStyleValue(style, "labelPosition") === "right";
}

function katanaDrawioHtmlTextSpacingLeft(style) {
  const value = Number(katanaDrawioStyleValue(style, "spacingLeft"));
  return [value].filter(Number.isFinite).concat([0])[0];
}

function katanaDrawioHtmlTextColor(style) {
  const color = katanaDrawioTextColor(style);
  return KATANA_DRAWIO_HTML_TEXT_DARK_COLOR.get(color.toLowerCase()) ?? color;
}

const KATANA_DRAWIO_HTML_TEXT_TOP_PADDING = 3;
const KATANA_DRAWIO_HTML_TEXT_LEFT_PADDING = 1;

const KATANA_DRAWIO_HTML_TEXT_TOP_BY_ALIGNMENT = new Map([
  ["top", katanaDrawioTopHtmlTextTop],
  ["middle", katanaDrawioMiddleHtmlTextTop],
  ["bottom", katanaDrawioBottomHtmlTextTop],
]);

const KATANA_DRAWIO_HTML_TEXT_ALIGN_ITEMS = new Map([
  ["top", "flex-start"],
  ["middle", "center"],
  ["bottom", "flex-end"],
]);

const KATANA_DRAWIO_HTML_TEXT_JUSTIFY_CONTENT = new Map([
  ["left", "flex-start"],
  ["center", "center"],
  ["right", "flex-end"],
]);

const KATANA_DRAWIO_HTML_TEXT_DARK_COLOR = new Map([["#000000", "light-dark(#000000, #ffffff)"]]);

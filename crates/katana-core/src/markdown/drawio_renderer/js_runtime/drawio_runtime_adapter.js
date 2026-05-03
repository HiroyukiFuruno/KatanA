function katanaInstallDrawioRuntimeAdapter() {
  globalThis.__katanaDrawioMissingResources = [];
  globalThis.__katanaDrawioResourceErrors = [];
  const context = katanaDrawioRuntimeContext();
  katanaRegisterDrawioResources(context);
  katanaInstallDrawioStencilLoader(context);
  katanaInstallDrawioMxUtils(context);
  katanaInstallDrawioTextConversionGuard();
  katanaInstallDrawioSvgTextOutput();
  katanaInstallDrawioImageBundles(context);
  katanaInstallDrawioSvgImages(context);
  katanaInstallDrawioOwnerSvgElement();
}

katanaInstallDrawioRuntimeAdapter();

function katanaInstallDrawioTextConversionGuard() {
  const originalConvertHtmlToText = Editor.convertHtmlToText;
  Editor.convertHtmlToText = function convertHtmlToText(value) {
    return KATANA_DRAWIO_TEXT_CONVERSION_HANDLERS[Number(value != null)](
      originalConvertHtmlToText,
      value,
    );
  };
}

const KATANA_DRAWIO_TEXT_CONVERSION_HANDLERS = [
  () => "",
  (originalConvertHtmlToText, value) => originalConvertHtmlToText.call(Editor, value),
];

function katanaDrawioHtmlLabelText(value) {
  return decodeHtmlEntities(
    String(value)
      .replace(/<\s*br\s*\/?>/gi, "\n")
      .replace(/<[^>]+>/g, ""),
  );
}

function katanaInstallDrawioSvgTextOutput() {
  katanaDisableDrawioForeignObjectWarning();
}

function katanaDisableDrawioForeignObjectWarning() {
  if (typeof Graph === "function") {
    Graph.prototype.addForeignObjectWarning = function addForeignObjectWarning() {};
  }
}

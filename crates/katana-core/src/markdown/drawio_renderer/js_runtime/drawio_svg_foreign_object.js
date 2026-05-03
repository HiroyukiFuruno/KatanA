const KATANA_DRAWIO_XHTML_NAMESPACE = "http://www.w3.org/1999/xhtml";

function katanaNormalizeDrawioForeignObjects(svg) {
  Array.from(svg.querySelectorAll("foreignObject")).forEach(katanaNormalizeDrawioForeignObject);
}

function katanaNormalizeDrawioForeignObject(foreignObject) {
  foreignObject.remove();
}

function katanaApplyDrawioXhtmlNamespace(node) {
  if (node.nodeType !== Node.ELEMENT_NODE) {
    return;
  }
  node.setAttribute("xmlns", KATANA_DRAWIO_XHTML_NAMESPACE);
  Array.from(node.children).forEach(katanaApplyDrawioXhtmlNamespace);
}

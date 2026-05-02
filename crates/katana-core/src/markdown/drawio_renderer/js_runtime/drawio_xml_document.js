function katanaDrawioCreateXmlDocument() {
  const detached = Object.create(document);
  katanaSetupDrawioXmlDocumentBase(detached);
  katanaSetupDrawioXmlDocumentFactories(detached);
  katanaSetupDrawioXmlDocumentQueries(detached);
  return detached;
}

function katanaSetupDrawioXmlDocumentBase(detached) {
  detached.nodeType = 9;
  detached.documentElement = null;
  detached.body = null;
  detached.defaultView = globalThis;
}

function katanaSetupDrawioXmlDocumentFactories(detached) {
  detached.createElement = (tagName) =>
    katanaDrawioOwnedNode(document.createElement(tagName), detached);
  detached.createElementNS = (namespaceURI, tagName) =>
    katanaDrawioOwnedNode(document.createElementNS(namespaceURI, tagName), detached);
  detached.createTextNode = (value) =>
    katanaDrawioOwnedNode(document.createTextNode(value), detached);
  detached.createDocumentFragment = () =>
    katanaDrawioOwnedNode(document.createDocumentFragment(), detached);
  detached.importNode = (node, deep) => katanaDrawioOwnedTree(node.cloneNode(deep), detached);
  detached.appendChild = (child) => katanaDrawioAppendDocumentChild(detached, child);
}

function katanaSetupDrawioXmlDocumentQueries(detached) {
  detached.getElementById = (id) =>
    katanaDrawioDocumentElement(detached)?.querySelector(`#${id}`) ?? null;
  detached.getElementsByTagName = (tagName) =>
    katanaDrawioDocumentElement(detached)?.getElementsByTagName(tagName) ?? [];
}

function katanaDrawioAppendDocumentChild(detached, child) {
  katanaDrawioOwnedTree(child, detached);
  child.parentNode = detached;
  detached.documentElement = child;
  return child;
}

function katanaDrawioDocumentElement(detached) {
  return detached.documentElement;
}

function katanaDrawioOwnedNode(node, ownerDocument) {
  node.ownerDocument = ownerDocument;
  return node;
}

function katanaDrawioOwnedTree(node, ownerDocument) {
  katanaDrawioOwnedNode(node, ownerDocument);
  katanaDrawioNodeChildren(node).forEach((child) => {
    katanaDrawioOwnedTree(child, ownerDocument);
  });
  return node;
}

function katanaDrawioNodeChildren(node) {
  if (node.children) {
    return node.children;
  }
  return [];
}

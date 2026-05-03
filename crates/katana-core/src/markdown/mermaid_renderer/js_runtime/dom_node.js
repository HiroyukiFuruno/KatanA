function katanaEscapeSvgAttribute(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function katanaEscapeSvgText(value) {
  return String(value).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

class KatanaNode {
  constructor(tagName, namespaceURI = null) {
    const rawName = katanaRawNodeName(tagName);
    const preservesCase = katanaPreservesNodeCase(namespaceURI);
    this.serializedName = katanaSerializedNodeName(rawName, preservesCase);
    this.tagName = katanaTagNodeName(rawName, preservesCase);
    this.nodeName = this.tagName;
    this.localName = rawName.toLowerCase();
    this.namespaceURI = namespaceURI;
    this.children = [];
    this.childNodes = this.children;
    this.parentNode = null;
    this.ownerDocument = null;
    this.nodeType = katanaNodeType(tagName);
    this.attributes = [];
    this.attributeMap = {};
    this.style = new KatanaStyle();
    this.dataset = {};
    this._textContent = "";
    this.className = "";
    this.id = "";
  }
  appendChild(child) {
    child.parentNode = this;
    child.ownerDocument = this.ownerDocument;
    this.children.push(child);
    this.childNodes = this.children;
    return child;
  }
  insertBefore(child, reference) {
    child.parentNode = this;
    child.ownerDocument = this.ownerDocument;
    const index = this.children.indexOf(reference);
    if (index < 0) {
      this.children.push(child);
      this.childNodes = this.children;
      return child;
    }
    this.children.splice(index, 0, child);
    this.childNodes = this.children;
    return child;
  }
  removeChild(child) {
    this.children = this.children.filter((candidate) => candidate !== child);
    this.childNodes = this.children;
    return child;
  }
  remove() {
    if (this.parentNode) {
      this.parentNode.removeChild(this);
    }
  }
  get firstChild() {
    return this.children[0] ?? null;
  }
  get lastChild() {
    return this.children[this.children.length - 1] ?? null;
  }
  get parentElement() {
    return this.parentNode;
  }
  getRootNode() {
    return this.ownerDocument ?? document;
  }
  hasChildNodes() {
    return this.children.length > 0;
  }
  compareDocumentPosition(other) {
    if (this === other) {
      return 0;
    }
    return 4;
  }
  get nextSibling() {
    if (!this.parentNode) {
      return null;
    }
    const index = this.parentNode.children.indexOf(this);
    return this.parentNode.children[index + 1] ?? null;
  }
  get previousSibling() {
    if (!this.parentNode) {
      return null;
    }
    const index = this.parentNode.children.indexOf(this);
    return this.parentNode.children[index - 1] ?? null;
  }
  setAttribute(name, value) {
    const normalized = String(name);
    const attribute = katanaUpsertAttribute(this, normalized);
    attribute.value = String(value);
    katanaSyncIdAttribute(this, normalized, value);
    katanaSyncClassAttribute(this, normalized, value);
  }
  getAttribute(name) {
    return this.attributeMap[String(name)]?.value ?? null;
  }
  removeAttribute(name) {
    const normalized = String(name);
    delete this.attributeMap[normalized];
    this.attributes = this.attributes.filter((attribute) => attribute.name !== normalized);
  }
  setAttributeNS(_namespace, name, value) {
    this.setAttribute(name, value);
  }
  getAttributeNS(_namespace, name) {
    return this.getAttribute(name);
  }
  hasAttribute(name) {
    return Object.hasOwn(this.attributeMap, String(name));
  }
  querySelector(selector) {
    const result = katanaFirstQuerySelectorResult(this, selector);
    if (result === null) {
      globalThis.__katanaMissingSelectors.push(`${this.localName}:${selector}`);
    }
    return result;
  }
  querySelectorAll(selector) {
    const compound = queryCompoundSelector(this, String(selector));
    if (compound !== null) {
      return compound;
    }
    return querySimpleSelector(this, String(selector));
  }
  getElementsByTagName(tagName) {
    return this.querySelectorAll(String(tagName).toLowerCase());
  }
  addEventListener() {}
  removeEventListener() {}
  toJSON() {
    return {};
  }
}

function katanaFirstQuerySelectorResult(node, selector) {
  return node.querySelectorAll(selector)[0] ?? null;
}

function katanaRawNodeName(tagName) {
  return String(tagName || "");
}

function katanaPreservesNodeCase(namespaceURI) {
  return [
    katanaNamespaceIncludes(namespaceURI, "svg"),
    katanaNamespaceIncludes(namespaceURI, "xml"),
  ].includes(true);
}

function katanaNamespaceIncludes(namespaceURI, value) {
  return String(namespaceURI || "").includes(value);
}

function katanaSerializedNodeName(rawName, preservesCase) {
  return preservesCase ? rawName : rawName.toLowerCase();
}

function katanaTagNodeName(rawName, preservesCase) {
  return preservesCase ? rawName : rawName.toUpperCase();
}

function katanaNodeType(tagName) {
  return tagName === "#text" ? 3 : 1;
}

function katanaUpsertAttribute(node, normalized) {
  const attribute = node.attributeMap[normalized];
  if (attribute) {
    return attribute;
  }
  return katanaCreateAttribute(node, normalized);
}

function katanaCreateAttribute(node, normalized) {
  const attribute = { name: normalized, nodeName: normalized, value: "" };
  node.attributeMap[normalized] = attribute;
  node.attributes.push(attribute);
  return attribute;
}

function katanaSyncIdAttribute(node, normalized, value) {
  if (normalized === "id") {
    node.id = String(value);
  }
}

function katanaSyncClassAttribute(node, normalized, value) {
  if (normalized === "class") {
    node.className = String(value);
  }
}

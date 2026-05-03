const document = {
  nodeType: 9,
  currentScript: null,
  addEventListener() {},
  removeEventListener() {},
  createElement(tagName) {
    const node = new KatanaNode(tagName);
    node.ownerDocument = document;
    return node;
  },
  createElementNS(namespaceURI, tagName) {
    const node = new KatanaNode(tagName, namespaceURI);
    node.ownerDocument = document;
    return node;
  },
  createTextNode(value) {
    const node = new KatanaTextNode(value);
    node.ownerDocument = document;
    return node;
  },
  createComment(value) {
    return this.createTextNode(value);
  },
  createDocumentFragment() {
    const node = new KatanaNode("#document-fragment");
    node.nodeType = 11;
    node.ownerDocument = document;
    return node;
  },
  createNodeIterator(root) {
    const nodes = [];
    const visit = (node) => {
      nodes.push(node);
      node.children.forEach(visit);
    };
    visit(root);
    let index = 0;
    return { nextNode: () => nodes[index++] ?? null };
  },
  getElementsByTagName(tagName) {
    return this.documentElement.querySelectorAll(String(tagName).toLowerCase());
  },
  importNode(node, deep) {
    const clone = node.cloneNode(deep);
    clone.ownerDocument = document;
    return clone;
  },
  getElementById(id) {
    const result = this.documentElement.querySelector(`#${id}`);
    if (result === null) {
      globalThis.__katanaMissingSelectors.push(`document.getElementById:${id}`);
    }
    return result;
  },
  querySelector(selector) {
    if (selector === "body") {
      return this.body;
    }
    return this.documentElement.querySelector(selector);
  },
  querySelectorAll(selector) {
    return this.documentElement.querySelectorAll(selector);
  },
  implementation: {
    createHTMLDocument() {
      return createDetachedDocument();
    },
  },
  fonts: {
    ready: Promise.resolve(),
  },
};

document.documentElement = document.createElement("html");
document.head = document.createElement("head");
document.body = document.createElement("body");
document.documentElement.appendChild(document.head);
document.documentElement.appendChild(document.body);
document.defaultView = globalThis;
globalThis.document = document;
globalThis.Element = KatanaNode;
globalThis.HTMLElement = KatanaNode;
globalThis.SVGElement = KatanaNode;
globalThis.Node = KatanaNode;
globalThis.Node.ELEMENT_NODE = 1;
globalThis.Node.TEXT_NODE = 3;
globalThis.Node.DOCUMENT_NODE = 9;
globalThis.Node.DOCUMENT_FRAGMENT_NODE = 11;
globalThis.DocumentFragment = KatanaNode;
globalThis.HTMLTemplateElement = KatanaNode;
globalThis.HTMLFormElement = KatanaNode;
globalThis.NamedNodeMap = Array;
globalThis.NodeFilter = { SHOW_ELEMENT: 1, SHOW_TEXT: 4, SHOW_COMMENT: 128 };
globalThis.trustedTypes = {
  createPolicy() {
    return {
      createHTML: (value) => String(value),
      createScriptURL: (value) => String(value),
    };
  },
};

globalThis.TextEncoder = class {
  encode(value) {
    return new Uint8Array(Array.from(String(value)).flatMap((char) => katanaUtf8Bytes(char)));
  }
};

function katanaUtf8Bytes(char) {
  const code = char.charCodeAt(0);
  if (code < 0x80) {
    return [code];
  }
  return katanaMultibyteUtf8Bytes(code);
}

function katanaMultibyteUtf8Bytes(code) {
  if (code < 0x800) {
    return [0xc0 | (code >> 6), 0x80 | (code & 0x3f)];
  }
  return [0xe0 | (code >> 12), 0x80 | ((code >> 6) & 0x3f), 0x80 | (code & 0x3f)];
}

globalThis.TextDecoder = class {
  decode(value) {
    return Array.from(value)
      .map((byte) => String.fromCharCode(byte))
      .join("");
  }
};
globalThis.structuredClone = (value) => JSON.parse(JSON.stringify(value));
globalThis.XMLSerializer = class {
  serializeToString(node) {
    return node.outerHTML ?? "";
  }
};
globalThis.DOMParser = class {
  parseFromString(source) {
    const parsed = parseInnerHtml(String(source).trim(), true);
    const detached = createDetachedDocument();
    const root = parsed[0] ?? detached.createElement("xml");
    attachOwnerDocument(root, detached);
    detached.documentElement = root;
    detached.body = root;
    return detached;
  }
};

function createDetachedDocument() {
  const detached = Object.create(document);
  detached.documentElement = document.createElement("html");
  detached.head = document.createElement("head");
  detached.body = document.createElement("body");
  detached.documentElement.ownerDocument = detached;
  detached.head.ownerDocument = detached;
  detached.body.ownerDocument = detached;
  detached.documentElement.appendChild(detached.head);
  detached.documentElement.appendChild(detached.body);
  detached.defaultView = globalThis;
  detached.nodeType = 9;
  return detached;
}

function attachOwnerDocument(node, ownerDocument) {
  node.ownerDocument = ownerDocument;
  for (const child of node.children) {
    attachOwnerDocument(child, ownerDocument);
  }
}

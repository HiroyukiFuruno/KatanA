function parseInnerHtml(source, xmlMode = false) {
  const root = {
    localName: "#root",
    children: [],
    appendChild(child) {
      this.children.push(child);
    },
  };
  const stack = [root];
  const tokenRegex = /<\/?([a-zA-Z0-9:_-]+)([^>]*)>|([^<]+)/g;
  Array.from(source.matchAll(tokenRegex)).forEach((match) => {
    appendHtmlToken(stack, match, xmlMode);
  });
  return root.children;
}

function appendHtmlToken(stack, match, xmlMode) {
  if (match[3] !== undefined) {
    appendHtmlText(stack[stack.length - 1], match[3]);
    return;
  }
  appendHtmlTag(stack, match, xmlMode);
}

function appendHtmlTag(stack, match, xmlMode) {
  if (match[0].startsWith("</")) {
    popHtmlStack(stack, match[1]);
    return;
  }
  appendHtmlStartTag(stack, match, xmlMode);
}

function appendHtmlStartTag(stack, match, xmlMode) {
  const node = new KatanaNode(match[1], katanaHtmlNamespace(xmlMode));
  node.ownerDocument = document;
  parseAttributes(match[2]).forEach(([name, value]) => {
    node.setAttribute(name, value);
  });
  stack[stack.length - 1].appendChild(node);
  pushHtmlElementIfOpen(stack, node, match[0], match[1]);
}

function katanaHtmlNamespace(xmlMode) {
  return xmlMode ? "katana-xml" : null;
}

function pushHtmlElementIfOpen(stack, node, fullTag, tagName) {
  if (katanaIsOpenHtmlTag(fullTag, tagName)) {
    stack.push(node);
  }
}

function katanaIsOpenHtmlTag(fullTag, tagName) {
  return [!fullTag.endsWith("/>"), !isHtmlVoidTag(tagName)].every(Boolean);
}

function appendHtmlText(parent, value) {
  const text = decodeHtmlEntities(value);
  if (text.length > 0) {
    parent.appendChild(new KatanaTextNode(text));
  }
}

function popHtmlStack(stack, tagName) {
  const normalized = String(tagName).toLowerCase();
  const index = stack.findLastIndex((node) => node.localName === normalized);
  if (index > 0) {
    stack.splice(index);
  }
}

function isHtmlVoidTag(tagName) {
  return new Set(["br", "hr", "img", "input", "meta", "link"]).has(String(tagName).toLowerCase());
}

function parseAttributes(source) {
  const attrRegex = /([a-zA-Z0-9:_-]+)="([^"]*)"/g;
  return Array.from(source.matchAll(attrRegex)).map((match) => [
    match[1],
    decodeHtmlEntities(match[2]),
  ]);
}

function decodeHtmlEntities(value) {
  return String(value)
    .replace(/&amp;nbsp;/g, "\u00A0")
    .replace(/&nbsp;/g, "\u00A0")
    .replace(/&#xa;/gi, "\n")
    .replace(/&#10;/g, "\n")
    .replace(/&#160;/g, "\u00A0")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&amp;/g, "&")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'");
}

class KatanaTextNode extends KatanaNode {
  constructor(value) {
    super("#text");
    this.textContent = String(value);
  }
  get outerHTML() {
    return katanaEscapeSvgText(this.textContent);
  }
}

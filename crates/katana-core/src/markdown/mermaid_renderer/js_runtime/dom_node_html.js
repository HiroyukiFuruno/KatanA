KatanaNode.prototype.getBBox = function getBBox() {
  const text = katanaNodeTextForBox(this);
  const childWidth = katanaNodeChildWidth(this);
  const width = Math.max(80, childWidth, katanaTextWidth(text));
  const height = Math.max(24, this.children.length * 24);
  return { x: 0, y: 0, width, height };
};

function katanaNodeTextForBox(node) {
  return String(katanaFirstTextValue([node.textContent, node.innerText]));
}

function katanaFirstTextValue(values) {
  return values.find(Boolean) ?? "";
}

function katanaNodeChildWidth(node) {
  return node.children.reduce((max, child) => Math.max(max, child.getBBox().width), 0);
}

KatanaNode.prototype.getBoundingClientRect = function getBoundingClientRect() {
  const box = this.getBBox();
  return {
    ...box,
    top: box.y,
    left: box.x,
    right: box.x + box.width,
    bottom: box.y + box.height,
  };
};

KatanaNode.prototype.getComputedTextLength = function getComputedTextLength() {
  return Math.max(16, katanaTextWidth(String(this.textContent || "")));
};

KatanaNode.prototype.getContext = function getContext() {
  return {
    font: "",
    measureText(value) {
      return { width: Math.max(16, katanaTextWidth(String(value || ""))) };
    },
  };
};

Object.defineProperty(KatanaNode.prototype, "innerHTML", {
  get() {
    if (this.children.length === 0) {
      return this.textContent;
    }
    return this.children.map((child) => child.outerHTML ?? child.textContent ?? "").join("");
  },
  set(value) {
    katanaClearInnerHtml(this);
    katanaApplyInnerHtml(this, String(value), parseInnerHtml(String(value)));
  },
});

function katanaClearInnerHtml(node) {
  node.textContent = "";
  node.children = [];
  node.childNodes = node.children;
}

function katanaApplyInnerHtml(node, value, parsed) {
  if (parsed.length === 0) {
    node.textContent = value;
    return;
  }
  katanaAppendParsedHtml(node, parsed);
}

function katanaAppendParsedHtml(node, parsed) {
  for (const child of parsed) {
    node.appendChild(child);
  }
}

Object.defineProperty(KatanaNode.prototype, "outerHTML", {
  get() {
    const attrs = katanaSerializedAttributeText(this);
    const body = katanaOuterHtmlBody(this);
    return `<${this.serializedName}${attrs}>${body}</${this.serializedName}>`;
  },
});

function katanaSerializedAttributeText(node) {
  return Object.entries({ ...node.serializedAttributes(), ...katanaStyleAttribute(node) })
    .map(([key, value]) => ` ${key}="${katanaEscapeSvgAttribute(value)}"`)
    .join("");
}

function katanaStyleAttribute(node) {
  if (katanaShouldSerializeStyle(node)) {
    return { style: node.style.cssText };
  }
  return {};
}

function katanaShouldSerializeStyle(node) {
  return [node.style.cssText, !node.hasAttribute("style")].every(Boolean);
}

function katanaOuterHtmlBody(node) {
  if (node.children.length === 0) {
    return katanaEscapeSvgText(node.textContent);
  }
  return node.innerHTML;
}

KatanaNode.prototype.cloneNode = function cloneNode(deep = false) {
  const clone = katanaShallowCloneNode(this);
  katanaCloneChildrenIfDeep(this, clone, deep);
  return clone;
};

function katanaShallowCloneNode(node) {
  const clone = new KatanaNode(node.serializedName, node.namespaceURI);
  node.attributes.forEach((attribute) => {
    clone.setAttribute(attribute.name, attribute.value);
  });
  clone.style = node.style.clone();
  clone._textContent = node._textContent ?? "";
  clone.className = node.className;
  clone.id = node.id;
  clone.ownerDocument = node.ownerDocument;
  return clone;
}

function katanaCloneChildrenIfDeep(node, clone, deep) {
  if (deep) {
    katanaCloneChildren(node, clone);
  }
}

function katanaCloneChildren(node, clone) {
  for (const child of node.children) {
    clone.appendChild(child.cloneNode(true));
  }
}

KatanaNode.prototype.serializedAttributes = function serializedAttributes() {
  return Object.fromEntries(this.attributes.map((attribute) => [attribute.name, attribute.value]));
};

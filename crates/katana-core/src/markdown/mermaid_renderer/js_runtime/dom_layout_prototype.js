Object.defineProperty(KatanaNode.prototype, "textContent", {
  get() {
    return katanaTextContent(this);
  },
  set(value) {
    this._textContent = String(value ?? "");
    this.children = [];
    this.childNodes = this.children;
  },
});

Object.defineProperty(KatanaNode.prototype, "innerText", {
  get() {
    return this.textContent;
  },
  set(value) {
    this.textContent = value;
  },
});

Object.defineProperty(KatanaNode.prototype, "nodeValue", {
  get() {
    return this.nodeType === 3 ? this.textContent : null;
  },
  set(value) {
    if (this.nodeType === 3) {
      this.textContent = value;
    }
  },
});

KatanaNode.prototype.append = function append(...values) {
  for (const value of values) {
    this.appendChild(katanaNodeValue(value));
  }
};

KatanaNode.prototype.prepend = function prepend(...values) {
  for (const value of values.reverse()) {
    this.insertBefore(katanaNodeValue(value), this.firstChild);
  }
};

KatanaNode.prototype.replaceChildren = function replaceChildren(...values) {
  this.children = [];
  this.childNodes = this.children;
  this.append(...values);
};

KatanaNode.prototype.contains = function contains(candidate) {
  if (this === candidate) {
    return true;
  }
  return katanaNodeChildren(this).some((child) => child.contains(candidate));
};

function katanaNodeChildren(node) {
  return node.children ?? [];
}

KatanaNode.prototype.text = function text(...values) {
  if (values.length === 0) return this.textContent;
  this.textContent = values[0];
  return this;
};

KatanaNode.prototype.getBBox = function getBBox() {
  return katanaMeasuredBox(this);
};

KatanaNode.prototype.getBoundingClientRect = function getBoundingClientRect() {
  const box = katanaElementClientBox(this);
  return { ...box, top: box.y, left: box.x, right: box.x + box.width, bottom: box.y + box.height };
};

KatanaNode.prototype.getComputedTextLength = function getComputedTextLength() {
  return Math.max(16, katanaTextContent(this).length * 8);
};

KatanaNode.prototype.getScreenCTM = function getScreenCTM() {
  return { a: 1, b: 0, c: 0, d: 1, e: 0, f: 0, inverse: () => this.getScreenCTM() };
};

KatanaNode.prototype.createSVGPoint = function createSVGPoint() {
  return {
    x: 0,
    y: 0,
    matrixTransform() {
      return { x: this.x, y: this.y };
    },
  };
};

Object.defineProperties(KatanaNode.prototype, {
  clientWidth: {
    get() {
      return Math.ceil(katanaElementClientBox(this).width);
    },
  },
  clientHeight: {
    get() {
      return Math.ceil(katanaElementClientBox(this).height);
    },
  },
  offsetWidth: {
    get() {
      return Math.ceil(katanaElementClientBox(this).width);
    },
  },
  offsetHeight: {
    get() {
      return Math.ceil(katanaElementClientBox(this).height);
    },
  },
  scrollWidth: {
    get() {
      return Math.ceil(katanaElementClientBox(this).width);
    },
  },
  scrollHeight: {
    get() {
      return Math.ceil(katanaElementClientBox(this).height);
    },
  },
});

function katanaElementClientBox(node) {
  return katanaMeasuredClientBox(node);
}

function katanaDetachChild(child) {
  const parent = child?.parentNode;
  if (!parent) return;
  katanaReplaceChildList(
    parent,
    katanaChildList(parent).filter((candidate) => candidate !== child),
  );
  child.parentNode = null;
}

function katanaChildList(parent) {
  return [parent.children, parent.childNodes, []].find(Array.isArray);
}

function katanaReplaceChildList(parent, children) {
  parent.children = children;
  parent.childNodes = children;
}

function katanaAdoptChild(parent, child) {
  child.parentNode = parent;
  child.ownerDocument = parent.ownerDocument ?? parent;
}

function katanaAppendFragment(parent, fragment) {
  const nodes = [...fragment.childNodes];
  for (const node of nodes) {
    parent.appendChild(node);
  }
  return fragment;
}

KatanaNode.prototype.appendChild = function appendChild(child) {
  if (child.nodeType === Node.DOCUMENT_FRAGMENT_NODE) {
    return katanaAppendFragment(this, child);
  }
  katanaDetachChild(child);
  katanaAdoptChild(this, child);
  this.children.push(child);
  this.childNodes = this.children;
  return child;
};

KatanaNode.prototype.insertBefore = function insertBefore(child, reference) {
  if (child.nodeType === Node.DOCUMENT_FRAGMENT_NODE) {
    return katanaInsertFragmentBefore(this, child, reference);
  }
  return katanaInsertNodeBefore(this, child, reference);
};

function katanaInsertFragmentBefore(parent, fragment, reference) {
  for (const node of fragment.childNodes) {
    parent.insertBefore(node, reference);
  }
  return fragment;
}

function katanaInsertNodeBefore(parent, child, reference) {
  if (child === reference) {
    return child;
  }
  return katanaInsertDetachedNodeBefore(parent, child, reference);
}

function katanaInsertDetachedNodeBefore(parent, child, reference) {
  katanaDetachChild(child);
  katanaAdoptChild(parent, child);
  katanaInsertAtReference(parent, child, reference);
  parent.childNodes = parent.children;
  return child;
}

function katanaInsertAtReference(parent, child, reference) {
  const index = parent.children.indexOf(reference);
  if (index < 0) {
    parent.children.push(child);
    return;
  }
  parent.children.splice(index, 0, child);
}

KatanaNode.prototype.removeChild = function removeChild(child) {
  this.children = this.children.filter((candidate) => candidate !== child);
  this.childNodes = this.children;
  child.parentNode = null;
  return child;
};

Object.defineProperty(KatanaNode.prototype, "firstElementChild", {
  get() {
    return this.children.find((child) => child.nodeType === Node.ELEMENT_NODE) ?? null;
  },
});

function katanaDocumentPath(node) {
  const path = [];
  let current = node;
  while (current?.parentNode) {
    path.unshift(current.parentNode.children.indexOf(current));
    current = current.parentNode;
  }
  return path;
}

function katanaComparePath(left, right) {
  const mismatch = katanaFirstPathMismatch(left, right);
  if (mismatch !== null) {
    return katanaPathMismatchPosition(left, right, mismatch);
  }
  return katanaPathLengthPosition(left, right);
}

function katanaFirstPathMismatch(left, right) {
  return katanaPathIndexes(left, right).find((index) => left[index] !== right[index]) ?? null;
}

function katanaPathIndexes(left, right) {
  return Array.from({ length: Math.min(left.length, right.length) }, (_value, index) => index);
}

function katanaPathMismatchPosition(left, right, index) {
  return left[index] < right[index] ? 4 : 2;
}

function katanaPathLengthPosition(left, right) {
  if (left.length === right.length) {
    return 0;
  }
  return katanaUnequalPathLengthPosition(left, right);
}

function katanaUnequalPathLengthPosition(left, right) {
  return left.length < right.length ? 20 : 10;
}

KatanaNode.prototype.compareDocumentPosition = function compareDocumentPosition(other) {
  if (this === other) return 0;
  return katanaComparePath(katanaDocumentPath(this), katanaDocumentPath(other));
};

Node.DOCUMENT_POSITION_DISCONNECTED = 1;
Node.DOCUMENT_POSITION_PRECEDING = 2;
Node.DOCUMENT_POSITION_FOLLOWING = 4;
Node.DOCUMENT_POSITION_CONTAINS = 8;
Node.DOCUMENT_POSITION_CONTAINED_BY = 16;

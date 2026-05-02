function matchesSelector(node, selector) {
  const matcher = KATANA_SELECTOR_MATCHERS.find((entry) => entry.applies(selector));
  if (matcher) {
    return matcher.matches(node, selector);
  }
  return katanaMatchesTag(node, selector);
}

const KATANA_SELECTOR_MATCHERS = [
  { applies: katanaIsBareFirstChildSelector, matches: katanaMatchesBareFirstChild },
  { applies: katanaIsNestedFirstChildSelector, matches: katanaMatchesNestedFirstChild },
  { applies: katanaIsWildcardSelector, matches: katanaMatchesWildcard },
  { applies: katanaIsTagIdSelector, matches: katanaMatchesTagId },
  { applies: katanaIsBodySelector, matches: katanaMatchesBody },
  { applies: katanaIsIdSelector, matches: katanaMatchesId },
  { applies: katanaIsIdAttributeSelector, matches: katanaMatchesIdAttribute },
  { applies: katanaIsClassSelector, matches: katanaMatchesClass },
];

function katanaIsBareFirstChildSelector(selector) {
  return [":first-child", "::first-child"].includes(selector);
}

function katanaMatchesBareFirstChild(node) {
  return node.parentNode?.firstElementChild === node;
}

function katanaIsNestedFirstChildSelector(selector) {
  return selector.endsWith(":first-child");
}

function katanaMatchesNestedFirstChild(node, selector) {
  const base = selector.replace(/:{1,2}first-child$/, "");
  return [matchesSelector(node, base), katanaMatchesBareFirstChild(node)].every(Boolean);
}

function katanaIsWildcardSelector(selector) {
  return selector === "*";
}

function katanaMatchesWildcard(node) {
  return node.nodeType === Node.ELEMENT_NODE;
}

function katanaIsTagIdSelector(selector) {
  return /^([a-zA-Z0-9:_-]+)#(.+)$/.test(selector);
}

function katanaMatchesTagId(node, selector) {
  const match = selector.match(/^([a-zA-Z0-9:_-]+)#(.+)$/);
  return [node.localName === match[1].toLowerCase(), node.id === match[2]].every(Boolean);
}

function katanaIsBodySelector(selector) {
  return selector === "body";
}

function katanaMatchesBody(node) {
  return node === document.body;
}

function katanaIsIdSelector(selector) {
  return selector.startsWith("#");
}

function katanaMatchesId(node, selector) {
  return node.id === selector.slice(1);
}

function katanaIsIdAttributeSelector(selector) {
  return /^\[id="([^"]+)"\]$/.test(selector);
}

function katanaMatchesIdAttribute(node, selector) {
  return node.id === selector.match(/^\[id="([^"]+)"\]$/)[1];
}

function katanaIsClassSelector(selector) {
  return selector.startsWith(".");
}

function katanaMatchesClass(node, selector) {
  return String(node.className).split(/\s+/).includes(selector.slice(1));
}

function katanaMatchesTag(node, selector) {
  return node.localName === selector.toLowerCase();
}

function queryCompoundSelector(root, selector) {
  const parts = selector.split(/\s+/).filter(Boolean);
  if (parts.length <= 1) {
    return null;
  }
  return katanaQuerySelectorParts([root], parts);
}

function katanaQuerySelectorParts(candidates, parts) {
  return parts.reduce(
    (current, part) => current.flatMap((candidate) => querySimpleSelector(candidate, part)),
    candidates,
  );
}

function querySimpleSelector(root, selector) {
  const results = [];
  root.children.forEach((node) => {
    katanaVisitSelector(node, selector, results);
  });
  return results;
}

function katanaVisitSelector(node, selector, results) {
  katanaAddMatchingSelector(node, selector, results);
  node.children.forEach((child) => {
    katanaVisitSelector(child, selector, results);
  });
}

function katanaAddMatchingSelector(node, selector, results) {
  if (matchesSelector(node, selector)) {
    results.push(node);
  }
}

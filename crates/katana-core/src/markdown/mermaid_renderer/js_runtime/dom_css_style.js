function katanaCssComputedStyleValue(node, name) {
  const classNames = katanaCssClassNames(node);
  if (classNames.length === 0) {
    return "";
  }
  return katanaCssRuleValue(katanaCssText(node), classNames, name);
}

function katanaCssClassNames(node) {
  return String(node?.className ?? "")
    .split(/\s+/)
    .filter(Boolean);
}

function katanaCssText(node) {
  const root = katanaCssRoot(node);
  return katanaCssStyleNodes(root)
    .map((styleNode) => styleNode.textContent)
    .join("\n");
}

function katanaCssRoot(node) {
  return katanaCssRootNode(node) || document.documentElement;
}

function katanaCssRootNode(node) {
  return KATANA_CSS_ROOT_READERS[Number(Boolean(katanaCssParentNode(node)))](node);
}

function katanaCssParentNode(node) {
  return node?.parentNode || null;
}

function katanaCssStyleNodes(root) {
  return katanaCssDescendantNodes(root).filter(katanaIsCssStyleNode);
}

function katanaCssDescendantNodes(node) {
  return [node].filter(Boolean).flatMap(katanaCssNodeWithDescendants);
}

function katanaCssNodeWithDescendants(node) {
  return [node].concat(Array.from(node.children || []).flatMap(katanaCssNodeWithDescendants));
}

function katanaIsCssStyleNode(node) {
  return node.localName === "style";
}

function katanaCssRuleValue(cssText, classNames, name) {
  return (
    Array.from(cssText.matchAll(/([^{}]+)\{([^{}]+)\}/g))
      .map((rule) => katanaCssRuleDeclarationValue(rule, classNames, name))
      .find(Boolean) ?? ""
  );
}

function katanaCssRuleDeclarationValue(rule, classNames, name) {
  return (
    [rule]
      .filter((it) => katanaCssRuleMatchesClass(it[1], classNames))
      .map((it) => katanaCssDeclarationValue(it[2], name))[0] ?? ""
  );
}

const KATANA_CSS_ROOT_READERS = [
  (node) => node,
  (node) => katanaCssRootNode(katanaCssParentNode(node)),
];

function katanaCssRuleMatchesClass(selectorText, classNames) {
  return selectorText
    .split(",")
    .some((selector) =>
      classNames.some((className) => katanaCssSelectorMatches(selector, className)),
    );
}

function katanaCssSelectorMatches(selector, className) {
  const classSelector = `.${className}`;
  return String(selector)
    .trim()
    .split(/\s+/)
    .some((part) => part === classSelector);
}

function katanaCssDeclarationValue(declarations, name) {
  return (
    String(declarations)
      .split(";")
      .map((declaration) => katanaCssDeclarationEntry(declaration))
      .find((entry) => entry.name === name)?.value ?? ""
  );
}

function katanaCssDeclarationEntry(declaration) {
  const parts = String(declaration).split(":");
  return {
    name: String(parts.shift() ?? "").trim(),
    value: parts.join(":").trim(),
  };
}

function katanaNormalizeClassSvg(svg) {
  if (!svg.includes('aria-roledescription="class"')) {
    return svg;
  }
  return katanaRewriteBalancedGroups(
    svg,
    /<g class="node default " id="[^"]*classId-[^"]*"/g,
    katanaNormalizeEmptyClassMethods,
  );
}

function katanaNormalizeEmptyClassMethods(node) {
  if (!katanaHasEmptyClassMethods(node)) {
    return katanaAddClassRowTextClass(node);
  }
  return katanaNormalizeEmptyClassMethodsWithLayout(node);
}

function katanaHasEmptyClassMethods(node) {
  return /<g class="methods-group text" transform="translate\([^)]+\)"><\/g>/.test(node);
}

function katanaNormalizeEmptyClassMethodsWithLayout(node) {
  const layout = katanaReadEmptyClassLayout(node);
  if (!layout) {
    return katanaAddClassRowTextClass(node);
  }
  return katanaTrimEmptyClassMethods(node, layout);
}

function katanaReadEmptyClassLayout(node) {
  const outer = katanaReadClassOuterBox(node);
  const dividerY = katanaReadLastClassDividerY(node);
  if (!katanaHasClassLayout(outer, dividerY)) {
    return null;
  }
  return { dividerY, outer };
}

function katanaHasClassLayout(outer, dividerY) {
  return [Boolean(outer), dividerY !== null].every(Boolean);
}

function katanaTrimEmptyClassMethods(node, layout) {
  const trim = Math.max(0, Math.min(60, layout.outer.bottom - layout.dividerY - 24));
  if (trim < 1) {
    return katanaAddClassRowTextClass(node);
  }
  return katanaApplyEmptyClassTrim(node, layout.outer, trim);
}

function katanaApplyEmptyClassTrim(node, outer, trim) {
  const shift = trim / 2;
  return katanaAddClassRowTextClass(
    katanaShiftClassNodeY(
      katanaCompressClassOuterPath(katanaShiftClassInnerGroups(node, shift), outer, shift),
      -shift,
    ),
  );
}

function katanaReadClassOuterBox(node) {
  const path = node.match(
    /<g class="basic label-container outer-path">[\s\S]*?<path d="([^"]+)"/,
  )?.[1];
  return katanaReadClassPathBox(path);
}

function katanaReadLastClassDividerY(node) {
  const dividers = Array.from(
    node.matchAll(/<g class="divider" style="">[\s\S]*?<path d="([^"]+)"/g),
  )
    .map((match) => katanaReadClassPathBox(match[1]))
    .filter(Boolean);
  if (dividers.length === 0) {
    return null;
  }
  return Math.max(...dividers.map((divider) => divider.top));
}

function katanaReadClassPathBox(pathData) {
  if (!pathData) {
    return null;
  }
  return katanaClassPathBoxFromValues(katanaClassPathNumbers(pathData));
}

function katanaClassPathNumbers(pathData) {
  return Array.from(pathData.matchAll(/-?\d+(?:\.\d+)?/g)).map((match) => Number(match[0]));
}

function katanaClassPathBoxFromValues(numbers) {
  if (numbers.length < 4) {
    return null;
  }
  return katanaClassPathBoxFromCoordinates(numbers);
}

function katanaClassPathBoxFromCoordinates(numbers) {
  const ys = numbers.filter((_value, index) => index % 2 === 1);
  return { top: Math.min(...ys), bottom: Math.max(...ys) };
}

function katanaShiftClassNodeY(node, delta) {
  return node.replace(
    /(<g class="node default " id="[^"]*classId-[^"]*"[^>]* transform="translate\([-\d.]+, )([-\d.]+)(\)")/,
    (_match, start, y, end) => `${start}${katanaFormatClassNumber(Number(y) + delta)}${end}`,
  );
}

function katanaShiftClassInnerGroups(node, shift) {
  return node
    .replace(
      /(<g class="(?:annotation|label|members|methods)-group text" transform="translate\([-\d.]+, )([-\d.]+)(\)")/g,
      (_match, start, y, end) => `${start}${katanaFormatClassNumber(Number(y) + shift)}${end}`,
    )
    .replace(
      /(<g class="divider" style="">\s*<path d=")([^"]+)(")/g,
      (_match, start, pathData, end) => `${start}${katanaShiftClassPathY(pathData, shift)}${end}`,
    );
}

function katanaCompressClassOuterPath(node, outer, shift) {
  return node.replace(/(<g class="basic label-container outer-path">[\s\S]*?<\/g>)/, (group) =>
    group.replace(
      /d="([^"]+)"/g,
      (_match, pathData) => `d="${katanaCompressClassPathY(pathData, outer, shift)}"`,
    ),
  );
}

function katanaCompressClassPathY(pathData, outer, shift) {
  return katanaRewriteClassPathY(pathData, (value) =>
    katanaShiftCompressedClassValue(value, outer, shift),
  );
}

function katanaShiftCompressedClassValue(value, outer, shift) {
  if (katanaShouldShiftClassPathValue(value, outer)) {
    return value + shift;
  }
  return value - shift;
}

function katanaShouldShiftClassPathValue(value, outer) {
  if (value <= 0) {
    return true;
  }
  return Math.abs(value - outer.top) < Math.abs(value - outer.bottom);
}

function katanaShiftClassPathY(pathData, shift) {
  return katanaRewriteClassPathY(pathData, (value) => value + shift);
}

function katanaRewriteClassPathY(pathData, transform) {
  const state = { index: 0 };
  return pathData.replace(/-?\d+(?:\.\d+)?/g, (raw) =>
    katanaRewriteClassPathValue(state, raw, transform),
  );
}

function katanaRewriteClassPathValue(state, raw, transform) {
  const value = Number(raw);
  const next = katanaClassPathYValue(state.index, value, transform);
  state.index += 1;
  return katanaFormatClassNumber(next);
}

function katanaClassPathYValue(index, value, transform) {
  if (index % 2 === 1) {
    return transform(value);
  }
  return value;
}

function katanaAddClassRowTextClass(node) {
  return node.replace(/class="text-outer-tspan(?! row)"/g, 'class="text-outer-tspan row"');
}

function katanaFormatClassNumber(value) {
  return Number(value.toFixed(6)).toString();
}

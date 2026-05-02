function katanaTranslate(tag) {
  const match = tag.match(/transform="translate\(([-\d.]+)[,\s]+([-\d.]+)\)/);
  if (match) {
    return [Number(match[1]), Number(match[2])];
  }
  return [0, 0];
}

function katanaAttrNumber(tag, name) {
  const match = tag.match(new RegExp(`(?:^|\\s)${name}="([^"]+)"`));
  if (!match) {
    return null;
  }
  return katanaFiniteAttrNumber(Number(match[1]));
}

function katanaFiniteAttrNumber(value) {
  if (Number.isFinite(value)) {
    return value;
  }
  return null;
}

function katanaAttrText(tag, name) {
  return tag.match(new RegExp(`(?:^|\\s)${name}="([^"]+)"`))?.[1] ?? null;
}

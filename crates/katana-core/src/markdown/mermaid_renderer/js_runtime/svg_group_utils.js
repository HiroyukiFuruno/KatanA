function katanaRewriteBalancedGroups(text, pattern, transform) {
  const state = { cursor: 0, result: "" };
  let match = pattern.exec(text);
  while (match !== null) {
    katanaAppendBalancedGroup(state, text, pattern, match, transform);
    match = pattern.exec(text);
  }
  return state.result + text.slice(state.cursor);
}

function katanaAppendBalancedGroup(state, text, pattern, match, transform) {
  const start = match.index;
  const end = katanaFindBalancedGroupEnd(text, start);
  if (end < 0) {
    return;
  }
  state.result += text.slice(state.cursor, start);
  state.result += transform(text.slice(start, end));
  state.cursor = end;
  pattern.lastIndex = end;
}

function katanaFindBalancedGroupEnd(text, start) {
  const pattern = /<\/?g\b[^>]*>/g;
  const state = { depth: 0, end: -1 };
  pattern.lastIndex = start;
  let match = pattern.exec(text);
  while (match !== null) {
    katanaApplyGroupDepth(state, match, pattern);
    match = katanaNextGroupMatch(text, pattern, state);
  }
  return state.end;
}

function katanaApplyGroupDepth(state, match, pattern) {
  state.depth += katanaGroupDepthDelta(match[0]);
  katanaCaptureGroupEnd(state, pattern);
}

function katanaGroupDepthDelta(tag) {
  if (tag.startsWith("</")) {
    return -1;
  }
  return 1;
}

function katanaCaptureGroupEnd(state, pattern) {
  if (state.depth === 0) {
    state.end = pattern.lastIndex;
  }
}

function katanaNextGroupMatch(text, pattern, state) {
  if (state.end >= 0) {
    return null;
  }
  return pattern.exec(text);
}

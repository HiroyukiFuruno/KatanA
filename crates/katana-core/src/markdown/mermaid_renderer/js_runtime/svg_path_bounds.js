function katanaSerializedSvgPathBox(tag, offset) {
  return katanaOffsetBox(katanaSerializedPathDataBox(katanaAttrText(tag, "d")), offset);
}

function katanaSerializedPathDataBox(pathData) {
  return pathData ? katanaNonEmptySerializedPathDataBox(pathData) : null;
}

function katanaNonEmptySerializedPathDataBox(pathData) {
  return (
    katanaCurvedSerializedPathDataBox(pathData) ??
    katanaParsedSerializedPathDataBox(pathData) ??
    katanaNumberPairsBox(katanaSerializedSvgNumberList(pathData))
  );
}

function katanaCurvedSerializedPathDataBox(pathData) {
  return katanaIshikawaHeadPathBox(pathData);
}

function katanaIshikawaHeadPathBox(pathData) {
  const match = pathData.match(/^M 0 -?([\d.]+) L 0 ([\d.]+) Q ([\d.]+) 0 0 -?[\d.]+ Z$/);
  if (!match) {
    return null;
  }
  const halfHeight = Math.max(Number(match[1]), Number(match[2]));
  return [0, -halfHeight, Number(match[3]) / 2, halfHeight];
}

function katanaOffsetNumberListBox(value, offset) {
  return katanaOffsetBox(katanaNumberPairsBox(katanaSerializedSvgNumberList(value)), offset);
}

function katanaParsedSerializedPathDataBox(pathData) {
  const parser = katanaPathParser(pathData);
  while (parser.index < parser.tokens.length) {
    katanaReadPathCommand(parser);
  }
  return katanaNumberPairsBox(parser.points);
}

function katanaPathParser(pathData) {
  return {
    tokens: Array.from(String(pathData).matchAll(/[a-zA-Z]|-?(?:\d*\.)?\d+(?:e-?\d+)?/gi)).map(
      (it) => it[0],
    ),
    index: 0,
    command: "",
    x: 0,
    y: 0,
    startX: 0,
    startY: 0,
    points: [],
  };
}

function katanaReadPathCommand(parser) {
  parser.command = katanaNextPathCommand(parser);
  const command = parser.command.toUpperCase();
  const relative = parser.command !== command;
  const reader = KATANA_PATH_COMMAND_READERS[command];
  if (reader) {
    reader(parser, relative);
    return;
  }
  parser.index += 1;
}

function katanaNextPathCommand(parser) {
  if (katanaIsPathCommand(parser.tokens[parser.index])) {
    return parser.tokens[parser.index++];
  }
  return parser.command;
}

function katanaIsPathCommand(token) {
  return /^[a-zA-Z]$/.test(String(token ?? ""));
}

const KATANA_PATH_COMMAND_READERS = {
  A: (parser, relative) => katanaReadPathArcs(parser, relative),
  C: (parser, relative) => katanaReadPathCoordinateGroups(parser, relative, 3),
  H: (parser, relative) => katanaReadPathHorizontalLines(parser, relative),
  L: (parser, relative) => katanaReadPathCoordinateGroups(parser, relative, 1),
  M: (parser, relative) => katanaReadPathMoves(parser, relative),
  Q: (parser, relative) => katanaReadPathCoordinateGroups(parser, relative, 2),
  S: (parser, relative) => katanaReadPathCoordinateGroups(parser, relative, 2),
  T: (parser, relative) => katanaReadPathCoordinateGroups(parser, relative, 1),
  V: (parser, relative) => katanaReadPathVerticalLines(parser, relative),
  Z: (parser) => katanaClosePath(parser),
};

function katanaReadPathMoves(parser, relative) {
  while (katanaCanReadPathNumbers(parser, 2)) {
    katanaSetPathPoint(parser, katanaReadPathPoint(parser, relative));
    parser.startX = parser.x;
    parser.startY = parser.y;
  }
}

function katanaReadPathCoordinateGroups(parser, relative, pointCount) {
  const numberCount = pointCount * 2;
  KATANA_PATH_COORDINATE_GROUP_READERS[Number(katanaCanReadPathNumbers(parser, numberCount))](
    parser,
    relative,
    pointCount,
    numberCount,
  );
}

function katanaReadPathPointGroup(parser, relative, pointCount) {
  Array.from({ length: pointCount }).forEach(() => {
    katanaSetPathPoint(parser, katanaReadPathPoint(parser, relative));
  });
}

const KATANA_PATH_COORDINATE_GROUP_READERS = [
  () => {},
  (parser, relative, pointCount, numberCount) => {
    katanaReadPathPointGroup(parser, relative, pointCount);
    katanaReadPathCoordinateGroups(parser, relative, numberCount / 2);
  },
];

function katanaReadPathHorizontalLines(parser, relative) {
  while (katanaCanReadPathNumbers(parser, 1)) {
    katanaSetPathPoint(parser, [
      katanaPathCoordinate(parser, katanaReadPathNumber(parser), relative, "x"),
      parser.y,
    ]);
  }
}

function katanaReadPathVerticalLines(parser, relative) {
  while (katanaCanReadPathNumbers(parser, 1)) {
    katanaSetPathPoint(parser, [
      parser.x,
      katanaPathCoordinate(parser, katanaReadPathNumber(parser), relative, "y"),
    ]);
  }
}

function katanaReadPathArcs(parser, relative) {
  while (katanaCanReadPathNumbers(parser, 7)) {
    parser.index += 5;
    katanaSetPathPoint(parser, katanaReadPathPoint(parser, relative));
  }
}

function katanaClosePath(parser) {
  katanaSetPathPoint(parser, [parser.startX, parser.startY]);
}

function katanaReadPathPoint(parser, relative) {
  return [
    katanaPathCoordinate(parser, katanaReadPathNumber(parser), relative, "x"),
    katanaPathCoordinate(parser, katanaReadPathNumber(parser), relative, "y"),
  ];
}

function katanaPathCoordinate(parser, value, relative, axis) {
  return relative ? parser[axis] + value : value;
}

function katanaSetPathPoint(parser, point) {
  parser.x = point[0];
  parser.y = point[1];
  parser.points.push(point[0], point[1]);
}

function katanaCanReadPathNumbers(parser, count) {
  const nextTokens = parser.tokens.slice(parser.index, parser.index + count);
  return nextTokens.length === count && nextTokens.every((token) => !katanaIsPathCommand(token));
}

function katanaReadPathNumber(parser) {
  return Number(parser.tokens[parser.index++]);
}

function katanaSerializedSvgNumberList(value) {
  return Array.from(String(value).matchAll(/-?(?:\d*\.)?\d+(?:e-?\d+)?/gi)).map((it) =>
    Number(it[0]),
  );
}

function katanaNumberPairsBox(numbers) {
  if (numbers.length < 2) {
    return null;
  }
  const xValues = numbers.filter((_value, index) => index % 2 === 0);
  const yValues = numbers.filter((_value, index) => index % 2 === 1);
  return [Math.min(...xValues), Math.min(...yValues), Math.max(...xValues), Math.max(...yValues)];
}

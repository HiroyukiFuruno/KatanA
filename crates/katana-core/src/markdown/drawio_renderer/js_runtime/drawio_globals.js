globalThis.navigator = {
  userAgent: "KatanA Rust-managed Draw.io runtime",
  appName: "Netscape",
  language: "en-US",
  languages: ["en-US", "en"],
  appVersion: "KatanA",
  vendor: "",
  platform: "MacIntel",
  maxTouchPoints: 0,
};
globalThis.location = {
  href: "https://viewer.diagrams.net/?offline=1&embed=1&proto=json",
  host: "viewer.diagrams.net",
  hostname: "viewer.diagrams.net",
  protocol: "https:",
  search: "?offline=1&embed=1&proto=json",
  hash: "",
  pathname: "/",
  toString() {
    return this.href;
  },
};
globalThis.window.location = globalThis.location;
globalThis.window.innerWidth = 1280;
globalThis.window.innerHeight = 800;
globalThis.window.top = globalThis.window;
globalThis.document.referrer = "";
globalThis.document.location = globalThis.location;
globalThis.document.compatMode = "CSS1Compat";
globalThis.localStorage = {
  getItem() {
    return null;
  },
  setItem() {},
  removeItem() {},
};
globalThis.sessionStorage = globalThis.localStorage;
globalThis.atob = katanaDrawioAtob;
Date.prototype.toLocaleDateString = katanaDrawioLocaleDate;
Date.prototype.toLocaleString = katanaDrawioLocaleDate;

function katanaDrawioLocaleDate() {
  return this.toISOString().slice(0, 10);
}

function katanaDrawioAtob(value) {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
  const state = { bits: 0, buffer: 0, output: "", stopped: false };
  const input = String(value).replace(/[^A-Za-z0-9+/=]/g, "");
  Array.from(input).forEach((char) => {
    katanaDrawioReadBase64Char(state, chars.indexOf(char));
  });
  return state.output;
}

function katanaDrawioReadBase64Char(state, code) {
  if (state.stopped) {
    return;
  }
  katanaDrawioApplyBase64Code(state, code);
}

function katanaDrawioApplyBase64Code(state, code) {
  if (katanaDrawioStopsBase64(code)) {
    state.stopped = true;
    return;
  }
  state.buffer = (state.buffer << 6) | code;
  state.bits += 6;
  katanaDrawioFlushBase64Byte(state);
}

function katanaDrawioStopsBase64(code) {
  return [code < 0, code === 64].some(Boolean);
}

function katanaDrawioFlushBase64Byte(state) {
  if (state.bits >= 8) {
    state.bits -= 8;
    state.output += String.fromCharCode((state.buffer >> state.bits) & 0xff);
  }
}
globalThis.urlParams = globalThis.urlParams || {};
globalThis.mxLoadResources = false;
globalThis.mxForceIncludes = false;
globalThis.mxResourceExtension = ".txt";
globalThis.mxBasePath = "";
globalThis.STYLE_PATH = "styles";
globalThis.SHAPES_PATH = "shapes";
globalThis.STENCIL_PATH = "stencils";
globalThis.IMAGE_PATH = "";
globalThis.RESOURCES_PATH = "";
globalThis.GRAPH_IMAGE_PATH = "img";
globalThis.EXPORT_URL = "";

if (typeof KatanaNode === "function") {
  Object.defineProperty(KatanaNode.prototype, "classList", {
    get() {
      return katanaDrawioClassList(this);
    },
  });
  KatanaNode.prototype.toString = function katanaDrawioNodeToString() {
    return katanaDrawioNodeClassName(this);
  };
}

function katanaDrawioClassList(node) {
  return {
    add(name) {
      katanaSetDrawioClasses(node, [...katanaDrawioClasses(node), name]);
    },
    remove(name) {
      katanaSetDrawioClasses(
        node,
        katanaDrawioClasses(node).filter((it) => it !== name),
      );
    },
    contains(name) {
      return katanaDrawioClasses(node).includes(name);
    },
  };
}

function katanaDrawioClasses(node) {
  return String(node.className || "")
    .split(/\s+/)
    .filter(Boolean);
}

function katanaSetDrawioClasses(node, values) {
  node.setAttribute("class", Array.from(new Set(values)).join(" "));
}

function katanaDrawioNodeClassName(node) {
  if (katanaIsDrawioSvgForeignObject(node)) {
    return "[object SVGForeignObjectElement]";
  }
  return "[object Element]";
}

function katanaIsDrawioSvgForeignObject(node) {
  return [
    katanaDrawioNodeNamespace(node).includes("svg"),
    katanaDrawioNodeLocalName(node) === "foreignobject",
  ].every(Boolean);
}

function katanaDrawioNodeNamespace(node) {
  return String(node.namespaceURI || "");
}

function katanaDrawioNodeLocalName(node) {
  return String(node.localName || "").toLowerCase();
}

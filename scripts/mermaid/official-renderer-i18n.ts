import os from "node:os";
import path from "node:path";

const I18N_RUNTIME_DIR = "crates/katana-core/src/markdown/mermaid_renderer/js_runtime";

export class MermaidI18nRuntimeScripts {
  static paths(): string[] {
    return [
      path.resolve(I18N_RUNTIME_DIR, "source_i18n_context.js"),
      path.resolve(I18N_RUNTIME_DIR, "source_i18n_normalize.js"),
    ];
  }
}

export function expandHome(value: string) {
  return value.startsWith("~/") ? path.join(os.homedir(), value.slice(2)) : value;
}

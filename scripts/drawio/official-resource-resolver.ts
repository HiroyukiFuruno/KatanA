import fs from "node:fs";
import path from "node:path";
import type { DrawioResourceManifest } from "./resource-common";

interface ResolvedDrawioResource {
  readonly filePath: string;
}

const MIME_TYPES_BY_EXTENSION = new Map([
  [".gif", "image/gif"],
  [".jpg", "image/jpeg"],
  [".jpeg", "image/jpeg"],
  [".png", "image/png"],
  [".svg", "image/svg+xml"],
  [".webp", "image/webp"],
]);

export class OfficialDrawioResourceResolver {
  private readonly resources = new Map<string, ResolvedDrawioResource>();

  constructor(resourcesDir: string, manifestPath: string) {
    this.load(resourcesDir, manifestPath);
  }

  resolveSvg(svg: string): string {
    return svg.replace(
      /(xlink:href|href)="([^"]+)"/g,
      (match: string, attributeName: string, value: string) =>
        this.resolvedAttribute(match, attributeName, value),
    );
  }

  private load(resourcesDir: string, manifestPath: string) {
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8")) as DrawioResourceManifest;
    for (const entry of manifest.resources) {
      this.resources.set(entry.logicalPath, {
        filePath: path.join(resourcesDir, entry.logicalPath),
      });
    }
  }

  private resolvedAttribute(match: string, attributeName: string, value: string): string {
    const resource = this.resources.get(this.logicalPath(value));
    return resource === undefined ? match : `${attributeName}="${this.dataUri(resource.filePath)}"`;
  }

  private logicalPath(value: string): string {
    return value
      .replace(/^https?:\/\/[^/]+\//, "")
      .replace(/^katana:\/\/drawio\//, "")
      .replace(/[?#].*$/, "")
      .replace(/^\/+/, "");
  }

  private dataUri(filePath: string): string {
    return `data:${this.mimeType(filePath)};base64,${fs.readFileSync(filePath).toString("base64")}`;
  }

  private mimeType(filePath: string): string {
    const mimeType = MIME_TYPES_BY_EXTENSION.get(path.extname(filePath).toLowerCase());
    if (mimeType === undefined) {
      throw new Error(`Unsupported Draw.io resource type: ${filePath}`);
    }
    return mimeType;
  }
}

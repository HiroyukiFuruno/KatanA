import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export interface DrawioResourceManifest {
  readonly version: number;
  readonly resourcesRoot: string;
  readonly resources: DrawioManifestEntry[];
}

export interface DrawioManifestEntry {
  readonly logicalPath: string;
  readonly sourceKind: string;
  readonly byteSize: number;
  readonly sha256: string;
}

export interface DrawioResourceReference {
  readonly fixturePath: string;
  readonly logicalPath: string;
  readonly sourceKind: string;
  readonly rawValue: string;
}

const RESOURCE_MANIFEST_FILE = "drawio-resource-manifest.json";
const SOURCE_KIND_BY_SEGMENT = new Map([
  ["assets", "drawio-webapp"],
  ["img", "drawio-webapp"],
  ["shapes", "drawio-shapes"],
  ["stencils", "drawio-stencils"],
]);

export class DrawioResourcePaths {
  static listFiles(root: string): string[] {
    return DrawioResourcePaths.walk(root)
      .filter((filePath) => path.basename(filePath) !== RESOURCE_MANIFEST_FILE)
      .sort((left, right) => left.localeCompare(right));
  }

  static logical(root: string, filePath: string): string {
    return path.relative(root, filePath).split(path.sep).join("/");
  }

  private static walk(root: string): string[] {
    const entries = fs.readdirSync(root, { withFileTypes: true });
    const files = entries
      .filter((entry) => entry.isFile())
      .map((entry) => path.join(root, entry.name));
    const directories = entries
      .filter((entry) => entry.isDirectory())
      .flatMap((entry) => DrawioResourcePaths.walk(path.join(root, entry.name)));
    return [...files, ...directories];
  }
}

export class DrawioResourceInventory {
  constructor(private resourcesDir: string) {}

  manifest(): DrawioResourceManifest {
    return {
      version: 1,
      resourcesRoot: DrawioResourcePaths.logical(process.cwd(), this.resourcesDir),
      resources: DrawioResourcePaths.listFiles(this.resourcesDir).map((filePath) =>
        this.entry(filePath),
      ),
    };
  }

  existingLogicalPaths(): Set<string> {
    return new Set(
      DrawioResourcePaths.listFiles(this.resourcesDir).map((filePath) =>
        DrawioResourcePaths.logical(this.resourcesDir, filePath),
      ),
    );
  }

  private entry(filePath: string): DrawioManifestEntry {
    const content = fs.readFileSync(filePath);
    const logicalPath = DrawioResourcePaths.logical(this.resourcesDir, filePath);
    return {
      logicalPath,
      sourceKind: DrawioSourceKind.fromLogicalPath(logicalPath),
      byteSize: content.byteLength,
      sha256: crypto.createHash("sha256").update(content).digest("hex"),
    };
  }
}

export class DrawioFixtureRepository {
  constructor(private fixturesDir: string) {}

  files(): string[] {
    return DrawioResourcePaths.listFiles(this.fixturesDir)
      .filter((filePath) => filePath.endsWith(".drawio"))
      .sort((left, right) => left.localeCompare(right));
  }
}

export class DrawioSourceKind {
  static fromLogicalPath(logicalPath: string): string {
    const firstSegment = logicalPath.split("/")[0];
    return [SOURCE_KIND_BY_SEGMENT.get(firstSegment), firstSegment]
      .filter(DrawioSourceKind.isString)
      .slice(0, 1)
      .join("");
  }

  private static isString(value: string | undefined): value is string {
    return typeof value === "string";
  }
}

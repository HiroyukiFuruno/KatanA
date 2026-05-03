import fs from "node:fs";
import path from "node:path";
import {
  DrawioFixtureRepository,
  DrawioResourceInventory,
  type DrawioResourceReference,
} from "./resource-common";
import { DrawioResourceReferenceExtractor } from "./resource-references";

const DRAWIO_SHAPE_SCRIPT_GROUPS = new Map([
  ["archimate3", ["shapes/mxArchiMate3.js"]],
  ["c4", ["shapes/mxC4.js"]],
  ["dfd", ["shapes/mxDFD.js"]],
  ["er", ["shapes/er/mxER.js"]],
  ["infographic", ["shapes/mxInfographic.js"]],
  ["pid2", ["shapes/pid2"]],
  ["sysml", ["shapes/mxSysML.js"]],
]);

interface CliParsedOptions {
  readonly fixturesDir: string;
  readonly resourcesDir: string;
  readonly manifestPath: string;
}

class CliOptions {
  static parse(argv: string[]): CliParsedOptions {
    CliOptions.exitIfHelp(argv);
    const resourcesDir = path.resolve(
      CliOptions.get(
        argv,
        "--resources",
        "crates/katana-core/src/markdown/drawio_renderer/js_runtime/resources",
      ),
    );
    return {
      fixturesDir: path.resolve(CliOptions.get(argv, "--fixtures", "assets/fixtures/drawio")),
      resourcesDir,
      manifestPath: path.resolve(
        CliOptions.get(
          argv,
          "--manifest",
          path.join(resourcesDir, "drawio-resource-manifest.json"),
        ),
      ),
    };
  }

  private static get(argv: string[], name: string, fallback: string): string {
    const index = argv.indexOf(name);
    return index >= 0 ? argv[index + 1] : fallback;
  }

  private static exitIfHelp(argv: string[]) {
    if (argv.includes("--help")) {
      console.log(
        "Usage: bun run scripts/drawio/resource-audit.ts [--fixtures DIR] [--resources DIR] [--manifest FILE]",
      );
      process.exit(0);
    }
  }
}

class DrawioResourceAudit {
  constructor(private options: CliParsedOptions) {}

  run() {
    this.validate();
    const available = new DrawioResourceInventory(this.options.resourcesDir).existingLogicalPaths();
    const references = this.fixtureReferences();
    const missing = this.missingReferences(references, available);
    this.printSummary(references.length, missing);
    process.exitCode = missing.length === 0 ? 0 : 1;
  }

  private validate() {
    this.missingRequiredPaths().forEach((requiredPath) => {
      throw new Error(`Required path not found: ${requiredPath}`);
    });
    this.missingManifestPaths().forEach((manifestPath) => {
      throw new Error(`Draw.io resource manifest not found: ${manifestPath}`);
    });
  }

  private missingRequiredPaths(): string[] {
    return [this.options.fixturesDir, this.options.resourcesDir].filter(
      (requiredPath) => !fs.existsSync(requiredPath),
    );
  }

  private missingManifestPaths(): string[] {
    return [this.options.manifestPath].filter((manifestPath) => !fs.existsSync(manifestPath));
  }

  private fixtureReferences(): DrawioResourceReference[] {
    const extractor = new DrawioResourceReferenceExtractor();
    return new DrawioFixtureRepository(this.options.fixturesDir)
      .files()
      .flatMap((fixturePath) => extractor.references(fixturePath));
  }

  private missingReferences(
    references: DrawioResourceReference[],
    available: Set<string>,
  ): DrawioResourceReference[] {
    return references
      .filter((reference) => !this.exists(reference.logicalPath, available))
      .filter(
        (reference, index, all) =>
          all.findIndex((it) => it.logicalPath === reference.logicalPath) === index,
      )
      .sort((left, right) => left.logicalPath.localeCompare(right.logicalPath));
  }

  private exists(logicalPath: string, available: Set<string>): boolean {
    return (
      available.has(logicalPath) ||
      available.has(`${logicalPath}.xml`) ||
      this.hasShapeScript(logicalPath, available) ||
      Array.from(available).some((it) => it.startsWith(`${logicalPath}/`))
    );
  }

  private hasShapeScript(logicalPath: string, available: Set<string>): boolean {
    const group = logicalPath.replace(/^stencils\//, "");
    const scripts = DRAWIO_SHAPE_SCRIPT_GROUPS.get(group) || [];
    return scripts.some(
      (script) =>
        available.has(script) || Array.from(available).some((it) => it.startsWith(`${script}/`)),
    );
  }

  private printSummary(referenceCount: number, missing: DrawioResourceReference[]) {
    console.log(`references: ${referenceCount}`);
    console.log(`missing: ${missing.length}`);
    missing.slice(0, 80).forEach((reference) => {
      console.log(
        `missing ${reference.logicalPath} (${reference.sourceKind}) in ${path.relative(
          process.cwd(),
          reference.fixturePath,
        )}`,
      );
    });
    if (missing.length > 80) {
      console.log(`... ${missing.length - 80} more missing resources`);
    }
  }
}

new DrawioResourceAudit(CliOptions.parse(process.argv.slice(2))).run();

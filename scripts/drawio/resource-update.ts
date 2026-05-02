import fs from "node:fs";
import path from "node:path";
import { DrawioResourceInventory } from "./resource-common";

interface CliParsedOptions {
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
        "Usage: bun run scripts/drawio/resource-update.ts [--resources DIR] [--manifest FILE]",
      );
      process.exit(0);
    }
  }
}

class DrawioResourceUpdate {
  constructor(private options: CliParsedOptions) {}

  run() {
    this.validate();
    const manifest = new DrawioResourceInventory(this.options.resourcesDir).manifest();
    fs.mkdirSync(path.dirname(this.options.manifestPath), { recursive: true });
    fs.writeFileSync(this.options.manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
    console.log(`updated ${path.relative(process.cwd(), this.options.manifestPath)}`);
    console.log(`resources: ${manifest.resources.length}`);
  }

  private validate() {
    if (!fs.existsSync(this.options.resourcesDir)) {
      throw new Error(`Draw.io resources directory not found: ${this.options.resourcesDir}`);
    }
  }
}

new DrawioResourceUpdate(CliOptions.parse(process.argv.slice(2))).run();

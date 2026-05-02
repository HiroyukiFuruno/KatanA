import fs from "node:fs";
import path from "node:path";
import type { Fixture } from "./diagram_update_fixtures";

const REFERENCE_IMAGE_EXTENSIONS = [".png", ".svg"] as const;

export class MarkdownReferenceAssets {
  constructor(
    private sourceDir: string,
    private targetDir: string,
  ) {}

  prepare() {
    fs.mkdirSync(this.targetDir, { recursive: true });
  }

  sync(fixture: Fixture) {
    for (const extension of REFERENCE_IMAGE_EXTENSIONS) {
      fs.copyFileSync(this.sourcePath(fixture, extension), this.targetPath(fixture, extension));
    }
  }

  remove(fixture: Fixture) {
    this.outputPaths(fixture)
      .filter(fs.existsSync)
      .forEach((outputPath) => {
        fs.unlinkSync(outputPath);
      });
  }

  private outputPaths(fixture: Fixture): string[] {
    return [...new Set(this.extensionsFor(fixture).flatMap((it) => [it.source, it.target]))];
  }

  private extensionsFor(fixture: Fixture): ImagePathPair[] {
    return REFERENCE_IMAGE_EXTENSIONS.map((extension) => ({
      source: this.sourcePath(fixture, extension),
      target: this.targetPath(fixture, extension),
    }));
  }

  private sourcePath(fixture: Fixture, extension: string): string {
    return path.join(this.sourceDir, `${fixture.slug}${extension}`);
  }

  private targetPath(fixture: Fixture, extension: string): string {
    return path.join(this.targetDir, `${fixture.slug}${extension}`);
  }
}

interface ImagePathPair {
  source: string;
  target: string;
}

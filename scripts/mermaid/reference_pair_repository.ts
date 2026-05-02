import fs from "node:fs";
import path from "node:path";
import type { CliParsedOptions } from "./reference_compare_options";

export interface ReferencePair {
  slug: string;
  officialPath: string;
  katanaPath: string;
}

export class ReferencePairRepository {
  constructor(private options: CliParsedOptions) {}

  list(): ReferencePair[] {
    return fs
      .readdirSync(this.options.officialDir)
      .filter((fileName) => fileName.endsWith(".png"))
      .sort()
      .map((fileName) => this.buildPair(fileName))
      .filter((pair) => fs.existsSync(pair.katanaPath));
  }

  private buildPair(fileName: string): ReferencePair {
    return {
      slug: path.basename(fileName, ".png"),
      officialPath: path.join(this.options.officialDir, fileName),
      katanaPath: path.join(this.options.katanaDir, fileName),
    };
  }
}

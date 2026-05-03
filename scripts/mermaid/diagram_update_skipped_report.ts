import fs from "node:fs";
import path from "node:path";
import type { Fixture } from "./diagram_update_fixtures";

interface SkippedFixture {
  slug: string;
  fileName: string;
  message: string;
}

export class SkippedFixtureReport {
  private entries: SkippedFixture[] = [];

  constructor(private outputDir: string) {}

  add(fixture: Fixture, message: string) {
    const entry = { slug: fixture.slug, fileName: fixture.fileName, message };
    this.entries.push(entry);
    console.warn(`skipped ${entry.slug}: ${entry.message}`);
  }

  write() {
    fs.writeFileSync(this.outputPath(), `${JSON.stringify({ skipped: this.entries }, null, 2)}\n`);
  }

  private outputPath(): string {
    return path.join(this.outputDir, "render-skipped.json");
  }
}

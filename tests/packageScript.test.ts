import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("package.ps1", () => {
  it("reads the package version from tauri.conf.json", () => {
    const script = readFileSync("package.ps1", "utf8");

    expect(script).toContain("tauri.conf.json");
    expect(script).not.toMatch(/\$Version\s*=\s*"0\.2\.0"/);
  });
});

import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("vite cache busting", () => {
  it("includes the app version in emitted asset names", () => {
    const config = readFileSync("vite.config.ts", "utf8");

    expect(config).toContain("package.json");
    expect(config).toContain("entryFileNames");
    expect(config).toContain("assetFileNames");
    expect(config).toContain("appVersion");
  });
});

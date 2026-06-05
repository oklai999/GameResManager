import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("Tauri WebView data directory", () => {
  it("uses a versioned WebView data directory to avoid stale release UI cache", () => {
    const config = JSON.parse(
      readFileSync(resolve(process.cwd(), "src-tauri/tauri.conf.json"), "utf8")
    );

    expect(config.app.windows[0].dataDirectory).toBe(`webview-v${config.version}`);
  });
});

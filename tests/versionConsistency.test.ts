import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

function extractTomlSection(source: string, section: string): string {
  const normalized = source.replace(/\r\n/g, "\n");
  const match = normalized.match(
    new RegExp(`^\\[${section.replace(".", "\\.")}\\]\\n([\\s\\S]*?)(?=^\\[|\\Z)`, "m")
  );
  if (!match) throw new Error(`missing TOML section: ${section}`);
  return match[1];
}

function extractCargoLockPackage(source: string, packageName: string): string {
  const normalized = source.replace(/\r\n/g, "\n");
  const blocks = normalized.split("[[package]]").slice(1);
  const block = blocks.find((item) =>
    new RegExp(`^\\s*name = "${packageName}"\\s*$`, "m").test(item)
  );
  if (!block) throw new Error(`missing Cargo.lock package: ${packageName}`);
  return block;
}

describe("version consistency", () => {
  it("has the same version across all release metadata files", () => {
    const pkg = JSON.parse(readFileSync(resolve(process.cwd(), "package.json"), "utf8"));
    const lock = JSON.parse(readFileSync(resolve(process.cwd(), "package-lock.json"), "utf8"));
    const tauriConf = JSON.parse(
      readFileSync(resolve(process.cwd(), "src-tauri/tauri.conf.json"), "utf8")
    );
    const cargoToml = readFileSync(resolve(process.cwd(), "src-tauri/Cargo.toml"), "utf8");
    const cargoLock = readFileSync(resolve(process.cwd(), "src-tauri/Cargo.lock"), "utf8");

    const version = pkg.version;

    expect(lock.version).toBe(version);
    expect(lock.packages[""].version).toBe(version);
    expect(tauriConf.version).toBe(version);

    const tomlPackage = extractTomlSection(cargoToml, "package");
    const tomlVersionMatch = tomlPackage.match(/^version\s*=\s*"([^"]+)"$/m);
    expect(tomlVersionMatch).not.toBeNull();
    expect(tomlVersionMatch![1]).toBe(version);

    const lockPackage = extractCargoLockPackage(cargoLock, "game-resource-manager");
    const lockVersionMatch = lockPackage.match(/^version\s*=\s*"([^"]+)"$/m);
    expect(lockVersionMatch).not.toBeNull();
    expect(lockVersionMatch![1]).toBe(version);
  });
});

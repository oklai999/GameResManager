import { describe, expect, it, vi } from "vitest";
import { recordMediaPreviewActivity } from "./App";
import * as tauri from "./api/tauri";
import type { Asset } from "./types/asset";
import type { WorkbenchSection } from "./components/NavigationRail";

vi.mock("./api/tauri", () => ({
  recordRecentAssetAction: vi.fn().mockResolvedValue(undefined),
}));

function makeAsset(id: number): Asset {
  return {
    id,
    library_folder_id: 1,
    absolute_path: `C:/assets/${id}.mp3`,
    file_name: `${id}.mp3`,
    extension: "mp3",
    asset_type: "audio",
    file_size: 1024,
    modified_at: "2026-06-16T00:00:00Z",
    width: null,
    height: null,
    thumbnail_path: null,
    thumbnail_status: "none",
    thumbnail_error: null,
    note: "",
    is_favorite: false,
    is_missing: false,
    created_at: "2026-06-16T00:00:00Z",
    updated_at: "2026-06-16T00:00:00Z",
  };
}

function makeDeferred<T>() {
  let resolve: (value: T) => void = () => {};
  let reject: (reason: unknown) => void = () => {};
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe("recordMediaPreviewActivity", () => {
  it("records preview_media for the asset", async () => {
    const asset = makeAsset(42);
    const loadRecentActivity = vi.fn().mockResolvedValue(undefined);

    await recordMediaPreviewActivity(asset, () => "library" as WorkbenchSection, loadRecentActivity);

    expect(tauri.recordRecentAssetAction).toHaveBeenCalledTimes(1);
    expect(tauri.recordRecentAssetAction).toHaveBeenCalledWith(42, "preview_media");
  });

  it("refreshes recent activity when section is recent at completion", async () => {
    const asset = makeAsset(42);
    const loadRecentActivity = vi.fn().mockResolvedValue(undefined);

    await recordMediaPreviewActivity(asset, () => "recent" as WorkbenchSection, loadRecentActivity);

    expect(loadRecentActivity).toHaveBeenCalledTimes(1);
  });

  it("does not refresh recent activity when section is not recent at completion", async () => {
    const asset = makeAsset(42);
    const loadRecentActivity = vi.fn().mockResolvedValue(undefined);

    await recordMediaPreviewActivity(asset, () => "library" as WorkbenchSection, loadRecentActivity);

    expect(loadRecentActivity).not.toHaveBeenCalled();
  });

  it("logs error and does not throw when recording fails", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(tauri.recordRecentAssetAction).mockRejectedValueOnce(new Error("DB locked"));
    const asset = makeAsset(42);
    const loadRecentActivity = vi.fn().mockResolvedValue(undefined);

    await recordMediaPreviewActivity(asset, () => "recent" as WorkbenchSection, loadRecentActivity);

    expect(consoleSpy).toHaveBeenCalledWith("Failed to record media preview", expect.any(Error));
    expect(loadRecentActivity).not.toHaveBeenCalled();
    consoleSpy.mockRestore();
  });

  it("uses section at completion time, not call time", async () => {
    const { promise, resolve } = makeDeferred<void>();
    vi.mocked(tauri.recordRecentAssetAction).mockImplementationOnce(() => promise);
    const asset = makeAsset(42);
    const loadRecentActivity = vi.fn().mockResolvedValue(undefined);
    let section: WorkbenchSection = "library";

    const activityPromise = recordMediaPreviewActivity(asset, () => section, loadRecentActivity);
    section = "recent";
    resolve(undefined);
    await activityPromise;

    expect(loadRecentActivity).toHaveBeenCalledTimes(1);
  });

  it("does not refresh if section leaves recent before completion", async () => {
    const { promise, resolve } = makeDeferred<void>();
    vi.mocked(tauri.recordRecentAssetAction).mockImplementationOnce(() => promise);
    const asset = makeAsset(42);
    const loadRecentActivity = vi.fn().mockResolvedValue(undefined);
    let section: WorkbenchSection = "recent";

    const activityPromise = recordMediaPreviewActivity(asset, () => section, loadRecentActivity);
    section = "library";
    resolve(undefined);
    await activityPromise;

    expect(loadRecentActivity).not.toHaveBeenCalled();
  });
});

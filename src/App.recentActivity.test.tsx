import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import App from "./App";
import type { Asset, RecentActivityItem, RecentActivityResponse, ScanSettings, Tag } from "./types/asset";

function createAsset(id: number, fileName: string): Asset {
  return {
    id,
    library_folder_id: 1,
    absolute_path: `C:/assets/${fileName}`,
    file_name: fileName,
    extension: fileName.split(".").pop() ?? "",
    asset_type: "audio",
    file_size: 100,
    modified_at: "2026-05-27T00:00:00Z",
    width: null,
    height: null,
    thumbnail_path: null,
    thumbnail_status: "none",
    thumbnail_error: null,
    note: "",
    is_favorite: false,
    is_missing: false,
    created_at: "2026-05-27T00:00:00Z",
    updated_at: "2026-05-27T00:00:00Z",
    tags: [],
  };
}

function createRecentItem(asset: Asset): RecentActivityItem {
  return {
    asset,
    latest_action_type: "open_file",
    latest_action_at: "2026-06-13T10:00:00Z",
    action_count: 1,
    open_file_count: 1,
    reveal_folder_count: 0,
    copy_path_count: 0,
    preview_media_count: 0,
    actions: [{ id: asset.id, asset_id: asset.id, action_type: "open_file", created_at: "2026-06-13T10:00:00Z" }],
  };
}

type Deferred<T> = {
  promise: Promise<T>;
  resolve: (value: T) => void;
  reject: (reason?: unknown) => void;
};

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

const defaultSettings: ScanSettings = {
  id: 1,
  include_images: true,
  include_audio: true,
  include_video: true,
  include_fonts: true,
  include_models: true,
  include_spine: true,
  include_psd: true,
  generate_psd_thumbnails: false,
  ignored_directory_names: "node_modules,.git",
  thumbnail_cache_dir: null,
  database_path: null,
  ignored_extensions: ".tmp",
  updated_at: "2026-06-13T00:00:00Z",
};

vi.mock("./api/tauri", () => ({
  listAssets: vi.fn(async () => []),
  listLibraryFolders: vi.fn(async () => []),
  addLibraryFolder: vi.fn(),
  pickLibraryFolder: vi.fn(async () => null),
  createLibraryFolderFromPath: vi.fn(),
  listAssetTags: vi.fn(async () => []),
  listTags: vi.fn(async () => []),
  listRecentTags: vi.fn(async () => []),
  getAssetTags: vi.fn(async () => []),
  listCommonTags: vi.fn(async () => []),
  updateTag: vi.fn(),
  removeTagFromAssets: vi.fn(),
  deleteTag: vi.fn(),
  setAssetFavorite: vi.fn(),
  applyTagToAssets: vi.fn(),
  openAssetFile: vi.fn(),
  revealAssetInFolder: vi.fn(),
  startScan: vi.fn(),
  cancelScan: vi.fn(),
  latestScanJob: vi.fn(async () => null),
  getScanSettings: vi.fn(async () => defaultSettings),
  saveScanSettings: vi.fn(),
  searchAssets: vi.fn(async () => []),
  searchAssetsPage: vi.fn(async () => ({ assets: [], total_count: 0 })),
  listCollections: vi.fn(async () => []),
  createCollection: vi.fn(),
  addAssetsToCollection: vi.fn(),
  removeAssetFromCollection: vi.fn(),
  updateCollection: vi.fn(),
  removeAssetsFromCollection: vi.fn(),
  deleteCollection: vi.fn(),
  listCollectionAssets: vi.fn(async () => []),
  assetThumbnailUrl: vi.fn(async () => null),
  deleteLibraryFolder: vi.fn(),
  openLibraryFolder: vi.fn(),
  getFolderAssetCounts: vi.fn(async () => ({ folder_id: 1, total: 0, missing: 0, is_accessible: true })),
  updateAssetNote: vi.fn(),
  recordRecentAssetAction: vi.fn(),
  listRecentActivity: vi.fn(async () => ({ items: [], total_count: 0 })),
  assetPathVariants: vi.fn(async () => ({
    absolute_path: "",
    folder_path: "",
    file_name: "",
    forward_slash_path: "",
    godot_res_path: null,
  })),
}));

import * as tauri from "./api/tauri";
import * as discoveryApi from "./api/discovery";

describe("App recent activity race conditions", () => {
  it("resets loadingMore when a stale load-more request is superseded by a filter change", async () => {
    const listRecentActivity = vi.mocked(tauri.listRecentActivity);
    const asset1 = createAsset(1, "a.wav");
    const asset2 = createAsset(2, "b.wav");
    const asset3 = createAsset(3, "c.wav");

    listRecentActivity.mockResolvedValueOnce({
      items: [createRecentItem(asset1)],
      total_count: 3,
      limit: 40,
      offset: 0,
    });

    const loadMoreDeferred = deferred<RecentActivityResponse>();
    const filterChangeDeferred = deferred<RecentActivityResponse>();

    listRecentActivity.mockImplementationOnce(async () => loadMoreDeferred.promise);
    listRecentActivity.mockImplementationOnce(async () => filterChangeDeferred.promise);

    render(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText("最近")).toBeInTheDocument();
    });

    await userEvent.click(screen.getByLabelText("最近"));

    await waitFor(() => {
      expect(screen.getByText("a.wav")).toBeInTheDocument();
    });

    const loadMoreButton = screen.getByRole("button", { name: "加载更多活动" });
    expect(loadMoreButton).toBeEnabled();

    await userEvent.click(loadMoreButton);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "加载更多活动" })).toBeDisabled();
    });

    await userEvent.click(screen.getByRole("button", { name: "今天" }));

    loadMoreDeferred.resolve({
      items: [createRecentItem(asset2)],
      total_count: 3,
      limit: 40,
      offset: 1,
    });

    filterChangeDeferred.resolve({
      items: [createRecentItem(asset2), createRecentItem(asset3)],
      total_count: 3,
      limit: 40,
      offset: 0,
    });

    await waitFor(() => {
      const button = screen.getByRole("button", { name: "加载更多活动" });
      expect(button).toBeEnabled();
      expect(button).toHaveTextContent("加载更多");
    });
  });
});


describe("resource discovery flow", () => {
  async function setupLibrary() {
    vi.clearAllMocks();
    const asset = { ...createAsset(1, "archer.png"), asset_type: "image" as const };
    vi.mocked(tauri.listAssets).mockResolvedValue([asset]);
    vi.mocked(tauri.listLibraryFolders).mockResolvedValue([{ id: 1, name: "素材包", path: "C:/assets", created_at: "", last_scanned_at: "", is_enabled: true }]);
    vi.mocked(tauri.listCollections).mockResolvedValue([{ id: 7, name: "候选", description: "", asset_count: 1 }]);
    vi.mocked(tauri.searchAssetsPage).mockResolvedValue({ assets: [asset], total_count: 1, limit: 200, offset: 0 });
    vi.mocked(tauri.listRecentActivity).mockResolvedValue({ items: [], total_count: 0, limit: 40, offset: 0 });
    vi.mocked(tauri.recordRecentAssetAction).mockResolvedValue(undefined);
    render(<App />);
    await screen.findByText("素材包");
    return asset;
  }

  it("composes library, type, favorite and collection without clearing the query", async () => {
    await setupLibrary();
    await userEvent.click(screen.getByText("素材包"));
    await userEvent.type(screen.getByPlaceholderText("搜索资源..."), "archer");
    await userEvent.click(screen.getByLabelText("类型"));
    await userEvent.click(screen.getByRole("button", { name: "图片" }));
    await userEvent.click(screen.getByLabelText("资源库"));
    await userEvent.click(screen.getByText("收藏", { selector: ".nav-item span" }));
    await userEvent.click(screen.getByLabelText("集合"));
    await userEvent.click(screen.getByText("候选"));
    await waitFor(() => expect(tauri.searchAssetsPage).toHaveBeenLastCalledWith(expect.objectContaining({ query: "archer", library_folder_id: 1, collection_id: 7, asset_type: "image", is_favorite: true })));
    await userEvent.click(screen.getByRole("button", { name: "移除筛选: 仅收藏" }));
    await waitFor(() => expect(tauri.searchAssetsPage).toHaveBeenLastCalledWith(expect.objectContaining({ library_folder_id: 1, collection_id: 7, asset_type: "image", is_favorite: null })));
  });

  it("restores selection and query after searching recent activity", async () => {
    await setupLibrary();
    await userEvent.type(screen.getByPlaceholderText("搜索资源..."), "archer");
    await userEvent.click(screen.getByRole("button", { name: "archer.png" }));
    const calls = vi.mocked(tauri.searchAssetsPage).mock.calls.length;
    await userEvent.click(screen.getByLabelText("最近"));
    await userEvent.type(screen.getByLabelText("搜索最近活动"), "old");
    await waitFor(() => expect(tauri.listRecentActivity).toHaveBeenLastCalledWith(expect.objectContaining({ query: "old", offset: 0 })));
    await userEvent.click(screen.getByRole("button", { name: "返回上次查找" }));
    expect(screen.getByPlaceholderText("搜索资源...")).toHaveValue("archer");
    expect(screen.getByRole("checkbox", { name: "选择 archer.png" })).toBeChecked();
    expect(vi.mocked(tauri.searchAssetsPage).mock.calls.length).toBe(calls);
  });

  it("combines an indexed directory with exact category filters and exclusion", async () => {
    vi.mocked(discoveryApi.listIndexedDirectories).mockResolvedValue([{ path: "C:/assets/Archer", count: 1 }]);
    vi.mocked(discoveryApi.listFacetTags).mockResolvedValue([{ id: 10, name: "角色", color: "#123456", dimension: "usage", asset_count: 1 }]);
    await setupLibrary();
    await userEvent.click(screen.getByText("素材包", { selector: ".folder-name" }));
    await userEvent.click(await screen.findByRole("button", { name: "Archer 1" }));
    await waitFor(() => expect(tauri.searchAssetsPage).toHaveBeenLastCalledWith(expect.objectContaining({ discovery: expect.objectContaining({ directory_path: "C:/assets/Archer", recursive: true }) })));
    await userEvent.click(screen.getByRole("checkbox", { name: "包含子目录" }));
    await userEvent.click(screen.getByLabelText("标签"));
    await userEvent.click(await screen.findByRole("button", { name: "角色 1" }));
    await waitFor(() => expect(tauri.searchAssetsPage).toHaveBeenLastCalledWith(expect.objectContaining({ discovery: expect.objectContaining({ directory_path: "C:/assets/Archer", recursive: false, tag_ids: [10] }) })));
    await userEvent.click(screen.getByRole("button", { name: "排除 角色" }));
    await waitFor(() => expect(tauri.searchAssetsPage).toHaveBeenLastCalledWith(expect.objectContaining({ discovery: expect.objectContaining({ tag_ids: [], excluded_tag_ids: [10] }) })));
    vi.mocked(discoveryApi.listFacetTags).mockResolvedValue([]);
  });

  it("records property copy only after clipboard succeeds", async () => {
    await setupLibrary();
    await userEvent.click(screen.getByRole("button", { name: "archer.png" }));
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText } });
    fireEvent.click(await screen.findByRole("button", { name: "复制文件名" }));
    await waitFor(() => expect(tauri.recordRecentAssetAction).toHaveBeenCalledWith(1, "copy_path"));
    expect(writeText).toHaveBeenCalledWith("archer.png");
    vi.mocked(tauri.recordRecentAssetAction).mockClear();
    writeText.mockRejectedValueOnce(new Error("clipboard denied"));
    fireEvent.click(screen.getByRole("button", { name: "复制正斜杠路径" }));
    await screen.findByText("clipboard denied");
    expect(tauri.recordRecentAssetAction).not.toHaveBeenCalled();
  });
});


vi.mock("./api/discovery", async (importOriginal) => {
  const actual = await importOriginal<typeof import("./api/discovery")>();
  return { ...actual, listFacetTags: vi.fn(async () => []), listIndexedDirectories: vi.fn(async () => []) };
});

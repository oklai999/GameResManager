import { invoke } from "@tauri-apps/api/core";
import type { Asset, AssetPathVariants, AssetSearchRequest, AssetSearchResponse, Collection, FolderAssetCounts, LibraryFolder, RecentAssetAction, ScanJob, ScanSettings, Tag } from "../types/asset";

export async function listAssets(): Promise<Asset[]> {
  return invoke<Asset[]>("list_assets");
}

export async function listLibraryFolders(): Promise<LibraryFolder[]> {
  return invoke<LibraryFolder[]>("list_library_folders");
}

export async function addLibraryFolder(name: string, path: string): Promise<LibraryFolder> {
  return invoke<LibraryFolder>("add_library_folder", { name, path });
}

export async function pickLibraryFolder(): Promise<string | null> {
  return invoke<string | null>("pick_library_folder");
}

export async function createLibraryFolderFromPath(path: string): Promise<LibraryFolder> {
  return invoke<LibraryFolder>("create_library_folder_from_path", { path });
}

export async function listAssetTags(): Promise<[number, string][]> {
  return invoke<[number, string][]>("list_asset_tags");
}

export async function listTags(): Promise<Tag[]> {
  return invoke<Tag[]>("list_tags");
}

export async function listRecentTags(limit: number): Promise<Tag[]> {
  return invoke<Tag[]>("list_recent_tags", { limit });
}

export async function getAssetTags(assetId: number): Promise<string[]> {
  return invoke<string[]>("get_asset_tags", { assetId });
}

export async function listCommonTags(assetIds: number[]): Promise<string[]> {
  return invoke<string[]>("list_common_tags", { assetIds });
}

export async function updateTag(
  tagId: number,
  name: string,
  color: string
): Promise<Tag> {
  return invoke<Tag>("update_tag", { tagId, name, color });
}

export async function removeTagFromAssets(
  tagId: number,
  assetIds: number[]
): Promise<number> {
  return invoke<number>("remove_tag_from_assets", { tagId, assetIds });
}

export async function deleteTag(tagId: number): Promise<boolean> {
  return invoke<boolean>("delete_tag", { tagId });
}

export async function setAssetFavorite(assetId: number, isFavorite: boolean): Promise<void> {
  await invoke("set_asset_favorite", { assetId, isFavorite });
}

export async function applyTagToAssets(tagName: string, assetIds: number[]): Promise<void> {
  await invoke("apply_tag_to_assets", { tagName, assetIds });
}

export async function openAssetFile(path: string): Promise<void> {
  await invoke("open_asset_file", { path });
}

export async function revealAssetInFolder(path: string): Promise<void> {
  await invoke("reveal_asset_in_folder", { path });
}

export async function startScan(folderId: number): Promise<ScanJob> {
  return invoke<ScanJob>("start_scan", { folderId });
}

export async function cancelScan(jobId: number): Promise<void> {
  await invoke("cancel_scan", { jobId });
}

export async function latestScanJob(folderId: number): Promise<ScanJob | null> {
  return invoke<ScanJob | null>("latest_scan_job", { folderId });
}

export async function getScanSettings(): Promise<ScanSettings> {
  return invoke<ScanSettings>("get_scan_settings");
}

export async function saveScanSettings(settings: ScanSettings): Promise<void> {
  await invoke("save_scan_settings", { settings });
}

export async function searchAssets(req: AssetSearchRequest): Promise<Asset[]> {
  return invoke<Asset[]>("search_assets", { req });
}

export async function searchAssetsPage(req: AssetSearchRequest): Promise<AssetSearchResponse> {
  return invoke<AssetSearchResponse>("search_assets_page", { req });
}

export async function listCollections(): Promise<Collection[]> {
  return invoke<Collection[]>("list_collections");
}

export async function createCollection(name: string, description: string): Promise<Collection> {
  return invoke<Collection>("create_collection", { name, description });
}

export async function addAssetsToCollection(collectionId: number, assetIds: number[]): Promise<void> {
  await invoke("add_assets_to_collection", { collectionId, assetIds });
}

export async function removeAssetFromCollection(collectionId: number, assetId: number): Promise<void> {
  await invoke("remove_asset_from_collection", { collectionId, assetId });
}

export async function updateCollection(
  collectionId: number,
  name: string,
  description: string
): Promise<Collection> {
  return invoke<Collection>("update_collection", { collectionId, name, description });
}

export async function removeAssetsFromCollection(
  collectionId: number,
  assetIds: number[]
): Promise<void> {
  await invoke("remove_assets_from_collection", { collectionId, assetIds });
}

export async function deleteCollection(collectionId: number): Promise<boolean> {
  return invoke<boolean>("delete_collection", { collectionId });
}

export async function listCollectionAssets(collectionId: number): Promise<number[]> {
  return invoke<number[]>("list_collection_assets", { collectionId });
}

export async function assetThumbnailUrl(assetId: number): Promise<string | null> {
  return invoke<string | null>("asset_thumbnail_url", { assetId });
}

export async function deleteLibraryFolder(folderId: number): Promise<boolean> {
  return invoke<boolean>("delete_library_folder", { folderId });
}

export async function openLibraryFolder(path: string): Promise<void> {
  await invoke("open_library_folder", { path });
}

export async function getFolderAssetCounts(folderId: number): Promise<FolderAssetCounts> {
  return invoke<FolderAssetCounts>("get_folder_asset_counts", { folderId });
}

export async function updateAssetNote(assetId: number, note: string): Promise<Asset> {
  return invoke<Asset>("update_asset_note", { assetId, note });
}

export async function recordRecentAssetAction(
  assetId: number,
  actionType: RecentAssetAction["action_type"]
): Promise<void> {
  await invoke("record_recent_asset_action", { assetId, actionType });
}

export async function listRecentAssetActions(limit: number): Promise<RecentAssetAction[]> {
  return invoke<RecentAssetAction[]>("list_recent_asset_actions", { limit });
}

export async function assetPathVariants(
  path: string,
  projectRoot: string | null
): Promise<AssetPathVariants> {
  return invoke<AssetPathVariants>("asset_path_variants", { path, projectRoot });
}

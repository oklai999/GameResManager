import { invoke } from "@tauri-apps/api/core";
import type { Asset, LibraryFolder, ScanJob, ScanSettings } from "../types/asset";

export async function listAssets(): Promise<Asset[]> {
  return invoke<Asset[]>("list_assets");
}

export async function listLibraryFolders(): Promise<LibraryFolder[]> {
  return invoke<LibraryFolder[]>("list_library_folders");
}

export async function addLibraryFolder(name: string, path: string): Promise<LibraryFolder> {
  return invoke<LibraryFolder>("add_library_folder", { name, path });
}

export async function listAssetTags(): Promise<[number, string][]> {
  return invoke<[number, string][]>("list_asset_tags");
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

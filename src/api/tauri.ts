import { invoke } from "@tauri-apps/api/core";
import type { Asset, LibraryFolder } from "../types/asset";

export async function listAssets(): Promise<Asset[]> {
  return invoke<Asset[]>("list_assets");
}

export async function listLibraryFolders(): Promise<LibraryFolder[]> {
  return invoke<LibraryFolder[]>("list_library_folders");
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

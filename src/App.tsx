import { useCallback, useEffect, useMemo, useState } from "react";
import {
  addLibraryFolder,
  applyTagToAssets,
  listAssets,
  listAssetTags,
  listLibraryFolders,
  openAssetFile,
  revealAssetInFolder,
  scanLibraryFolder,
  setAssetFavorite,
} from "./api/tauri";
import { AssetGrid } from "./components/AssetGrid";
import { DetailsPanel } from "./components/DetailsPanel";
import { LibrarySidebar } from "./components/LibrarySidebar";
import { SearchToolbar } from "./components/SearchToolbar";
import type { Asset, LibraryFolder, ScanResult, SearchScope } from "./types/asset";

export default function App() {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [folders, setFolders] = useState<LibraryFolder[]>([]);
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [activeFilter, setActiveFilter] = useState("all");
  const [query, setQuery] = useState("");
  const [scope, setScope] = useState<SearchScope>({ fileName: true, tag: true, note: true, path: false });
  const [error, setError] = useState<string | null>(null);
  const [scanResult, setScanResult] = useState<ScanResult | null>(null);

  const loadData = useCallback(async () => {
    try {
      const [assetList, folderList, tagList] = await Promise.all([
        listAssets(),
        listLibraryFolders(),
        listAssetTags(),
      ]);
      const tagMap = new Map<number, string[]>();
      for (const [assetId, tagName] of tagList) {
        const arr = tagMap.get(assetId) ?? [];
        arr.push(tagName);
        tagMap.set(assetId, arr);
      }
      const assetsWithTags = assetList.map((a) => ({
        ...a,
        tags: tagMap.get(a.id) ?? [],
      }));
      setAssets(assetsWithTags);
      setFolders(folderList);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const filteredAssets = useMemo(() => {
    return assets.filter((asset) => {
      if (activeFilter === "favorites" && !asset.is_favorite) return false;
      if (activeFilter === "missing" && !asset.is_missing) return false;
      if (!["all", "favorites", "missing"].includes(activeFilter) && asset.asset_type !== activeFilter) return false;
      const normalizedQuery = query.trim().toLowerCase();
      if (!normalizedQuery) return true;
      const haystacks = [
        scope.fileName ? asset.file_name : "",
        scope.note ? asset.note : "",
        scope.path ? asset.absolute_path : "",
        scope.tag ? (asset.tags ?? []).join(" ") : "",
      ];
      return haystacks.some((value) => value.toLowerCase().includes(normalizedQuery));
    });
  }, [activeFilter, assets, query, scope]);

  const selectedAssets = assets.filter((asset) => selectedIds.includes(asset.id));

  const handleAddFolder = useCallback(async (name: string, path: string) => {
    if (!name.trim() || !path.trim()) {
      setError("名称和路径不能为空");
      return;
    }
    try {
      await addLibraryFolder(name, path);
      await loadData();
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [loadData]);

  const handleScanFolder = useCallback(async (folderId: number) => {
    try {
      const result = await scanLibraryFolder(folderId);
      setScanResult(result);
      await loadData();
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [loadData]);

  const handleToggleFavorite = useCallback(async (asset: Asset) => {
    try {
      await setAssetFavorite(asset.id, !asset.is_favorite);
      setAssets((prev) =>
        prev.map((a) => (a.id === asset.id ? { ...a, is_favorite: !a.is_favorite } : a))
      );
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const handleApplyTag = useCallback(async (tagName: string, assetIds: number[]) => {
    try {
      await applyTagToAssets(tagName, assetIds);
      await loadData();
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [loadData]);

  return (
    <main className="app-shell">
      <LibrarySidebar
        folders={folders}
        activeFilter={activeFilter}
        onFilterChange={setActiveFilter}
        onAddFolder={handleAddFolder}
        onScanFolder={handleScanFolder}
        error={error}
      />
      <section className="workspace">
        <SearchToolbar query={query} scope={scope} onQueryChange={setQuery} onScopeChange={setScope} />
        {scanResult && (
          <div className="scan-result">
            扫描完成：发现 {scanResult.found}，新增 {scanResult.added}，更新 {scanResult.updated}，跳过 {scanResult.skipped}，缺失 {scanResult.missing}
          </div>
        )}
        <AssetGrid
          assets={filteredAssets}
          selectedIds={selectedIds}
          onSelectionChange={setSelectedIds}
          onToggleFavorite={handleToggleFavorite}
        />
      </section>
      <DetailsPanel
        selectedAssets={selectedAssets}
        onOpenFile={(asset) => openAssetFile(asset.absolute_path)}
        onReveal={(asset) => revealAssetInFolder(asset.absolute_path)}
        onCopyPath={(asset) => navigator.clipboard.writeText(asset.absolute_path)}
        onApplyTag={handleApplyTag}
        onToggleFavorite={handleToggleFavorite}
      />
    </main>
  );
}

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  addLibraryFolder,
  applyTagToAssets,
  cancelScan,
  getScanSettings,
  latestScanJob,
  listAssets,
  listAssetTags,
  listLibraryFolders,
  openAssetFile,
  revealAssetInFolder,
  saveScanSettings,
  setAssetFavorite,
  startScan,
} from "./api/tauri";
import { AssetGrid } from "./components/AssetGrid";
import { DetailsPanel } from "./components/DetailsPanel";
import { LibrarySidebar } from "./components/LibrarySidebar";
import { ScanSettingsPanel } from "./components/ScanSettingsPanel";
import { ScanStatusBar } from "./components/ScanStatusBar";
import { SearchToolbar } from "./components/SearchToolbar";
import type { Asset, LibraryFolder, ScanJob, ScanSettings, SearchScope } from "./types/asset";

export default function App() {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [folders, setFolders] = useState<LibraryFolder[]>([]);
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [activeFilter, setActiveFilter] = useState("all");
  const [query, setQuery] = useState("");
  const [scope, setScope] = useState<SearchScope>({ fileName: true, tag: true, note: true, path: false });
  const [error, setError] = useState<string | null>(null);
  const [latestJobs, setLatestJobs] = useState<Record<number, ScanJob | null>>({});
  const [scanSettings, setScanSettings] = useState<ScanSettings | null>(null);

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

      const jobs: Record<number, ScanJob | null> = {};
      await Promise.all(
        folderList.map(async (folder) => {
          jobs[folder.id] = await latestScanJob(folder.id);
        })
      );
      setLatestJobs(jobs);

      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  useEffect(() => {
    const hasRunning = Object.values(latestJobs).some((j) => j?.status === "running");
    if (!hasRunning) return;
    const interval = setInterval(() => {
      loadData();
    }, 1000);
    return () => clearInterval(interval);
  }, [latestJobs, loadData]);

  useEffect(() => {
    getScanSettings()
      .then(setScanSettings)
      .catch((e) => setError(String(e)));
  }, []);

  const handleScanSettingsChange = useCallback(async (settings: ScanSettings) => {
    setScanSettings(settings);
    try {
      await saveScanSettings(settings);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

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
      const job = await startScan(folderId);
      setLatestJobs((prev) => ({ ...prev, [folderId]: job }));
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const handleCancelScan = useCallback(async (jobId: number) => {
    try {
      await cancelScan(jobId);
      setLatestJobs((prev) => {
        const next = { ...prev };
        for (const [folderId, job] of Object.entries(next)) {
          if (job?.id === jobId) {
            next[Number(folderId)] = { ...job, status: "cancelled" as const };
          }
        }
        return next;
      });
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

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
        onCancelScan={handleCancelScan}
        latestJobs={latestJobs}
        error={error}
        settingsPanel={
          scanSettings ? (
            <ScanSettingsPanel settings={scanSettings} onChange={handleScanSettingsChange} />
          ) : null
        }
      />
      <section className="workspace">
        <SearchToolbar query={query} scope={scope} onQueryChange={setQuery} onScopeChange={setScope} />
        {Object.values(latestJobs)
          .filter(Boolean)
          .map((job) => (
            <ScanStatusBar key={job!.id} job={job!} />
          ))}
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

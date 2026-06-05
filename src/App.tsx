import { useCallback, useEffect, useRef, useState } from "react";
import {
  addAssetsToCollection,
  applyTagToAssets,
  cancelScan,
  createCollection,
  createLibraryFolderFromPath,
  deleteLibraryFolder,
  getFolderAssetCounts,
  getScanSettings,
  latestScanJob,
  listAssets,
  listAssetTags,
  listCollections,
  listLibraryFolders,
  listRecentAssetActions,
  openAssetFile,
  openLibraryFolder,
  pickLibraryFolder,
  recordRecentAssetAction,
  revealAssetInFolder,
  saveScanSettings,
  searchAssets,
  setAssetFavorite,
  startScan,
  updateAssetNote,
} from "./api/tauri";
import { AssetGrid } from "./components/AssetGrid";
import { DetailsPanel } from "./components/DetailsPanel";
import { EmptyState } from "./components/EmptyState";
import { LibrarySidebar } from "./components/LibrarySidebar";
import { SettingsPanel } from "./components/SettingsPanel";
import { ScanStatusBar } from "./components/ScanStatusBar";
import { SearchToolbar } from "./components/SearchToolbar";
import { ToastProvider, useToast } from "./components/ToastHost";
import type { Asset, AssetSearchRequest, Collection, FolderAssetCounts, LibraryFolder, ScanJob, ScanSettings, SearchScope } from "./types/asset";

const SEARCH_RESULT_LIMIT = 2000;

async function fetchLatestJobs(folderList: LibraryFolder[]) {
  const jobs: Record<number, ScanJob | null> = {};
  await Promise.all(
    folderList.map(async (folder) => {
      jobs[folder.id] = await latestScanJob(folder.id);
    })
  );
  return jobs;
}

function AppInner() {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [folders, setFolders] = useState<LibraryFolder[]>([]);
  const [collections, setCollections] = useState<Collection[]>([]);
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [activeFilter, setActiveFilter] = useState("all");
  const [selectedFolderId, setSelectedFolderId] = useState<number | null>(null);
  const [selectedCollectionId, setSelectedCollectionId] = useState<number | null>(null);
  const [query, setQuery] = useState("");
  const [scope, setScope] = useState<SearchScope>({ fileName: true, tag: true, note: true, path: false });
  const [latestJobs, setLatestJobs] = useState<Record<number, ScanJob | null>>({});
  const [scanSettings, setScanSettings] = useState<ScanSettings | null>(null);
  const { showToast } = useToast();
  const [isScanning, setIsScanning] = useState(false);
  const [scanMessage, setScanMessage] = useState<string | null>(null);
  const [activeJobId, setActiveJobId] = useState<number | null>(null);
  const [gridAssets, setGridAssets] = useState<Asset[]>([]);
  const [folderCounts, setFolderCounts] = useState<Record<number, FolderAssetCounts>>({});
  const [recentAssetIds, setRecentAssetIds] = useState<number[]>([]);
  const searchVersionRef = useRef(0);

  const showError = useCallback((e: unknown) => {
    showToast((e as any)?.message ?? String(e), "error");
  }, [showToast]);

  const loadData = useCallback(async () => {
    try {
      const [assetList, folderList, tagList, collectionList] = await Promise.all([
        listAssets(),
        listLibraryFolders(),
        listAssetTags(),
        listCollections(),
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
      setCollections(collectionList);

      try {
        const actions = await listRecentAssetActions(100);
        setRecentAssetIds([...new Set(actions.map((action) => action.asset_id))]);
      } catch {
        // non-blocking: recent activity is optional
      }

      setLatestJobs(await fetchLatestJobs(folderList));

      const counts: Record<number, FolderAssetCounts> = {};
      await Promise.all(
        folderList.map(async (f) => {
          try {
            counts[f.id] = await getFolderAssetCounts(f.id);
          } catch {
            counts[f.id] = { folder_id: f.id, total: 0, missing: 0, is_accessible: false };
          }
        })
      );
      setFolderCounts(counts);
    } catch (e) {
      showError(e);
    }
  }, [showError]);

  const refreshScanJobs = useCallback(async () => {
    try {
      setLatestJobs(await fetchLatestJobs(folders));
    } catch (e) {
      showError(e);
    }
  }, [folders, showError]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  useEffect(() => {
    const hasRunning = Object.values(latestJobs).some((j) => j?.status === "running");
    setIsScanning(hasRunning);

    if (!hasRunning && activeJobId != null) {
      const completedJob = Object.values(latestJobs).find((j) => j?.id === activeJobId);
      if (completedJob) {
        const msg =
          completedJob.status === "completed"
            ? `扫描完成：发现 ${completedJob.found_count} 个，新增 ${completedJob.added_count}，更新 ${completedJob.updated_count}，未变化 ${completedJob.unchanged_count}，缺失 ${completedJob.missing_count}，跳过 ${completedJob.skipped_count}`
            : completedJob.status === "cancelled"
              ? "扫描已取消"
              : `扫描失败：${completedJob.error_message ?? "未知错误"}`;
        setScanMessage(msg);
        setSelectedFolderId(null);
        setSelectedIds([]);
        setActiveJobId(null);
        loadData();
      }
      return;
    }

    if (hasRunning) {
      const interval = setInterval(() => {
        refreshScanJobs();
      }, 1000);
      return () => clearInterval(interval);
    }
  }, [latestJobs, loadData, refreshScanJobs, activeJobId]);

  useEffect(() => {
    getScanSettings()
      .then(setScanSettings)
      .catch(showError);
  }, [showError]);

  const handleScanSettingsChange = useCallback(async (settings: ScanSettings) => {
    setScanSettings(settings);
    try {
      await saveScanSettings(settings);
    } catch (e) {
      showError(e);
    }
  }, [showError]);

  const executeSearch = useCallback(async () => {
    const version = ++searchVersionRef.current;
    try {
      const normalizedQuery = query.trim();
      const req: AssetSearchRequest = {
        query: normalizedQuery,
        search_file_name: scope.fileName,
        search_note: scope.note,
        search_path: scope.path,
        search_tags: scope.tag,
        asset_type: ["all", "favorites", "missing", "recent"].includes(activeFilter) ? null : activeFilter,
        library_folder_id: selectedFolderId,
        collection_id: selectedCollectionId,
        is_favorite: activeFilter === "favorites" ? true : null,
        is_missing: activeFilter === "missing" ? true : null,
        limit: SEARCH_RESULT_LIMIT,
        offset: 0,
      };
      let results = await searchAssets(req);
      if (version !== searchVersionRef.current) return;

      const tagMap = new Map<number, string[]>();
      for (const asset of assets) {
        if (asset.tags && asset.tags.length > 0) {
          tagMap.set(asset.id, asset.tags);
        }
      }
      results = results.map((a) => ({
        ...a,
        tags: tagMap.get(a.id) ?? [],
      }));
      if (version !== searchVersionRef.current) return;

      setGridAssets(results);
    } catch (e) {
      if (version !== searchVersionRef.current) return;
      showError(e);
    }
  }, [query, scope, activeFilter, selectedFolderId, selectedCollectionId, assets, showError]);

  useEffect(() => {
    executeSearch();
  }, [executeSearch]);

  const displayAssets = activeFilter === "recent"
    ? gridAssets.filter((asset) => recentAssetIds.includes(asset.id))
    : gridAssets;

  const selectedAssets = selectedIds
    .map((id) => {
      const fromGrid = gridAssets.find((a) => a.id === id);
      if (fromGrid) return fromGrid;
      return assets.find((a) => a.id === id);
    })
    .filter(Boolean) as Asset[];

  const handlePickFolder = useCallback(async () => {
    try {
      const path = await pickLibraryFolder();
      if (!path) return;
      await createLibraryFolderFromPath(path);
      await loadData();
    } catch (e) {
      showError(e);
    }
  }, [loadData, showError]);

  const handleScanFolder = useCallback(async (folderId: number) => {
    try {
      setScanMessage(null);
      setActiveJobId(null);
      const job = await startScan(folderId);
      setActiveJobId(job.id);
      setLatestJobs((prev) => ({ ...prev, [folderId]: job }));
    } catch (e) {
      showError(e);
    }
  }, [showError]);

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
      setScanMessage("扫描已取消");
    } catch (e) {
      showError(e);
    }
  }, [showError]);

  const handleToggleFavorite = useCallback(async (asset: Asset) => {
    try {
      await setAssetFavorite(asset.id, !asset.is_favorite);
      const toggle = (a: Asset) => a.id === asset.id ? { ...a, is_favorite: !a.is_favorite } : a;
      setAssets((prev) => prev.map(toggle));
      setGridAssets((prev) => prev.map(toggle));
    } catch (e) {
      showError(e);
    }
  }, [showError]);

  const handleApplyTag = useCallback(async (tagName: string, assetIds: number[]) => {
    try {
      await applyTagToAssets(tagName, assetIds);
      await loadData();
    } catch (e) {
      showError(e);
    }
  }, [loadData, showError]);

  const handleCreateCollection = useCallback(async (name: string) => {
    try {
      const col = await createCollection(name, "");
      setCollections((prev) => [...prev, col].sort((a, b) => a.name.localeCompare(b.name)));
    } catch (e) {
      showError(e);
    }
  }, [showError]);

  const handleAddToCollection = useCallback(async (collectionId: number, assetIds: number[]) => {
    try {
      await addAssetsToCollection(collectionId, assetIds);
      await executeSearch();
    } catch (e) {
      showError(e);
    }
  }, [executeSearch, showError]);

  const handleUpdateNote = useCallback((updated: Asset) => {
    setAssets((prev) => prev.map((a) => (a.id === updated.id ? { ...a, note: updated.note } : a)));
    setGridAssets((prev) => prev.map((a) => (a.id === updated.id ? { ...a, note: updated.note } : a)));
  }, []);

  const handleOpenFile = useCallback(async (asset: Asset) => {
    try {
      await openAssetFile(asset.absolute_path);
      try {
        await recordRecentAssetAction(asset.id, "open_file");
        setRecentAssetIds((prev) => [asset.id, ...prev.filter((id) => id !== asset.id)].slice(0, 100));
      } catch (e) {
        console.error("Failed to record recent action", e);
      }
    } catch (e) {
      showError(e);
    }
  }, [showError]);

  const handleRevealFile = useCallback(async (asset: Asset) => {
    try {
      await revealAssetInFolder(asset.absolute_path);
      try {
        await recordRecentAssetAction(asset.id, "reveal_folder");
        setRecentAssetIds((prev) => [asset.id, ...prev.filter((id) => id !== asset.id)].slice(0, 100));
      } catch (e) {
        console.error("Failed to record recent action", e);
      }
    } catch (e) {
      showError(e);
    }
  }, [showError]);

  const handleDeleteFolder = useCallback(async (folderId: number) => {
    try {
      await deleteLibraryFolder(folderId);
      if (selectedFolderId === folderId) {
        setSelectedFolderId(null);
        setSelectedIds([]);
      }
      await loadData();
    } catch (e) {
      showError(e);
    }
  }, [loadData, selectedFolderId, showError]);

  const handleOpenFolder = useCallback(async (folder: LibraryFolder) => {
    try {
      await openLibraryFolder(folder.path);
    } catch (e) {
      showError(e);
    }
  }, [showError]);

  const handleCopyPath = useCallback(async (asset: Asset) => {
    try {
      await navigator.clipboard.writeText(asset.absolute_path);
      showToast("路径已复制", "success");
      try {
        await recordRecentAssetAction(asset.id, "copy_path");
        setRecentAssetIds((prev) => [asset.id, ...prev.filter((id) => id !== asset.id)].slice(0, 100));
      } catch (e) {
        console.error("Failed to record recent action", e);
      }
    } catch (e) {
      showToast((e as any)?.message ?? "复制路径失败", "error");
    }
  }, [showToast]);

  const isEmptySearch = displayAssets.length === 0;
  const hasFolders = folders.length > 0;
  const hasScanned = hasFolders && assets.length > 0;
  const showSearchEmpty = query.trim() !== "" || activeFilter !== "all" || selectedFolderId != null || selectedCollectionId != null;

  return (
    <main className="app-shell">
      <header className="app-topbar">
        <div className="app-brand">
          <div className="app-mark" aria-hidden="true">GR</div>
          <div>
            <div className="app-title">游戏资源管理器</div>
            <div className="app-subtitle">本地素材索引 · 不接管原文件结构</div>
          </div>
        </div>
        <div className="app-topbar-stats" aria-label="资源统计">
          <span className="app-build-marker">新版界面 0.2.1</span>
          <span>{folders.length} 个资源库</span>
          <span>{assets.length} 个资源</span>
          <span>{selectedIds.length} 个已选</span>
        </div>
      </header>
      <div className="workbench-shell">
        <LibrarySidebar
          folders={folders}
          collections={collections}
          activeFilter={activeFilter}
          selectedFolderId={selectedFolderId}
          selectedCollectionId={selectedCollectionId}
          onFilterChange={(f) => { setActiveFilter(f); setSelectedFolderId(null); setSelectedCollectionId(null); setSelectedIds([]); }}
          onSelectFolder={(id) => { setSelectedFolderId(id); setSelectedCollectionId(null); setSelectedIds([]); }}
          onSelectCollection={(id) => { setSelectedCollectionId(id); setSelectedFolderId(null); setSelectedIds([]); }}
          onPickFolder={handlePickFolder}
          onScanFolder={handleScanFolder}
          onCancelScan={handleCancelScan}
          onDeleteFolder={handleDeleteFolder}
          onOpenFolder={handleOpenFolder}
          onCreateCollection={handleCreateCollection}
          isScanning={isScanning}
          latestJobs={latestJobs}
          folderCounts={folderCounts}
          settingsPanel={
            scanSettings ? (
              <SettingsPanel settings={scanSettings} onChange={handleScanSettingsChange} />
            ) : null
          }
        />
        <section className="workspace">
          <SearchToolbar query={query} scope={scope} onQueryChange={(q) => { setQuery(q); setSelectedIds([]); }} onScopeChange={(s) => { setScope(s); setSelectedIds([]); }} />
          <div className="workspace-meta">
            <span>{displayAssets.length} 个结果</span>
            <span>网格视图</span>
          </div>
          {scanMessage && (
            <div className="scan-summary" onClick={() => setScanMessage(null)}>
              {scanMessage}
            </div>
          )}
          {Object.values(latestJobs)
            .filter(Boolean)
            .map((job) => (
              <ScanStatusBar key={job!.id} job={job!} />
            ))}
          {!hasFolders ? (
            <EmptyState variant="no-folders" />
          ) : isEmptySearch && hasScanned ? (
            <EmptyState variant="no-results" />
          ) : !hasScanned ? (
            <EmptyState variant="no-assets" />
          ) : (
            <AssetGrid
              assets={displayAssets}
              selectedIds={selectedIds}
              onSelectionChange={setSelectedIds}
              onToggleFavorite={handleToggleFavorite}
            />
          )}
        </section>
        <DetailsPanel
          selectedAssets={selectedAssets}
          collections={collections}
          onOpenFile={handleOpenFile}
          onReveal={handleRevealFile}
          onCopyPath={handleCopyPath}
          onApplyTag={handleApplyTag}
          onToggleFavorite={handleToggleFavorite}
          onAddToCollection={handleAddToCollection}
          onUpdateNote={handleUpdateNote}
        />
      </div>
    </main>
  );
}

export default function App() {
  return (
    <ToastProvider>
      <AppInner />
    </ToastProvider>
  );
}

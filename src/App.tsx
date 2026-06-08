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
  listRecentTags,
  openAssetFile,
  openLibraryFolder,
  pickLibraryFolder,
  recordRecentAssetAction,
  revealAssetInFolder,
  saveScanSettings,
  searchAssetsPage,
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
import type { Asset, AssetSearchFilters, AssetSearchRequest, AssetSearchSort, Collection, FolderAssetCounts, LibraryFolder, ScanJob, ScanSettings, SearchScope } from "./types/asset";

const SEARCH_PAGE_SIZE = 200;

const DEFAULT_SEARCH_FILTERS: AssetSearchFilters = {
  min_file_size: null,
  max_file_size: null,
  min_width: null,
  max_width: null,
  min_height: null,
  max_height: null,
  modified_after: null,
  modified_before: null,
};

const DEFAULT_SEARCH_SORT: AssetSearchSort = {
  sort_by: "file_name",
  sort_direction: "asc",
};

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
  const [filters, setFilters] = useState<AssetSearchFilters>(DEFAULT_SEARCH_FILTERS);
  const [sort, setSort] = useState<AssetSearchSort>(DEFAULT_SEARCH_SORT);
  const [latestJobs, setLatestJobs] = useState<Record<number, ScanJob | null>>({});
  const [scanSettings, setScanSettings] = useState<ScanSettings | null>(null);
  const { showToast } = useToast();
  const [isScanning, setIsScanning] = useState(false);
  const [scanMessage, setScanMessage] = useState<string | null>(null);
  const [activeJobId, setActiveJobId] = useState<number | null>(null);
  const [gridAssets, setGridAssets] = useState<Asset[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [isSearching, setIsSearching] = useState(false);
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const [folderCounts, setFolderCounts] = useState<Record<number, FolderAssetCounts>>({});
  const [recentAssetIds, setRecentAssetIds] = useState<number[]>([]);
  const [recentTags, setRecentTags] = useState<string[]>([]);
  const [projectRoot, setProjectRoot] = useState("");
  const searchVersionRef = useRef(0);

  const showError = useCallback((e: unknown) => {
    showToast((e as any)?.message ?? String(e), "error");
  }, [showToast]);

  const loadRecentTags = useCallback(async () => {
    try {
      const tags = await listRecentTags(12);
      setRecentTags(tags.map((tag) => tag.name));
    } catch {
      setRecentTags([]);
    }
  }, []);

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

      await loadRecentTags();
    } catch (e) {
      showError(e);
    }
  }, [showError, loadRecentTags]);

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

  const handleFiltersChange = useCallback((next: AssetSearchFilters) => {
    setFilters(next);
    setSelectedIds([]);
  }, []);

  const handleSortChange = useCallback((next: AssetSearchSort) => {
    setSort(next);
    setSelectedIds([]);
  }, []);

  const buildSearchRequest = useCallback((offset: number): AssetSearchRequest => {
    const normalizedQuery = query.trim();
    return {
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
      min_file_size: filters.min_file_size,
      max_file_size: filters.max_file_size,
      min_width: filters.min_width,
      max_width: filters.max_width,
      min_height: filters.min_height,
      max_height: filters.max_height,
      modified_after: filters.modified_after,
      modified_before: filters.modified_before,
      sort_by: sort.sort_by,
      sort_direction: sort.sort_direction,
      limit: SEARCH_PAGE_SIZE,
      offset,
    };
  }, [query, scope, activeFilter, selectedFolderId, selectedCollectionId, filters, sort]);

  const executeSearch = useCallback(async () => {
    const version = ++searchVersionRef.current;
    setIsSearching(true);
    setIsLoadingMore(false);
    try {
      const req = buildSearchRequest(0);
      const page = await searchAssetsPage(req);
      if (version !== searchVersionRef.current) return;

      const tagMap = new Map<number, string[]>();
      for (const asset of assets) {
        if (asset.tags && asset.tags.length > 0) {
          tagMap.set(asset.id, asset.tags);
        }
      }
      const results = page.assets.map((a) => ({
        ...a,
        tags: tagMap.get(a.id) ?? [],
      }));
      if (version !== searchVersionRef.current) return;

      setGridAssets(results);
      setTotalCount(page.total_count);
    } catch (e) {
      if (version !== searchVersionRef.current) return;
      showError(e);
    } finally {
      if (version === searchVersionRef.current) {
        setIsSearching(false);
      }
    }
  }, [buildSearchRequest, assets, showError]);

  const handleLoadMore = useCallback(async () => {
    if (isSearching || isLoadingMore || gridAssets.length >= totalCount) return;
    const version = searchVersionRef.current;
    setIsLoadingMore(true);
    try {
      const req = buildSearchRequest(gridAssets.length);
      const page = await searchAssetsPage(req);
      if (version !== searchVersionRef.current) return;

      const tagMap = new Map<number, string[]>();
      for (const asset of assets) {
        if (asset.tags && asset.tags.length > 0) {
          tagMap.set(asset.id, asset.tags);
        }
      }
      const nextAssets = page.assets.map((a) => ({
        ...a,
        tags: tagMap.get(a.id) ?? [],
      }));
      setGridAssets((prev) => [...prev, ...nextAssets]);
      setTotalCount(page.total_count);
    } catch (e) {
      if (version !== searchVersionRef.current) return;
      showError(e);
    } finally {
      if (version === searchVersionRef.current) {
        setIsLoadingMore(false);
      }
    }
  }, [isSearching, isLoadingMore, gridAssets.length, totalCount, buildSearchRequest, assets, showError]);

  useEffect(() => {
    executeSearch();
  }, [executeSearch]);

  const displayAssets = activeFilter === "recent"
    ? recentAssetIds
        .map((id) => assets.find((a) => a.id === id))
        .filter(Boolean) as Asset[]
    : gridAssets;
  const displayedTotalCount = activeFilter === "recent" ? displayAssets.length : totalCount;
  const canLoadMore = activeFilter !== "recent" && displayAssets.length < totalCount;

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

  const handleCopyText = useCallback(async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      showToast("已复制", "success");
    } catch (e) {
      showToast((e as any)?.message ?? "复制失败", "error");
    }
  }, [showToast]);

  const isEmptySearch = displayAssets.length === 0;
  const hasFolders = folders.length > 0;
  const hasScanned = hasFolders && assets.length > 0;
  const hasAdvancedFilters = Object.values(filters).some((value) => value != null);
  const showSearchEmpty =
    query.trim() !== "" ||
    activeFilter !== "all" ||
    selectedFolderId != null ||
    selectedCollectionId != null ||
    hasAdvancedFilters;

  return (
    <main className="app-shell">
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
        <SearchToolbar
          query={query}
          scope={scope}
          filters={filters}
          sort={sort}
          onQueryChange={(q) => { setQuery(q); setSelectedIds([]); }}
          onScopeChange={(s) => { setScope(s); setSelectedIds([]); }}
          onFiltersChange={handleFiltersChange}
          onSortChange={handleSortChange}
        />
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
        ) : isEmptySearch && (hasScanned || showSearchEmpty) ? (
          <EmptyState variant="no-results" />
        ) : !hasScanned ? (
          <EmptyState variant="no-assets" />
        ) : (
          <>
            <AssetGrid
              assets={displayAssets}
              selectedIds={selectedIds}
              onSelectionChange={setSelectedIds}
              onToggleFavorite={handleToggleFavorite}
            />
            {displayAssets.length > 0 && (
              <div className="result-footer">
                <span>已显示 {displayAssets.length} / {displayedTotalCount}</span>
                {canLoadMore && (
                  <button
                    type="button"
                    onClick={handleLoadMore}
                    disabled={isSearching || isLoadingMore}
                  >
                    {isLoadingMore ? "加载中..." : "加载更多"}
                  </button>
                )}
              </div>
            )}
          </>
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
        recentTags={recentTags}
        onCopyText={handleCopyText}
        projectRoot={projectRoot}
        onProjectRootChange={setProjectRoot}
      />
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

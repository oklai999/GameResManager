import { CategoryFilters } from "./components/CategoryFilters";
import { ClassificationRules } from "./components/ClassificationRules";
import { DirectoryTree } from "./components/DirectoryTree";
import { defaultDiscovery, listFacetTags, type FacetTag } from "./api/discovery";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  addAssetsToCollection,
  applyTagToAssets,
  cancelScan,
  createCollection,
  createLibraryFolderFromPath,
  deleteCollection,
  deleteLibraryFolder,
  deleteTag,
  getFolderAssetCounts,
  getScanSettings,
  latestScanJob,
  listAssets,
  listAssetTags,
  listCollections,
  listLibraryFolders,
  listRecentActivity,
  listRecentTags,
  listTags,
  openAssetFile,
  openLibraryFolder,
  pickLibraryFolder,
  recordRecentAssetAction,
  removeAssetsFromCollection,
  removeTagFromAssets,
  revealAssetInFolder,
  saveScanSettings,
  searchAssetsPage,
  setAssetFavorite,
  startScan,
  updateAssetNote,
  updateCollection,
  updateTag,
} from "./api/tauri";
import { ImageQuickLook } from "./components/ImageQuickLook";
import { AssetGrid } from "./components/AssetGrid";
import { DetailsPanel } from "./components/DetailsPanel";
import { EmptyState } from "./components/EmptyState";
import { LibrarySidebar } from "./components/LibrarySidebar";
import { NavigationRail } from "./components/NavigationRail";
import type { WorkbenchSection } from "./components/NavigationRail";
import { RecentActivityTimeline } from "./components/RecentActivityTimeline";
import { SettingsPanel } from "./components/SettingsPanel";
import { ScanStatusBar } from "./components/ScanStatusBar";
import { ActiveFilterChips } from "./components/ActiveFilterChips";
import { FilterPanel } from "./components/FilterPanel";
import { SearchToolbar } from "./components/SearchToolbar";
import { ToastProvider, useToast } from "./components/ToastHost";
import type { Asset, AssetSearchFilters, AssetSearchRequest, AssetSearchSort, Collection, FolderAssetCounts, LibraryFolder, RecentActionType, RecentActivityItem, RecentActivityPeriod, ScanJob, ScanSettings, SearchScope, Tag } from "./types/asset";

const SEARCH_PAGE_SIZE = 200;
const RECENT_PAGE_SIZE = 40;

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

export async function recordMediaPreviewActivity(
  asset: Asset,
  getActiveSection: () => WorkbenchSection,
  loadRecentActivity: () => Promise<void>
): Promise<void> {
  try {
    await recordRecentAssetAction(asset.id, "preview_media");
    if (getActiveSection() === "recent") await loadRecentActivity();
  } catch (error) {
    console.error("Failed to record media preview", error);
  }
}

function AppInner() {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [folders, setFolders] = useState<LibraryFolder[]>([]);
  const [collections, setCollections] = useState<Collection[]>([]);
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [discovery, setDiscovery] = useState(defaultDiscovery);
  const [facetTags, setFacetTags] = useState<FacetTag[]>([]);
  const [facetError, setFacetError] = useState("");
  const [rulesOpen, setRulesOpen] = useState(false);
  const [favoriteOnly, setFavoriteOnly] = useState(false);
  const [missingOnly, setMissingOnly] = useState(false);
  const [dataVersion, setDataVersion] = useState(0);
  const [searchError, setSearchError] = useState(false);
  const [recentQuery, setRecentQuery] = useState("");
  const gridScrollRef = useRef(0);
  const librarySelectionRef = useRef<number[]>([]);
  const previousSectionRef = useRef<WorkbenchSection>("library");
  const [quickLook, setQuickLook] = useState<{ assets: Asset[]; initialId: number } | null>(null);
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
  const [recentPeriod, setRecentPeriod] = useState<RecentActivityPeriod>("all");
  const [recentActionType, setRecentActionType] = useState<"all" | RecentActionType>("all");
  const [recentItems, setRecentItems] = useState<RecentActivityItem[]>([]);
  const [recentTotalCount, setRecentTotalCount] = useState(0);
  const [recentLoading, setRecentLoading] = useState(false);
  const [recentLoadingMore, setRecentLoadingMore] = useState(false);
  const recentRequestIdRef = useRef(0);
  const recentInitialRequestIdRef = useRef(0);
  const recentLoadMoreRequestIdRef = useRef(0);
  const [recentTags, setRecentTags] = useState<string[]>([]);
  const [tags, setTags] = useState<Tag[]>([]);
  const [tagRefreshVersion, setTagRefreshVersion] = useState(0);
  const [activeSection, setActiveSection] = useState<WorkbenchSection>("library");
  const activeSectionRef = useRef(activeSection);
  useEffect(() => {
    activeSectionRef.current = activeSection;
  }, [activeSection]);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [filterOpen, setFilterOpen] = useState(false);
  const [gridDensity, setGridDensity] = useState<"compact" | "comfortable">("compact");
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

  const refreshTagState = useCallback(async () => {
    const [tagList] = await Promise.all([
      listTags(),
      loadRecentTags(),
    ]);
    setTags(tagList);
    const valid = new Set(tagList.map(tag => tag.id));
    setDiscovery(d => ({ ...d, tag_ids: d.tag_ids.filter(id => valid.has(id)), excluded_tag_ids: d.excluded_tag_ids.filter(id => valid.has(id)) }));
    setTagRefreshVersion((version) => version + 1);
  }, [loadRecentTags]);

  const loadData = useCallback(async () => {
    try {
      const [assetList, folderList, tagList, collectionList, tagMetaList] = await Promise.all([
        listAssets(),
        listLibraryFolders(),
        listAssetTags(),
        listCollections(),
        listTags(),
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
      setDataVersion(v => v + 1);
      setFolders(folderList);
      setCollections(collectionList);
      setTags(tagMetaList);

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

  const loadRecentActivity = useCallback(async (offset: number = 0) => {
    if (activeSection !== "recent") return;
    const requestId = ++recentRequestIdRef.current;
    if (offset === 0) {
      recentInitialRequestIdRef.current = requestId;
      setRecentLoading(true);
    } else {
      recentLoadMoreRequestIdRef.current = requestId;
      setRecentLoadingMore(true);
    }
    try {
      const page = await listRecentActivity({
        query: recentQuery,
        period: recentPeriod,
        action_type: recentActionType,
        limit: RECENT_PAGE_SIZE,
        offset,
      });
      if (requestId !== recentRequestIdRef.current) return;
      if (offset === 0) {
        setRecentItems(page.items);
      } else {
        setRecentItems((prev) => [...prev, ...page.items]);
      }
      setRecentTotalCount(page.total_count);
    } catch (e) {
      if (requestId !== recentRequestIdRef.current) return;
      showError(e);
    } finally {
      if (offset === 0 && requestId === recentInitialRequestIdRef.current) {
        setRecentLoading(false);
      } else if (offset !== 0 && requestId === recentLoadMoreRequestIdRef.current) {
        setRecentLoadingMore(false);
      }
    }
  }, [activeSection, recentActionType, recentPeriod, recentQuery, showError]);

  const loadRecentActivityRef = useRef(loadRecentActivity);
  useEffect(() => {
    loadRecentActivityRef.current = loadRecentActivity;
  }, [loadRecentActivity]);

  const refreshRecentIfVisible = useCallback(() => {
    if (activeSection === "recent") {
      void loadRecentActivity(0);
    }
  }, [activeSection, loadRecentActivity]);

  const handleLoadMoreRecent = useCallback(() => {
    if (recentLoading || recentLoadingMore || recentItems.length >= recentTotalCount) return;
    void loadRecentActivity(recentItems.length);
  }, [recentLoading, recentLoadingMore, recentItems.length, recentTotalCount, loadRecentActivity]);

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
    loadRecentActivity(0);
  }, [loadRecentActivity]);

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
      discovery,
      search_file_name: scope.fileName,
      search_note: scope.note,
      search_path: scope.path,
      search_tags: scope.tag,
      asset_type: ["all", "favorites", "missing"].includes(activeFilter) ? null : activeFilter,
      library_folder_id: selectedFolderId,
      collection_id: selectedCollectionId,
      is_favorite: favoriteOnly ? true : null,
      is_missing: missingOnly ? true : null,
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
  }, [query, scope, activeFilter, favoriteOnly, missingOnly, selectedFolderId, selectedCollectionId, filters, sort, discovery]);

  const executeSearch = useCallback(async () => {
    const version = ++searchVersionRef.current;
    setIsSearching(true);
    setSearchError(false);
    setIsLoadingMore(false);
    try {
      const req = buildSearchRequest(0);
      const page = await searchAssetsPage(req);
      if (version !== searchVersionRef.current) return;

      const tagList = await listAssetTags();
      const tagMap = new Map<number, string[]>();
      for (const [assetId, tagName] of tagList) {
        const arr = tagMap.get(assetId) ?? [];
        arr.push(tagName);
        tagMap.set(assetId, arr);
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
      setSearchError(true);
      showError(e);
    } finally {
      if (version === searchVersionRef.current) {
        setIsSearching(false);
      }
    }
  }, [buildSearchRequest, showError]);

  const handleLoadMore = useCallback(async () => {
    if (isSearching || isLoadingMore || gridAssets.length >= totalCount) return;
    const version = searchVersionRef.current;
    setIsLoadingMore(true);
    try {
      const req = buildSearchRequest(gridAssets.length);
      const page = await searchAssetsPage(req);
      if (version !== searchVersionRef.current) return;

      const tagList = await listAssetTags();
      if (version !== searchVersionRef.current) return;

      const tagMap = new Map<number, string[]>();
      for (const [assetId, tagName] of tagList) {
        const arr = tagMap.get(assetId) ?? [];
        arr.push(tagName);
        tagMap.set(assetId, arr);
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
  }, [isSearching, isLoadingMore, gridAssets.length, totalCount, buildSearchRequest, showError]);

  useEffect(() => {
    executeSearch();
  }, [executeSearch, dataVersion]);

  useEffect(() => {
    let cancelled = false; setFacetError("");
    listFacetTags(buildSearchRequest(0)).then(rows => { if (!cancelled) setFacetTags(rows); })
      .catch(e => { if (!cancelled) { setFacetTags([]); setFacetError(String(e?.message ?? e)); } });
    return () => { cancelled = true; };
  }, [buildSearchRequest, dataVersion, tagRefreshVersion, gridAssets]);

  const gridResetKey = JSON.stringify({
    discovery,
    query: query.trim(),
    scope,
    activeFilter,
    favoriteOnly, missingOnly,
    selectedFolderId,
    selectedCollectionId,
    filters,
    sort,
  });

  useEffect(() => { gridScrollRef.current = 0; setSelectedIds([]); }, [gridResetKey]);

  const displayAssets = gridAssets;
  const displayedTotalCount = totalCount;
  const canLoadMore = displayAssets.length < totalCount;

  const selectedAssets = selectedIds
    .map((id) => {
      const fromRecent = recentItems.find((item) => item.asset.id === id)?.asset;
      if (activeSection === "recent" && fromRecent) return fromRecent;
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
      const toggle = (a: Asset) => a.id === asset.id ? { ...a, is_favorite: !asset.is_favorite } : a;
      setAssets((prev) => prev.map(toggle));
      setGridAssets((prev) => prev.map(toggle));
      setRecentItems(prev => prev.map(item => ({ ...item, asset: toggle(item.asset) })));
      if (favoriteOnly) await executeSearch();
    } catch (e) {
      showError(e);
    }
  }, [showError, favoriteOnly, executeSearch]);

  const handleApplyTag = useCallback(async (tagName: string, assetIds: number[]) => {
    try {
      await applyTagToAssets(tagName, assetIds);
      await refreshTagState();
      await executeSearch();
      showToast("标签已添加", "success");
    } catch (e) {
      showError(e);
    }
  }, [executeSearch, refreshTagState, showError, showToast]);

  const handleCreateCollection = useCallback(async (name: string) => {
    try {
      const col = await createCollection(name, "");
      setCollections((prev) => [...prev, col].sort((a, b) => a.name.localeCompare(b.name)));
    } catch (e) {
      showError(e);
    }
  }, [showError]);

  const handleAddToCollection = useCallback(
    async (collectionId: number, assetIds: number[]) => {
      try {
        await addAssetsToCollection(collectionId, assetIds);
        const collectionList = await listCollections();
        setCollections(collectionList);
        await executeSearch();
        showToast("已加入集合", "success");
      } catch (e) {
        showError(e);
      }
    },
    [executeSearch, showError, showToast]
  );

  const handleUpdateCollection = useCallback(
    async (
      collectionId: number,
      name: string,
      description: string
    ) => {
      try {
        const updated = await updateCollection(
          collectionId,
          name,
          description
        );
        setCollections((previous) =>
          previous
            .map((collection) =>
              collection.id === updated.id ? updated : collection
            )
            .sort((left, right) => left.name.localeCompare(right.name))
        );
        showToast("集合已保存", "success");
      } catch (e) {
        showError(e);
      }
    },
    [showError, showToast]
  );

  const handleDeleteCollection = useCallback(
    async (collectionId: number) => {
      try {
        const deleted = await deleteCollection(collectionId);
        if (!deleted) {
          showToast("集合不存在或已删除", "error");
          return;
        }
        setCollections((previous) =>
          previous.filter((collection) => collection.id !== collectionId)
        );
        if (selectedCollectionId === collectionId) {
          setSelectedCollectionId(null);
          setSelectedIds([]);
        }
        showToast("集合已删除，原始素材未改动", "success");
      } catch (e) {
        showError(e);
      }
    },
    [selectedCollectionId, showError, showToast]
  );

  const handleRemoveFromCollection = useCallback(
    async (collectionId: number, assetIds: number[]) => {
      try {
        await removeAssetsFromCollection(collectionId, assetIds);
        const collectionList = await listCollections();
        setCollections(collectionList);
        setSelectedIds([]);
        await executeSearch();
        showToast(
          assetIds.length > 1
            ? `已从集合移出 ${assetIds.length} 个资源`
            : "已从集合移出资源",
          "success"
        );
      } catch (e) {
        showError(e);
      }
    },
    [executeSearch, showError, showToast]
  );

  const handleUpdateTag = useCallback(async (
    tagId: number,
    name: string,
    color: string
  ) => {
    try {
      await updateTag(tagId, name, color);
      await refreshTagState();
      await executeSearch();
      showToast("标签已保存", "success");
    } catch (e) {
      showError(e);
    }
  }, [executeSearch, refreshTagState, showError, showToast]);

  const handleDeleteTag = useCallback(async (tagId: number) => {
    try {
      const deleted = await deleteTag(tagId);
      if (!deleted) {
        showToast("标签不存在或已删除", "error");
        return;
      }
      await refreshTagState();
      await executeSearch();
      showToast("标签已删除，原始素材未改动", "success");
    } catch (e) {
      showError(e);
    }
  }, [executeSearch, refreshTagState, showError, showToast]);

  const handleRemoveTag = useCallback(async (
    tagName: string,
    assetIds: number[]
  ) => {
    const tag = tags.find(
      (candidate) => candidate.name.toLowerCase() === tagName.toLowerCase()
    );
    if (!tag) {
      showToast("标签不存在或已删除", "error");
      return;
    }
    try {
      await removeTagFromAssets(tag.id, assetIds);
      await refreshTagState();
      await executeSearch();
      showToast(
        assetIds.length > 1
          ? `已从 ${assetIds.length} 个资源解除标签`
          : "已解除标签",
        "success"
      );
    } catch (e) {
      showError(e);
    }
  }, [executeSearch, refreshTagState, showError, showToast, tags]);

  const handleUpdateNote = useCallback((updated: Asset) => {
    setAssets((prev) => prev.map((a) => (a.id === updated.id ? { ...a, note: updated.note } : a)));
    setGridAssets((prev) => prev.map((a) => (a.id === updated.id ? { ...a, note: updated.note } : a)));
    setRecentItems(prev => prev.map(item => item.asset.id === updated.id ? { ...item, asset: { ...item.asset, note: updated.note } } : item));
    void executeSearch();
  }, [executeSearch]);

  const handleOpenFile = useCallback(async (asset: Asset) => {
    try {
      await openAssetFile(asset.absolute_path);
    } catch (e) {
      showError(e);
      return;
    }
    recordRecentAssetAction(asset.id, "open_file")
      .then(() => refreshRecentIfVisible())
      .catch((e) => console.error("Failed to record recent action", e));
  }, [showError, refreshRecentIfVisible]);

  const handleRevealFile = useCallback(async (asset: Asset) => {
    try {
      await revealAssetInFolder(asset.absolute_path);
    } catch (e) {
      showError(e);
      return;
    }
    recordRecentAssetAction(asset.id, "reveal_folder")
      .then(() => refreshRecentIfVisible())
      .catch((e) => console.error("Failed to record recent action", e));
  }, [showError, refreshRecentIfVisible]);

  const handleDeleteFolder = useCallback(async (folderId: number) => {
    try {
      await deleteLibraryFolder(folderId);
      if (selectedFolderId === folderId) {
        setDiscovery(d => ({ ...d, directory_path: null }));
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

  const handleCopyAssetText = useCallback(async (asset: Asset, text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch (e) {
      showToast((e as any)?.message ?? "复制路径失败", "error");
      return;
    }
    showToast("路径已复制", "success");
    recordRecentAssetAction(asset.id, "copy_path")
      .then(() => refreshRecentIfVisible())
      .catch((e) => console.error("Failed to record recent action", e));
  }, [showToast, refreshRecentIfVisible]);

  const handleWorkbenchSection = useCallback(
    (section: WorkbenchSection) => {
      if (section === activeSection) {
        setSidebarOpen((open) => !open);
        return;
      }
      if (section === "recent") {
        previousSectionRef.current = activeSection;
        librarySelectionRef.current = selectedIds;
        setSelectedIds([]);
      } else if (activeSection === "recent") {
        setSelectedIds(librarySelectionRef.current);
      }
      setActiveSection(section);
      setSidebarOpen(true);
    },
    [activeSection, selectedIds]
  );

  const handleRecentPeriodChange = useCallback((value: RecentActivityPeriod) => {
    setRecentPeriod(value);
    setSelectedIds([]);
    setRecentItems([]);
    setRecentTotalCount(0);
  }, []);

  const handleRecentActionTypeChange = useCallback((value: "all" | RecentActionType) => {
    setRecentActionType(value);
    setSelectedIds([]);
    setRecentItems([]);
    setRecentTotalCount(0);
  }, []);

  const handleCopyText = (text: string, asset?: Asset) => {
    if (asset) void handleCopyAssetText(asset, text);
  };
  const handleCopyPath = (asset: Asset) => handleCopyAssetText(asset, asset.absolute_path);
  const clearAllConditions = () => {
    setDiscovery(defaultDiscovery());
    setActiveFilter("all"); setFavoriteOnly(false); setMissingOnly(false);
    setSelectedFolderId(null); setSelectedCollectionId(null); setFilters(DEFAULT_SEARCH_FILTERS);
    setScope({ fileName: true, tag: true, note: true, path: true }); setSelectedIds([]);
  };
  const openQuickLook = (asset: Asset) => {
    if (asset.asset_type !== "image" || asset.is_missing) return;
    const candidates = (activeSection === "recent" ? recentItems.map(item => item.asset) : gridAssets)
      .filter(a => a.asset_type === "image" && !a.is_missing);
    if (!candidates.some(a => a.id === asset.id)) candidates.push(asset);
    setSelectedIds([asset.id]); setQuickLook({ assets: candidates, initialId: asset.id });
  };

  const refreshClassification = async () => {
    await refreshTagState(); await executeSearch(); refreshRecentIfVisible();
  };
  const selectLibraryFolder = (id: number | null) => {
    setSelectedFolderId(id); setDiscovery(d => ({ ...d, directory_path: null })); setSelectedIds([]);
  };
  const browseDirectory = (asset: Asset) => {
    clearAllConditions(); setQuery(""); setSelectedFolderId(asset.library_folder_id);
    const path = asset.absolute_path.replace(/\\/g, "/");
    setDiscovery({ ...defaultDiscovery(), directory_path: path.slice(0, path.lastIndexOf("/")), recursive: false });
    setActiveSection("library"); setSidebarOpen(true);
  };
  const currentFolder = folders.find(f => f.id === selectedFolderId);
  const directoryRoot = currentFolder?.path.replace(/\\/g, "/").replace(/\/$/, "") ?? "";
  const directoryParts = discovery.directory_path && directoryRoot ? discovery.directory_path.slice(directoryRoot.length).split("/").filter(Boolean) : [];

  const handleMediaPlaybackStarted = useCallback((asset: Asset) => {
    void recordMediaPreviewActivity(
      asset,
      () => activeSectionRef.current,
      () => loadRecentActivityRef.current()
    );
  }, []);

  const isEmptySearch = displayAssets.length === 0;
  const hasFolders = folders.length > 0;
  const hasScanned = hasFolders && assets.length > 0;
  const hasAdvancedFilters = Object.values(filters).some((value) => value != null);
  const showSearchEmpty =
    query.trim() !== "" ||
    activeFilter !== "all" || favoriteOnly || missingOnly ||
    selectedFolderId != null ||
    selectedCollectionId != null ||
    hasAdvancedFilters || discovery.directory_path !== null || discovery.tag_ids.length > 0 || discovery.excluded_tag_ids.length > 0 || discovery.unclassified;

  return (
    <main
      className={sidebarOpen ? "app-shell" : "app-shell sidebar-collapsed"}
    >
      <NavigationRail
        activeSection={activeSection}
        sidebarOpen={sidebarOpen}
        onSelect={handleWorkbenchSection}
      />
      <LibrarySidebar
        folders={folders}
        collections={collections}
        activeFilter={activeFilter}
        favoriteOnly={favoriteOnly}
        missingOnly={missingOnly}
        activeSection={activeSection}
        hidden={!sidebarOpen}
        selectedFolderId={selectedFolderId}
        selectedCollectionId={selectedCollectionId}
        recentPeriod={recentPeriod}
        recentActionType={recentActionType}
        onFilterChange={(f) => {
          if (f === "favorites") setFavoriteOnly(v => !v);
          else if (f === "missing") setMissingOnly(v => !v);
          else if (f === "all") { setFavoriteOnly(false); setMissingOnly(false); setActiveFilter("all"); }
          else setActiveFilter(v => v === f ? "all" : f);
          setSelectedIds([]);
        }}
        onSelectFolder={selectLibraryFolder}
        onSelectCollection={(id) => { setSelectedCollectionId(id); setSelectedIds([]); }}
        onRecentPeriodChange={handleRecentPeriodChange}
        onRecentActionTypeChange={handleRecentActionTypeChange}
        onPickFolder={handlePickFolder}
        onScanFolder={handleScanFolder}
        onCancelScan={handleCancelScan}
        onDeleteFolder={handleDeleteFolder}
        onOpenFolder={handleOpenFolder}
        onCreateCollection={handleCreateCollection}
        onUpdateCollection={handleUpdateCollection}
        onDeleteCollection={handleDeleteCollection}
        tags={tags}
        onUpdateTag={handleUpdateTag}
        onDeleteTag={handleDeleteTag}
        isScanning={isScanning}
        latestJobs={latestJobs}
        folderCounts={folderCounts}
        directoryPanel={currentFolder && <DirectoryTree folder={currentFolder} selectedPath={discovery.directory_path} version={dataVersion} onSelect={path => setDiscovery(d => ({ ...d, directory_path: path }))} />}
        categoryPanel={<CategoryFilters tags={facetTags} filter={discovery} onChange={setDiscovery} selectedIds={selectedIds} onSaved={refreshClassification} onRules={() => setRulesOpen(true)} error={facetError} />}
        settingsPanel={
          scanSettings ? (
            <SettingsPanel settings={scanSettings} onChange={handleScanSettingsChange} />
          ) : null
        }
      />
      <section className="workspace">
        {activeSection !== "recent" && scanMessage && (
          <div className="scan-summary" onClick={() => setScanMessage(null)}>
            {scanMessage}
          </div>
        )}
        {activeSection !== "recent" &&
          Object.values(latestJobs)
            .filter(Boolean)
            .map((job) => (
              <ScanStatusBar key={job!.id} job={job!} />
            ))}
        {activeSection === "recent" && <div className="recent-search-bar">
          <button onClick={() => handleWorkbenchSection(previousSectionRef.current)}>返回上次查找</button>
          <input aria-label="搜索最近活动" placeholder="搜索活动中的文件名或路径" value={recentQuery} onChange={e => { setRecentQuery(e.target.value); setRecentItems([]); setRecentTotalCount(0); setSelectedIds([]); }} />
          <small>保留最近 30 天，最多 1000 条操作记录</small>
        </div>}
        {activeSection === "recent" ? (
          recentItems.length === 0 && !recentLoading ? (
            <EmptyState
              variant={
                recentPeriod === "all" && recentActionType === "all" && !recentQuery.trim()
                  ? "no-recent-activity"
                  : "no-recent-filter-results"
              }
            />
          ) : (
            <RecentActivityTimeline
              items={recentItems}
              selectedIds={selectedIds}
              totalCount={recentTotalCount}
              loading={recentLoading}
              loadingMore={recentLoadingMore}
              canLoadMore={recentItems.length < recentTotalCount}
              onSelectAsset={(id) => setSelectedIds([id])}
              onOpenAsset={handleOpenFile}
              onLoadMore={handleLoadMoreRecent}
            />
          )
        ) : (
          <>
            <SearchToolbar
              query={query}
              sort={sort}
              totalCount={displayedTotalCount}
              filterOpen={filterOpen}
              density={gridDensity}
              onQueryChange={(q) => { setQuery(q); setSelectedIds([]); }}
              onSortChange={handleSortChange}
              onToggleFilters={() => setFilterOpen((prev) => !prev)}
              onDensityChange={(d) => setGridDensity(d)}
            />
            {currentFolder && <nav className="directory-breadcrumb" aria-label="当前目录范围">
              <button onClick={() => setDiscovery(d => ({ ...d, directory_path: null }))}>{currentFolder.name}</button>
              {directoryParts.map((part, index) => <button key={index} onClick={() => setDiscovery(d => ({ ...d, directory_path: directoryRoot + "/" + directoryParts.slice(0, index + 1).join("/") }))}>/ {part}</button>)}
              {discovery.directory_path && <label><input type="checkbox" checked={discovery.recursive} onChange={e => setDiscovery(d => ({ ...d, recursive: e.target.checked }))} />包含子目录</label>}
            </nav>}
            <FilterPanel
              open={filterOpen}
              scope={scope}
              filters={filters}
              onScopeChange={(s) => { setScope(s); setSelectedIds([]); }}
              onFiltersChange={handleFiltersChange}
            />
            <ActiveFilterChips
              onClearAll={clearAllConditions}
              extraChips={[
                ...(discovery.directory_path ? [{ id: "directory", label: `目录：${discovery.directory_path.split("/").pop()}（${discovery.recursive ? "含子目录" : "仅本层"}）`, onRemove: () => setDiscovery(d => ({ ...d, directory_path: null })) }] : []),
                ...discovery.tag_ids.map(id => ({ id: `tag-${id}`, label: `分类：${tags.find(t => t.id === id)?.name ?? id}`, onRemove: () => setDiscovery(d => ({ ...d, tag_ids: d.tag_ids.filter(v => v !== id) })) })),
                ...discovery.excluded_tag_ids.map(id => ({ id: `exclude-${id}`, label: `排除：${tags.find(t => t.id === id)?.name ?? id}`, onRemove: () => setDiscovery(d => ({ ...d, excluded_tag_ids: d.excluded_tag_ids.filter(v => v !== id) })) })),
                ...(discovery.unclassified ? [{ id: "unclassified", label: "尚未分类", onRemove: () => setDiscovery(d => ({ ...d, unclassified: false })) }] : []),
                ...(selectedFolderId == null ? [] : [{ id: "folder", label: `资源库：${folders.find(f => f.id === selectedFolderId)?.name ?? selectedFolderId}`, onRemove: () => selectLibraryFolder(null) }]),
                ...(selectedCollectionId == null ? [] : [{ id: "collection", label: `集合：${collections.find(c => c.id === selectedCollectionId)?.name ?? selectedCollectionId}`, onRemove: () => setSelectedCollectionId(null) }]),
                ...(activeFilter === "all" ? [] : [{ id: "type", label: `类型：${({ image: "图片", audio: "音频", video: "视频", font: "字体", model3d: "3D", spine: "Spine", other: "未识别" } as Record<string, string>)[activeFilter] ?? activeFilter}`, onRemove: () => setActiveFilter("all") }]),
                ...(favoriteOnly ? [{ id: "favorite", label: "仅收藏", onRemove: () => setFavoriteOnly(false) }] : []),
                ...(missingOnly ? [{ id: "missing", label: "仅缺失", onRemove: () => setMissingOnly(false) }] : []),
              ]}
              filters={filters}
              scope={scope}
              onFiltersChange={handleFiltersChange}
              onScopeChange={(s) => { setScope(s); setSelectedIds([]); }}
            />
            {isSearching && <div role="status">正在更新结果，下方为上次结果…</div>}
            {searchError && <div role="alert">搜索失败，下方为上次结果。<button onClick={() => void executeSearch()}>重试</button></div>}
            {!hasFolders ? (
              <EmptyState variant="no-folders" />
            ) : isEmptySearch && (hasScanned || showSearchEmpty) ? (
              <><EmptyState variant="no-results" /><div className="empty-recovery">
                {!scope.path && <button onClick={() => setScope(s => ({ ...s, path: true }))}>同时搜索路径</button>}
                {selectedFolderId != null && <button onClick={() => selectLibraryFolder(null)}>搜索所有资源库</button>}
                <button onClick={clearAllConditions}>保留关键词，清除条件</button>
              </div></>
            ) : !hasScanned ? (
              <EmptyState variant="no-assets" />
            ) : (
              <>
                <AssetGrid
                  assets={displayAssets}
                  selectedIds={selectedIds}
                  onSelectionChange={setSelectedIds}
                  onToggleFavorite={handleToggleFavorite}
                  resetKey={gridResetKey}
                  density={gridDensity}
                  onQuickLook={openQuickLook}
                  initialScrollTop={gridScrollRef.current}
                  onScrollPosition={top => { gridScrollRef.current = top; }}
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
        activeCollectionId={selectedCollectionId}
        onRemoveFromCollection={handleRemoveFromCollection}
        onRemoveTag={handleRemoveTag}
        tagRefreshVersion={tagRefreshVersion}
        recentTags={recentTags}
        onCopyText={handleCopyText}
        onQuickLook={openQuickLook}
        onBrowseDirectory={browseDirectory}
        projectRoot={projectRoot}
        onProjectRootChange={setProjectRoot}
        onMediaPlaybackStarted={handleMediaPlaybackStarted}
      />
      {rulesOpen && <ClassificationRules tags={facetTags} request={buildSearchRequest(0)} onClose={() => setRulesOpen(false)} onChanged={refreshClassification} />}
      {quickLook && <ImageQuickLook {...quickLook} folders={folders} onSelect={setId => setSelectedIds([setId])}
        onClose={() => setQuickLook(null)} onViewed={asset => {
          void recordRecentAssetAction(asset.id, "preview_image").then(refreshRecentIfVisible).catch(console.error);
        }} />}
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

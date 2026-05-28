import { useEffect, useMemo, useState } from "react";
import { applyTagToAssets, listAssets, listLibraryFolders, openAssetFile, revealAssetInFolder } from "./api/tauri";
import { AssetGrid } from "./components/AssetGrid";
import { DetailsPanel } from "./components/DetailsPanel";
import { LibrarySidebar } from "./components/LibrarySidebar";
import { SearchToolbar } from "./components/SearchToolbar";
import type { Asset, LibraryFolder, SearchScope } from "./types/asset";

export default function App() {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [folders, setFolders] = useState<LibraryFolder[]>([]);
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [activeFilter, setActiveFilter] = useState("all");
  const [query, setQuery] = useState("");
  const [scope, setScope] = useState<SearchScope>({ fileName: true, tag: true, note: true, path: false });

  useEffect(() => {
    listAssets().then(setAssets).catch(console.error);
    listLibraryFolders().then(setFolders).catch(console.error);
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
      ];
      return haystacks.some((value) => value.toLowerCase().includes(normalizedQuery));
    });
  }, [activeFilter, assets, query, scope]);

  const selectedAssets = assets.filter((asset) => selectedIds.includes(asset.id));

  return (
    <main className="app-shell">
      <LibrarySidebar folders={folders} activeFilter={activeFilter} onFilterChange={setActiveFilter} />
      <section className="workspace">
        <SearchToolbar query={query} scope={scope} onQueryChange={setQuery} onScopeChange={setScope} />
        <AssetGrid assets={filteredAssets} selectedIds={selectedIds} onSelectionChange={setSelectedIds} />
      </section>
      <DetailsPanel
        selectedAssets={selectedAssets}
        onOpenFile={(asset) => openAssetFile(asset.absolute_path)}
        onReveal={(asset) => revealAssetInFolder(asset.absolute_path)}
        onCopyPath={(asset) => navigator.clipboard.writeText(asset.absolute_path)}
        onApplyTag={(tagName, assetIds) => applyTagToAssets(tagName, assetIds)}
      />
    </main>
  );
}

import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { LibrarySidebar } from "./LibrarySidebar";
import type { Collection, Tag } from "../types/asset";

function makeFolder(id: number, name: string) {
  return { id, name, path: `C:/${name.toLowerCase()}`, created_at: "", last_scanned_at: null, is_enabled: true };
}

function makeCollection(id: number, name: string): Collection {
  return { id, name, description: "", asset_count: 0 };
}

describe("LibrarySidebar", () => {
  it("shows recent activity filter", () => {
    render(
      <LibrarySidebar
        folders={[]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    expect(screen.getByRole("button", { name: /最近使用/ })).toBeInTheDocument();
  });

  it("calls onPickFolder when add folder button is clicked", async () => {
    const onPickFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={onPickFolder}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    await userEvent.click(screen.getByText("添加文件夹"));

    expect(onPickFolder).toHaveBeenCalled();
  });

  it("calls onScanFolder when scan button is clicked", async () => {
    const onScanFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={onScanFolder}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    await userEvent.click(screen.getByText("扫描"));

    expect(onScanFolder).toHaveBeenCalledWith(1);
  });

  it("calls onSelectFolder when folder row is clicked", async () => {
    const onSelectFolder = vi.fn();
    const onSelectCollection = vi.fn();
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={onSelectFolder}
        onSelectCollection={onSelectCollection}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    await userEvent.click(screen.getByTitle("C:/assets"));

    expect(onSelectFolder).toHaveBeenCalledWith(1);
    expect(onSelectCollection).not.toHaveBeenCalled();
  });

  it("clicking scan button does not trigger row selection", async () => {
    const onSelectFolder = vi.fn();
    const onScanFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={onSelectFolder}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={onScanFolder}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    await userEvent.click(screen.getByText("扫描"));

    expect(onScanFolder).toHaveBeenCalledWith(1);
    expect(onSelectFolder).not.toHaveBeenCalled();
  });

  it("disables scan button when isScanning is true", () => {
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={true}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    const btn = screen.getByText("扫描") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("deselects folder on second click", async () => {
    const onSelectFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={1}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={onSelectFolder}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    await userEvent.click(screen.getByTitle("C:/assets"));

    expect(onSelectFolder).toHaveBeenCalledWith(null);
  });

  it("shows collection names and calls onSelectCollection on click", async () => {
    const onSelectCollection = vi.fn();
    const onSelectFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[]}
        collections={[makeCollection(1, "Heroes"), makeCollection(2, "Icons")]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={onSelectFolder}
        onSelectCollection={onSelectCollection}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    expect(screen.getByText("Heroes")).toBeInTheDocument();
    expect(screen.getByText("Icons")).toBeInTheDocument();

    await userEvent.click(screen.getByText("Heroes"));

    expect(onSelectCollection).toHaveBeenCalledWith(1);
    expect(onSelectFolder).not.toHaveBeenCalled();
  });

  it("calls onCreateCollection when form is submitted with non-empty name", async () => {
    const onCreateCollection = vi.fn();
    render(
      <LibrarySidebar
        folders={[]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={onCreateCollection}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    const input = screen.getByPlaceholderText("新建集合...");
    await userEvent.type(input, "My Set");
    await userEvent.click(screen.getByText("创建"));

    expect(onCreateCollection).toHaveBeenCalledWith("My Set");
  });

  it("confirms folder index removal without implying disk file deletion", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    await userEvent.click(screen.getByTitle("从资源库移除索引"));

    expect(confirmSpy).toHaveBeenCalledWith(
      "确定要从资源库移除「Assets」的索引吗？这只会删除应用内索引和关联整理数据，不会删除磁盘上的原始文件。"
    );
    expect(screen.queryByTitle("删除文件夹")).not.toBeInTheDocument();

    confirmSpy.mockRestore();
  });

  it("calls onOpenFolder when open folder button is clicked", async () => {
    const onOpenFolder = vi.fn();
    const folder = makeFolder(1, "Assets");
    render(
      <LibrarySidebar
        folders={[folder]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={onOpenFolder}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    await userEvent.click(screen.getByTitle("打开文件夹"));

    expect(onOpenFolder).toHaveBeenCalledWith(folder);
  });

  it("displays folder asset counts and missing info", () => {
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{ 1: { folder_id: 1, total: 12, missing: 3, is_accessible: true } }}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    expect(screen.getByText(/12 个资源/)).toBeInTheDocument();
    expect(screen.getByText(/3 个缺失/)).toBeInTheDocument();
  });

  it("displays inaccessible status when folder path is not accessible", () => {
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{ 1: { folder_id: 1, total: 5, missing: 0, is_accessible: false } }}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    expect(screen.getByText(/路径不可访问/)).toBeInTheDocument();
  });

  it("does not call onDeleteFolder when removal is cancelled", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
    const onDeleteFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={onDeleteFolder}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    await userEvent.click(screen.getByTitle("从资源库移除索引"));

    expect(onDeleteFolder).not.toHaveBeenCalled();
    confirmSpy.mockRestore();
  });

  it("calls onDeleteFolder when removal is confirmed", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
    const onDeleteFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={onDeleteFolder}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    await userEvent.click(screen.getByTitle("从资源库移除索引"));

    expect(onDeleteFolder).toHaveBeenCalledWith(1);
    confirmSpy.mockRestore();
  });

  it("opens a folder management panel for backup cleanup", async () => {
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{ 1: { folder_id: 1, total: 12, missing: 2, is_accessible: true } }}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "管理文件夹" }));

    const dialog = screen.getByRole("dialog", { name: "资源库文件夹管理" });
    expect(dialog).toBeInTheDocument();
    expect(screen.getByText("备份工程前，可以在这里移除应用内索引。不会删除、移动或修改磁盘上的原始文件。")).toBeInTheDocument();
    expect(within(dialog).getByText("C:/assets")).toBeInTheDocument();
    expect(within(dialog).getByText(/12 个资源/)).toBeInTheDocument();
    expect(within(dialog).getByText(/2 个缺失/)).toBeInTheDocument();
  });

  it("removes a folder index from the management panel after confirmation", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
    const onDeleteFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={onDeleteFolder}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "管理文件夹" }));
    await userEvent.click(screen.getByRole("button", { name: "从资源库移除 Assets 的索引" }));

    expect(confirmSpy).toHaveBeenCalledWith(
      "确定要从资源库移除「Assets」的索引吗？这只会删除应用内索引和关联整理数据，不会删除磁盘上的原始文件。"
    );
    expect(onDeleteFolder).toHaveBeenCalledWith(1);
    confirmSpy.mockRestore();
  });

  it("disables management panel removal while a folder is scanning", async () => {
    render(
      <LibrarySidebar
        folders={[makeFolder(1, "Assets")]}
        collections={[]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={true}
        latestJobs={{
          1: {
            id: 10,
            library_folder_id: 1,
            status: "running",
            started_at: "2026-06-08T00:00:00Z",
            finished_at: null,
            cancelled_at: null,
            found_count: 1,
            added_count: 0,
            updated_count: 0,
            unchanged_count: 0,
            missing_count: 0,
            skipped_count: 0,
            current_path: null,
            error_message: null,
          },
        }}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "管理文件夹" }));

    expect(screen.getByRole("button", { name: "从资源库移除 Assets 的索引" })).toBeDisabled();
    expect(screen.getByText("扫描中，先取消扫描后再移除索引")).toBeInTheDocument();
  });

  it("shows collection asset counts and opens collection management", async () => {
    render(
      <LibrarySidebar
        folders={[]}
        collections={[{ id: 1, name: "角色", description: "", asset_count: 12 }]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        tags={[]}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />
    );

    expect(screen.getByText("12")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "管理集合" }));
    expect(screen.getByRole("dialog", { name: "集合管理" })).toBeInTheDocument();
  });

  it("opens the tag manager", async () => {
    render(
      <LibrarySidebar
        folders={[]}
        collections={[]}
        tags={[{ id: 1, name: "角色", color: "#5B8DEF", asset_count: 3 }]}
        activeFilter="all"
        selectedFolderId={null}
        selectedCollectionId={null}
        onFilterChange={vi.fn()}
        onSelectFolder={vi.fn()}
        onSelectCollection={vi.fn()}
        onPickFolder={vi.fn()}
        onScanFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onDeleteFolder={vi.fn()}
        onOpenFolder={vi.fn()}
        onCreateCollection={vi.fn()}
        onUpdateCollection={vi.fn()}
        onDeleteCollection={vi.fn()}
        onUpdateTag={vi.fn()}
        onDeleteTag={vi.fn()}
        isScanning={false}
        latestJobs={{}}
        folderCounts={{}}
        settingsPanel={null}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "管理标签" }));
    expect(screen.getByRole("dialog", { name: "标签管理" })).toBeInTheDocument();
  });
});
